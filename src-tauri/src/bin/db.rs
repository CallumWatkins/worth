use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::collections::HashSet;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DB_FILENAME: &str = "worth.sqlite";

#[derive(Debug, Parser)]
#[command(
    name = "db",
    about = "Database dev operations",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Backup the database
    Backup(BackupArgs),
    /// Restore a backup by name after backing up the current database
    Restore(RestoreArgs),
    /// Backup, clear, and seed the database
    Seed(SeedArgs),
    /// Backup and clear the database
    Clear(ClearArgs),
    /// Delete database backup files
    Clean(ConfirmArgs),
}

#[derive(Debug, Args)]
struct ConfirmArgs {
    /// Skip confirmation prompt
    #[arg(short = 'y', long = "yes")]
    yes: bool,
}

#[derive(Debug, Args)]
struct BackupArgs {
    /// Backup name (defaults to a timestamp)
    #[arg(value_name = "BACKUP_NAME")]
    name: Option<String>,

    /// Overwrite existing backup files with the same backup name
    #[arg(long = "backup-overwrite")]
    backup_overwrite: bool,
}

#[derive(Debug, Args)]
struct RestoreArgs {
    /// Backup name to restore (e.g. a timestamp). If omitted, you'll be prompted to choose.
    #[arg(value_name = "BACKUP_NAME")]
    name: Option<String>,

    /// Name for the backup created from the current database before restoring (defaults to a timestamp)
    #[arg(short = 'n', long = "backup-name", value_name = "NAME")]
    backup_name: Option<String>,

    /// Overwrite existing backup files with the same backup name (for the pre-restore backup)
    #[arg(long = "backup-overwrite")]
    backup_overwrite: bool,

    #[command(flatten)]
    confirm: ConfirmArgs,
}

#[derive(Debug, Args)]
struct SeedArgs {
    /// Seed name (from src-tauri/db/seeds) or a .sql file path
    #[arg(value_name = "NAME|PATH")]
    name_or_path: Option<String>,

    /// List available seeds and exit
    #[arg(long)]
    list: bool,

    /// Name for the database backup created before seeding (defaults to a timestamp)
    #[arg(short = 'n', long = "backup-name", value_name = "NAME")]
    backup_name: Option<String>,

    /// Overwrite existing backup files with the same backup name
    #[arg(long = "backup-overwrite")]
    backup_overwrite: bool,
}

#[derive(Debug, Args)]
struct ClearArgs {
    #[command(flatten)]
    confirm: ConfirmArgs,

    /// Name for the database backup created by this operation (defaults to a timestamp)
    #[arg(short = 'n', long = "backup-name", value_name = "NAME")]
    name: Option<String>,

    /// Overwrite existing backup files with the same backup name
    #[arg(long = "backup-overwrite")]
    backup_overwrite: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Backup(args) => backup_command(args),
        Command::Seed(args) => seed_command(args).await,
        Command::Clear(args) => clear_db(args),
        Command::Clean(args) => clean_db_backups(args.yes),
        Command::Restore(args) => restore_db(args),
    }
}

async fn seed_command(args: SeedArgs) -> Result<()> {
    let SeedArgs {
        name_or_path,
        list,
        backup_name,
        backup_overwrite,
    } = args;

    if list {
        print_available_seeds();
        return Ok(());
    }

    let Some(seed_arg) = name_or_path else {
        print_available_seeds();
        bail!("missing seed name (or path)")
    };

    let seed_path = resolve_seed_path(&seed_arg)?;
    let db_path = resolve_db_path()?;

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create dir {}", parent.display()))?;
    }

    let backup_name = resolve_backup_name(backup_name)?;
    backup_db_files(&db_path, &backup_name, backup_overwrite)?;

    let pool = connect_pool(&db_path).await?;

    sqlx::migrate!("./db/migrations")
        .run(&pool)
        .await
        .context("run migrations")?;

    run_seed(&pool, &seed_path).await?;

    println!("Seed complete.");
    println!("Database: {}", db_path.display());
    println!("Seed file: {}", seed_path.display());

    Ok(())
}

