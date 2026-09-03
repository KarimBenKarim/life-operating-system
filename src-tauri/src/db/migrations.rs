use crate::db::errors::{DbError, Result};
use rusqlite::Connection;

pub struct Migration {
    pub version: i32,
    pub description: &'static str,
    pub sql: &'static str,
}

/// Statistically defined schema migrations.
pub fn get_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "Initial foundational schema (users, system_audit_logs)",
        sql: "
                CREATE TABLE IF NOT EXISTS users (
                    id TEXT PRIMARY KEY,
                    email TEXT UNIQUE NOT NULL,
                    password_hash TEXT NOT NULL,
                    first_name TEXT NOT NULL,
                    last_name TEXT,
                    timezone TEXT DEFAULT 'UTC' NOT NULL,
                    created_at TEXT DEFAULT (datetime('now')) NOT NULL,
                    updated_at TEXT DEFAULT (datetime('now')) NOT NULL
                );

                CREATE TABLE IF NOT EXISTS system_audit_logs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id TEXT,
                    action_type TEXT NOT NULL,
                    entity_name TEXT NOT NULL,
                    entity_id TEXT,
                    before_state TEXT,
                    after_state TEXT,
                    client_ip TEXT,
                    previous_log_hash TEXT NOT NULL,
                    signature TEXT NOT NULL,
                    logged_at TEXT DEFAULT (datetime('now')) NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
                );

                CREATE INDEX IF NOT EXISTS idx_system_audit_logs_user_action
                ON system_audit_logs(user_id, action_type);
            ",
    }]
}

/// Runs all pending migrations deterministically in transaction blocks.
pub fn run_migrations(conn: &Connection) -> Result<i32> {
    // 1. Ensure tracking table exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT DEFAULT (datetime('now')) NOT NULL
        );",
    )?;

    // 2. Fetch applied versions
    let mut stmt = conn.prepare("SELECT version FROM schema_migrations ORDER BY version ASC;")?;
    let applied_versions: Vec<i32> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<std::result::Result<Vec<i32>, _>>()?;

    let migrations = get_migrations();
    let mut newly_applied = 0;

    // 3. Execute pending migrations in transactions
    for migration in migrations {
        if !applied_versions.contains(&migration.version) {
            let tx = conn.unchecked_transaction()?;

            tx.execute_batch(migration.sql).map_err(|e| {
                DbError::Migration(format!("Migration {} failed: {e}", migration.version))
            })?;

            tx.execute(
                "INSERT INTO schema_migrations (version) VALUES (?1);",
                [migration.version],
            )
            .map_err(|e| {
                DbError::Migration(format!(
                    "Failed to record migration version {}: {e}",
                    migration.version
                ))
            })?;

            tx.commit()?;
            newly_applied += 1;
        }
    }

    Ok(newly_applied)
}

/// Gets the highest applied schema version.
pub fn get_current_schema_version(conn: &Connection) -> Result<i32> {
    let has_table: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_migrations';",
            [],
            |row| row.get(0),
        )
        .map(|count: i32| count > 0)
        .unwrap_or(false);

    if !has_table {
        return Ok(0);
    }

    let mut stmt = conn.prepare("SELECT COALESCE(MAX(version), 0) FROM schema_migrations;")?;
    let version: i32 = stmt.query_row([], |row| row.get(0))?;
    Ok(version)
}
