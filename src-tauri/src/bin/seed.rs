use anyhow::{anyhow, bail, Context, Result};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DB_FILENAME: &str = "worth.sqlite";

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);

    let seed_arg = match args.next() {
        Some(arg) if arg == "-h" || arg == "--help" => {
            print_usage_and_seeds();
            return Ok(());
        }
        Some(arg) if arg == "--list" => {
            print_available_seeds();
            return Ok(());
        }
        Some(arg) if arg == "--clear" => {
            clear_db()?;
            return Ok(());
        }
        Some(arg) if arg == "--clean" => {
            let mut assume_yes = false;
            for extra in args {
                match extra.as_str() {
                    "-y" => assume_yes = true,
                    other => bail!("unknown argument for --clean: {other}"),
                }
            }

            clean_db_backups(assume_yes)?;
            return Ok(());
        }
        Some(arg) => arg,
        None => {
            print_usage_and_seeds();
            bail!("missing seed name (or path)")
        }
    };

    let seed_path = resolve_seed_path(&seed_arg)?;
    let db_path = resolve_db_path()?;

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create dir {}", parent.display()))?;
    }

    let ts = timestamp_ms();
    backup_db_files(&db_path, &ts)?;

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

fn print_usage_and_seeds() {
    eprintln!(
        r"Usage:
  cargo run --manifest-path src-tauri/Cargo.toml --bin seed -- <seed-name|path>
  cargo run --manifest-path src-tauri/Cargo.toml --bin seed -- --clear
  cargo run --manifest-path src-tauri/Cargo.toml --bin seed -- --clean [-y]

Examples:
  bun run db:seed dev
  bun run db:seed path/to/file.sql
  bun run db:seed --list
  bun run db:clear
  bun run db:clean
  bun run db:clean --y
"
    );
    print_available_seeds();
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

fn clear_db() -> Result<()> {
    let db_path = resolve_db_path()?;
    let ts = timestamp_ms();
    backup_db_files(&db_path, &ts)?;
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
        print!("Delete these backup files? [Y/n] ");
        io::stdout().flush().context("flush stdout")?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("read confirmation from stdin")?;
        let input = input.trim().to_ascii_lowercase();
        if !input.is_empty() && input != "y" {
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

fn backup_db_files(db_path: &Path, timestamp: &str) -> Result<()> {
    let mut backed_up_any = false;

    for path in [
        db_path.to_path_buf(),
        with_os_suffix(db_path, "-wal"),
        with_os_suffix(db_path, "-shm"),
        with_os_suffix(db_path, "-journal"),
    ] {
        if let Some(backup_path) = backup_one(&path, timestamp)? {
            backed_up_any = true;
            println!("Backed up: {} -> {}", path.display(), backup_path.display());
        }
    }

    if !backed_up_any {
        println!("No existing database found at {}", db_path.display());
    }

    Ok(())
}

fn backup_one(path: &Path, timestamp: &str) -> Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }

    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("path has no file name: {}", path.display()))?
        .to_string_lossy();

    let backup_name = format!("{file_name}.{timestamp}.bak");
    let backup_path = path.with_file_name(backup_name);

    std::fs::rename(path, &backup_path)
        .with_context(|| format!("rename {} -> {}", path.display(), backup_path.display()))?;

    Ok(Some(backup_path))
}

fn with_os_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut s: OsString = path.as_os_str().to_os_string();
    s.push(suffix);
    PathBuf::from(s)
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