fn print_available_seeds() {
    let seeds = list_seed_names().unwrap_or_default();
    if seeds.is_empty() {
        eprintln!("No seed files found in {}", seeds_dir().display());
    } else {
        eprintln!("Available seeds:");
        for s in seeds {
            eprintln!("  - {s}");
        }
    }
}

fn list_seed_names() -> Result<Vec<String>> {
    let mut out = Vec::new();
    let dir = seeds_dir();
    let entries = std::fs::read_dir(&dir).with_context(|| format!("read dir {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let ext_is_sql = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("sql"));
        if !ext_is_sql {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            out.push(stem.to_string());
        }
    }
    out.sort();
    Ok(out)
}

fn resolve_seed_path(seed_arg: &str) -> Result<PathBuf> {
    let as_path = PathBuf::from(seed_arg);
    if as_path.is_file() {
        return Ok(as_path);
    }

    let has_sql_ext = seed_arg
        .rsplit_once('.')
        .is_some_and(|(_, ext)| ext.eq_ignore_ascii_case("sql"));

    let file = if has_sql_ext {
        seed_arg.to_string()
    } else {
        format!("{seed_arg}.sql")
    };

    let candidate = seeds_dir().join(file);
    if candidate.exists() {
        return Ok(candidate);
    }

    let mut msg = format!(
        "seed file not found: {seed_arg}\nlooked for: {}",
        candidate.display()
    );
    if let Ok(seeds) = list_seed_names() {
        if !seeds.is_empty() {
            msg.push_str("\n\nAvailable seeds:\n");
            for s in seeds {
                msg.push_str("  - ");
                msg.push_str(&s);
                msg.push('\n');
            }
        }
    }
    Err(anyhow!(msg))
}

fn resolve_db_path() -> Result<PathBuf> {
    let bundle_id = read_bundle_identifier()?;
    Ok(user_local_data_dir()?
        .join(bundle_id)
        .join("db")
        .join(DB_FILENAME))
}

fn read_bundle_identifier() -> Result<String> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let raw = std::fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let v: serde_json::Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    let id = v
        .get("identifier")
        .and_then(|x| x.as_str())
        .ok_or_else(|| anyhow!("missing `identifier` in {}", config_path.display()))?;
    Ok(id.to_string())
}

fn user_local_data_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let base =
            std::env::var_os("LOCALAPPDATA").ok_or_else(|| anyhow!("LOCALAPPDATA is not set"))?;
        Ok(PathBuf::from(base))
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support"))
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(base) = std::env::var_os("XDG_DATA_HOME") {
            Ok(PathBuf::from(base))
        } else {
            let home = std::env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
            Ok(PathBuf::from(home).join(".local").join("share"))
        }
    }
}

fn clear_db(args: ClearArgs) -> Result<()> {
    let ClearArgs {
        confirm,
        name,
        backup_overwrite,
    } = args;

    let db_path = resolve_db_path()?;

    if !confirm.yes {
        println!("Database: {}", db_path.display());
        println!("This will move any existing database files to named .bak backups (defaults to a timestamp).");
        println!();

        if !confirm_default_yes("Clear the database?")? {
            println!("Aborted; database was not cleared.");
            return Ok(());
        }
    }

    let backup_name = resolve_backup_name(name)?;
    backup_db_files(&db_path, &backup_name, backup_overwrite)?;
    Ok(())
}

