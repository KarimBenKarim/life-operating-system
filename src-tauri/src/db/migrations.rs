use chrono::Utc;
use rusqlite::Connection;

use crate::db::errors::DatabaseError;

/// Individual database migration definition.
pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

/// Static, ordered list of all database migrations.
pub static MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "V1__init_schema",
    sql: "
        CREATE TABLE IF NOT EXISTS system_audit_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT,
            action_type TEXT NOT NULL,
            entity_name TEXT NOT NULL,
            entity_id TEXT,
            before_state TEXT,
            after_state TEXT,
            client_info TEXT,
            created_at TEXT NOT NULL,
            previous_hash TEXT NOT NULL,
            hash TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_audit_action_type ON system_audit_logs(action_type);
        CREATE INDEX IF NOT EXISTS idx_audit_created_at ON system_audit_logs(created_at);
        ",
}];

/// Initialize the `_migrations` tracking table if missing.
fn ensure_migrations_table(conn: &Connection) -> Result<(), DatabaseError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );",
        [],
    )?;
    Ok(())
}

/// Run all pending versioned migrations inside a single transaction.
/// Verifies that migration versions are contiguous without gaps.
pub fn run_migrations(conn: &mut Connection) -> Result<usize, DatabaseError> {
    ensure_migrations_table(conn)?;

    let tx = conn.transaction()?;

    let mut stmt = tx.prepare("SELECT version FROM _migrations ORDER BY version ASC;")?;
    let applied_versions: Vec<i32> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<i32>, _>>()?;
    drop(stmt);

    let max_applied = applied_versions.into_iter().max().unwrap_or(0);

    let pending: Vec<&Migration> = MIGRATIONS
        .iter()
        .filter(|m| m.version > max_applied)
        .collect();

    if pending.is_empty() {
        tx.commit()?;
        return Ok(0);
    }

    // Verify contiguous migration version sequence without gaps
    for (idx, m) in pending.iter().enumerate() {
        let expected_version = max_applied + 1 + (idx as i32);
        if m.version != expected_version {
            return Err(DatabaseError::MigrationError(format!(
                "Migration sequence gap detected: expected version {expected_version}, but found version {}",
                m.version
            )));
        }
    }

    let count = pending.len();

    for migration in pending {
        tx.execute_batch(migration.sql).map_err(|e| {
            DatabaseError::MigrationError(format!(
                "Failed to execute migration {} ({}): {e}",
                migration.version, migration.name
            ))
        })?;

        let applied_at = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO _migrations (version, name, applied_at) VALUES (?1, ?2, ?3);",
            rusqlite::params![migration.version, migration.name, applied_at],
        )?;
    }

    tx.commit()?;
    Ok(count)
}

/// Helper function to execute a custom migration list for testing gap detection.
#[cfg(test)]
pub fn run_migrations_custom(
    conn: &mut Connection,
    migrations: &[Migration],
) -> Result<usize, DatabaseError> {
    ensure_migrations_table(conn)?;

    let tx = conn.transaction()?;

    let mut stmt = tx.prepare("SELECT version FROM _migrations ORDER BY version ASC;")?;
    let applied_versions: Vec<i32> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<i32>, _>>()?;
    drop(stmt);

    let max_applied = applied_versions.into_iter().max().unwrap_or(0);

    let pending: Vec<&Migration> = migrations
        .iter()
        .filter(|m| m.version > max_applied)
        .collect();

    if pending.is_empty() {
        tx.commit()?;
        return Ok(0);
    }

    for (idx, m) in pending.iter().enumerate() {
        let expected_version = max_applied + 1 + (idx as i32);
        if m.version != expected_version {
            return Err(DatabaseError::MigrationError(format!(
                "Migration sequence gap detected: expected version {expected_version}, but found version {}",
                m.version
            )));
        }
    }

    let count = pending.len();

    for migration in pending {
        tx.execute_batch(migration.sql).map_err(|e| {
            DatabaseError::MigrationError(format!(
                "Failed to execute migration {} ({}): {e}",
                migration.version, migration.name
            ))
        })?;

        let applied_at = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO _migrations (version, name, applied_at) VALUES (?1, ?2, ?3);",
            rusqlite::params![migration.version, migration.name, applied_at],
        )?;
    }

    tx.commit()?;
    Ok(count)
}
