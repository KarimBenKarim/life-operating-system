use crate::db::crypto::DerivedKey;
use crate::db::errors::{DbError, Result};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Resolves the default database path in the OS app data directory.
pub fn get_default_db_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let app_dir = app_handle.path().app_data_dir().map_err(|e| {
        DbError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            e.to_string(),
        ))
    })?;

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }

    Ok(app_dir.join("lifeos.db"))
}

/// Database Connection Manager
pub struct DbManager {
    connection: Connection,
}

impl DbManager {
    /// Opens an unencrypted database connection at the specified path (or in-memory if path is `:memory:`).
    pub fn open_unencrypted<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::configure_connection(&conn)?;
        Ok(DbManager { connection: conn })
    }

    /// Opens an encrypted SQLCipher database connection using a derived key.
    pub fn open_encrypted<P: AsRef<Path>>(path: P, derived_key: &DerivedKey) -> Result<Self> {
        let conn = Connection::open(path)?;
        let pragma_sql = format!("PRAGMA key = {};", derived_key.to_pragma_hex());
        conn.execute_batch(&pragma_sql)?;
        Self::configure_connection(&conn)?;
        Ok(DbManager { connection: conn })
    }

    /// Configures standard database PRAGMAs (WAL mode, foreign keys, synchronous=NORMAL).
    fn configure_connection(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;
        Ok(())
    }

    /// Provides access to the underlying Rusqlite connection.
    pub fn conn(&self) -> &Connection {
        &self.connection
    }
}