fn clean_db_backups(assume_yes: bool) -> Result<()> {
    let db_path = resolve_db_path()?;
    let dir = db_path
        .parent()
        .ok_or_else(|| anyhow!("db path has no parent dir: {}", db_path.display()))?;

    if !dir.exists() {
        println!("No database backup files found in {}", dir.display());
        return Ok(());
    }

    let mut backups = Vec::<PathBuf>::new();
    let entries = std::fs::read_dir(dir).with_context(|| format!("read dir {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let ext_is_bak = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("bak"));
        if !file_name.starts_with(DB_FILENAME) || !ext_is_bak {
            continue;
        }
        backups.push(path);
    }

    backups.sort();

    if backups.is_empty() {
        println!("No database backup files found in {}", dir.display());
        return Ok(());
    }

    println!(
        "Found {} database backup file(s) in {}:",
        backups.len(),
        dir.display()
    );
    for path in &backups {
        println!("  - {}", path.display());
    }

    if !assume_yes {
        println!();
        if !confirm_default_yes("Delete these backup files?")? {
            println!("Aborted; no files were deleted.");
            return Ok(());
        }
    }

    let mut deleted = 0usize;
    for path in backups {
        std::fs::remove_file(&path).with_context(|| format!("remove file {}", path.display()))?;
        deleted += 1;
    }

    println!("Deleted {deleted} backup file(s).");
    Ok(())
}

fn backup_command(args: BackupArgs) -> Result<()> {
    let BackupArgs {
        name,
        backup_overwrite,
    } = args;

    let db_path = resolve_db_path()?;
    let backup_name = resolve_backup_name(name)?;

    copy_db_files_to_backup(&db_path, &backup_name, backup_overwrite)?;

    println!("Backup complete.");
    println!("Backup name: {backup_name}");
    Ok(())
}

fn restore_db(args: RestoreArgs) -> Result<()> {
    let RestoreArgs {
        name,
        backup_name,
        backup_overwrite,
        confirm,
    } = args;

    let db_path = resolve_db_path()?;

    let restore_name = match name {
        Some(name) => parse_backup_name_arg(&name)?,
        None => choose_backup_name(&db_path)?,
    };

    // Validate the backup exists before we back up the current DB (avoid leaving the user with no DB).
    let main_backup = backup_path_for(db_path.as_path(), &restore_name)?;
    if !main_backup.is_file() {
        let available = list_backup_names(&db_path).unwrap_or_default();
        if available.is_empty() {
            bail!(
                "backup not found: {restore_name}\nexpected file: {}\n(no backups found)",
                main_backup.display()
            );
        }
        bail!(
            "backup not found: {restore_name}\nexpected file: {}\n\nAvailable backups:\n  - {}",
            main_backup.display(),
            available.join("\n  - ")
        );
    }

    if !confirm.yes {
        println!("Restore backup: {restore_name}");
        println!("This will back up any existing database files to new named .bak backups (defaults to a timestamp), then restore the selected backup.");
        println!();

        if !confirm_default_yes("Proceed with restore?")? {
            println!("Aborted; database was not restored.");
            return Ok(());
        }
    }

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create dir {}", parent.display()))?;
    }

    // 1) Back up current database
    let current_backup_name = resolve_backup_name(backup_name)?;
    backup_db_files(&db_path, &current_backup_name, backup_overwrite)?;

    // 2) Restore selected database
    restore_db_files(&db_path, &restore_name)?;

    println!("Restore complete.");
    println!("Previous database backed up as: {current_backup_name}");

    Ok(())
}

fn resolve_backup_name(name: Option<String>) -> Result<String> {
    match name {
        Some(name) => parse_backup_name_arg(&name),
        None => Ok(timestamp_ms()),
    }
}

fn validate_backup_name(name: &str) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        bail!("backup name cannot be empty");
    }
    if name == "." || name == ".." {
        bail!("invalid backup name: {name}");
    }
    if name.contains('/') || name.contains('\\') {
        bail!("invalid backup name: {name} (must not contain path separators)");
    }
    if name.chars().any(|c| c == '\0') {
        bail!("invalid backup name: contains NUL");
    }
    Ok(())
}

fn parse_backup_name_arg(arg: &str) -> Result<String> {
    let name = arg.trim();
    if name.is_empty() {
        bail!("missing backup name");
    }
    validate_backup_name(name)?;
    Ok(name.to_string())
}

