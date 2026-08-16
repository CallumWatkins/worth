use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions};
use std::{path::Path, process::Command};

static MIGRATOR: Migrator = sqlx::migrate!("./db/migrations");
const MIGRATIONS_PATH: &str = "src-tauri/db/migrations";

async fn in_memory_pool() -> sqlx::SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .expect("connect to in-memory SQLite database")
}

fn git_output(arguments: &[&str]) -> Vec<u8> {
    let output = Command::new("git")
        .args(arguments)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {} failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn stable_version(tag: &str) -> Option<(u64, u64, u64)> {
    let mut components = tag.strip_prefix('v')?.split('.');
    let version = (
        components.next()?.parse().ok()?,
        components.next()?.parse().ok()?,
        components.next()?.parse().ok()?,
    );
    components.next().is_none().then_some(version)
}

fn latest_stable_tag() -> Option<String> {
    String::from_utf8(git_output(&["tag", "--list", "v*"]))
        .expect("Git tags are UTF-8")
        .lines()
        .filter_map(|tag| stable_version(tag).map(|version| (version, tag.to_string())))
        .max_by_key(|(version, _)| *version)
        .map(|(_, tag)| tag)
}

#[tokio::test]
async fn current_migrations_apply_to_an_empty_database_and_are_idempotent() {
    let pool = in_memory_pool().await;

    MIGRATOR
        .run(&pool)
        .await
        .expect("apply current migrations to an empty database");
    MIGRATOR
        .run(&pool)
        .await
        .expect("reapply current migrations without changing the database");

    let applied_migration_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = TRUE")
            .fetch_one(&pool)
            .await
            .expect("count applied migrations");
    assert_eq!(applied_migration_count, MIGRATOR.iter().count() as i64);
}

#[tokio::test]
async fn current_migrations_upgrade_the_latest_release_database() {
    let Some(release_tag) = latest_stable_tag() else {
        eprintln!("No stable release tag exists; skipping the released-database upgrade check.");
        return;
    };

    let repository_root = String::from_utf8(git_output(&["rev-parse", "--show-toplevel"]))
        .expect("repository path is UTF-8");
    let repository_root = repository_root.trim();
    let released_migrations = tempfile::tempdir().expect("create released migrations directory");
    let tree = git_output(&[
        "-C",
        repository_root,
        "ls-tree",
        "-r",
        "--name-only",
        "-z",
        &release_tag,
        "--",
        MIGRATIONS_PATH,
    ]);
    let migration_files = tree
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| String::from_utf8(path.to_vec()).expect("migration path is UTF-8"))
        .filter(|path| path.ends_with(".sql"))
        .collect::<Vec<_>>();
    assert!(
        !migration_files.is_empty(),
        "{release_tag} contains no migration files"
    );

    for file_path in migration_files {
        let relative_path = file_path
            .strip_prefix(&format!("{MIGRATIONS_PATH}/"))
            .expect("migration is inside the migrations directory");
        let destination = released_migrations.path().join(relative_path);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).expect("create released migration subdirectory");
        }
        std::fs::write(
            destination,
            git_output(&[
                "-C",
                repository_root,
                "show",
                &format!("{release_tag}:{file_path}"),
            ]),
        )
        .expect("write released migration");
    }

    let released_migrator = Migrator::new(Path::new(released_migrations.path()))
        .await
        .expect("load released migrations");
    let pool = in_memory_pool().await;

    released_migrator
        .run(&pool)
        .await
        .expect("build a database using the latest released migrations");
    MIGRATOR
        .run(&pool)
        .await
        .expect("upgrade the latest release database using current migrations");
}