fn list_backup_names(db_path: &Path) -> Result<Vec<String>> {
    let dir = db_path
        .parent()
        .ok_or_else(|| anyhow!("db path has no parent dir: {}", db_path.display()))?;

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let prefix = format!("{DB_FILENAME}.");
    let prefix_lower = prefix.to_ascii_lowercase();
    let suffix = ".bak";

    let mut candidates = Vec::<(String, SystemTime)>::new();
    let entries = std::fs::read_dir(dir).with_context(|| format!("read dir {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let file_name_lower = file_name.to_ascii_lowercase();

        if !file_name_lower.starts_with(&prefix_lower) || !file_name_lower.ends_with(suffix) {
            continue;
        }

        let name = &file_name[prefix.len()..file_name.len() - suffix.len()];
        if name.is_empty() {
            continue;
        }

        let modified = entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(UNIX_EPOCH);

        candidates.push((name.to_string(), modified));
    }

    candidates.sort_by(|(a_name, a_modified), (b_name, b_modified)| {
        b_modified.cmp(a_modified).then_with(|| a_name.cmp(b_name))
    });

    let mut seen = HashSet::<String>::new();
    let mut out = Vec::<String>::new();
    for (name, _) in candidates {
        if seen.insert(name.clone()) {
            out.push(name);
        }
    }

    Ok(out)
}

fn choose_backup_name(db_path: &Path) -> Result<String> {
    let names = list_backup_names(db_path)?;
    if names.is_empty() {
        bail!(
            "no database backups found for {}\nlooked in: {}",
            db_path.display(),
            db_path
                .parent()
                .map_or_else(|| "<unknown>".to_string(), |p| p.display().to_string())
        );
    }

    println!("Available backups (newest first):");
    for (i, name) in names.iter().enumerate() {
        println!("  {}) {}", i + 1, name);
    }
    println!();

    print!(
        "Select a backup to restore [1-{}] (default 1): ",
        names.len()
    );
    io::stdout().flush().context("flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("read selection from stdin")?;
    let input = input.trim();

    if input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("quit") {
        bail!("aborted");
    }

    if input.is_empty() {
        return Ok(names[0].clone());
    }

    // Exact name match wins (useful for numeric timestamps).
    if names.iter().any(|n| n == input) {
        return Ok(input.to_string());
    }

    // Allow "#3" to force index selection.
    let input_for_index = input.strip_prefix('#').unwrap_or(input);
    if let Ok(idx) = input_for_index.parse::<usize>() {
        if (1..=names.len()).contains(&idx) {
            return Ok(names[idx - 1].clone());
        }
    }

    Err(anyhow!(
        "invalid selection: {input} (enter a number 1-{}, a backup name, or 'q' to quit)",
        names.len()
    ))
}

fn backup_path_for(path: &Path, backup_name: &str) -> Result<PathBuf> {
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("path has no file name: {}", path.display()))?
        .to_string_lossy();

    let file_name = format!("{file_name}.{backup_name}.bak");
    Ok(path.with_file_name(file_name))
}

fn restore_db_files(db_path: &Path, backup_name: &str) -> Result<()> {
    let mut restored_any = false;

    for path in db_file_paths(db_path) {
        let backup_path = backup_path_for(&path, backup_name)?;
        if !backup_path.exists() {
            continue;
        }
        if !backup_path.is_file() {
            bail!(
                "backup path exists but is not a file: {}",
                backup_path.display()
            );
        }

        std::fs::copy(&backup_path, &path)
            .with_context(|| format!("copy {} -> {}", backup_path.display(), path.display()))?;

        restored_any = true;
        println!("Restored: {} -> {}", backup_path.display(), path.display());
    }

    if !restored_any {
        bail!("no backup files found for backup {backup_name}");
    }

    Ok(())
}

fn seeds_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("db")
        .join("seeds")
}

fn timestamp_ms() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0));
    now.as_millis().to_string()
}

fn db_file_paths(db_path: &Path) -> [PathBuf; 4] {
    [
        db_path.to_path_buf(),
        with_os_suffix(db_path, "-wal"),
        with_os_suffix(db_path, "-shm"),
        with_os_suffix(db_path, "-journal"),
    ]
}

fn plan_backup_pairs(db_path: &Path, backup_name: &str) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut planned = Vec::<(PathBuf, PathBuf)>::new();
    for path in db_file_paths(db_path) {
        if !path.exists() {
            continue;
        }
        if !path.is_file() {
            bail!("path exists but is not a file: {}", path.display());
        }
        let backup_path = backup_path_for(&path, backup_name)?;
        planned.push((path, backup_path));
    }
    Ok(planned)
}

fn preflight_backup_collisions(
    planned: &[(PathBuf, PathBuf)],
    backup_name: &str,
    overwrite: bool,
) -> Result<()> {
    let mut collisions = Vec::<PathBuf>::new();

    for (_, backup_path) in planned {
        if !backup_path.exists() {
            continue;
        }

        if !overwrite {
            collisions.push(backup_path.clone());
            continue;
        }

        if !backup_path.is_file() {
            bail!(
                "backup destination exists but is not a file: {}",
                backup_path.display()
            );
        }
    }

    if collisions.is_empty() {
        return Ok(());
    }

    let list = collisions
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join("\n  - ");
    bail!(
        "backup name already in use: {backup_name}\nThese backup files already exist:\n  - {list}\n\nPass --backup-overwrite to overwrite them."
    );
}

fn backup_db_files(db_path: &Path, backup_name: &str, overwrite: bool) -> Result<()> {
    validate_backup_name(backup_name)?;

    let planned = plan_backup_pairs(db_path, backup_name)?;

    if planned.is_empty() {
        println!("No existing database found at {}", db_path.display());
        return Ok(());
    }

    preflight_backup_collisions(&planned, backup_name, overwrite)?;

    for (src, dest) in planned {
        std::fs::rename(&src, &dest)
            .with_context(|| format!("rename {} -> {}", src.display(), dest.display()))?;

        println!("Backed up: {} -> {}", src.display(), dest.display());
    }

    Ok(())
}

fn copy_db_files_to_backup(db_path: &Path, backup_name: &str, overwrite: bool) -> Result<()> {
    validate_backup_name(backup_name)?;

    let planned = plan_backup_pairs(db_path, backup_name)?;

    if planned.is_empty() {
        println!("No existing database found at {}", db_path.display());
        return Ok(());
    }

    preflight_backup_collisions(&planned, backup_name, overwrite)?;

    for (src, dest) in planned {
        std::fs::copy(&src, &dest)
            .with_context(|| format!("copy {} -> {}", src.display(), dest.display()))?;
        println!("Backed up: {} -> {}", src.display(), dest.display());
    }

    Ok(())
}

fn with_os_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut s: OsString = path.as_os_str().to_os_string();
    s.push(suffix);
    PathBuf::from(s)
}

fn confirm_default_yes(question: &str) -> Result<bool> {
    print!("{question} [Y/n] ");
    io::stdout().flush().context("flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("read confirmation from stdin")?;
    let input = input.trim().to_ascii_lowercase();

    match input.as_str() {
        "" | "y" | "yes" => Ok(true),
        _ => Ok(false),
    }
}

async fn connect_pool(db_path: &Path) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .context("connect sqlite pool")?;

    Ok(pool)
}

async fn run_seed(pool: &SqlitePool, seed_path: &Path) -> Result<()> {
    let seed_sql = std::fs::read_to_string(seed_path)
        .with_context(|| format!("read seed file {}", seed_path.display()))?;

    if seed_sql.trim().is_empty() {
        println!("Seed file is empty; skipping.");
        return Ok(());
    }

    let mut conn = pool.acquire().await.context("acquire sqlite connection")?;
    sqlx::raw_sql(&seed_sql)
        .execute(&mut *conn)
        .await
        .with_context(|| format!("execute seed SQL from {}", seed_path.display()))?;

    Ok(())
}
