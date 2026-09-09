use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::db::audit::verify_audit_chain;
use crate::db::crypto::{
    derive_key, format_pragma_key, generate_salt, ARGON2_M_COST, ARGON2_P_COST, ARGON2_T_COST,
    SALT_BYTES,
};
use crate::db::errors::DatabaseError;
use crate::db::migrations::run_migrations;

/// Persistent KDF salt and parameters metadata stored alongside the database file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KdfMetadata {
    pub version: u32,
    pub kdf_algorithm: String,
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
    pub salt_hex: String,
}

impl KdfMetadata {
    pub fn new(salt: &[u8; SALT_BYTES]) -> Self {
        Self {
            version: 1,
            kdf_algorithm: "Argon2id".to_string(),
            m_cost: ARGON2_M_COST,
            t_cost: ARGON2_T_COST,
            p_cost: ARGON2_P_COST,
            salt_hex: hex::encode(salt),
        }
    }

    pub fn salt_bytes(&self) -> Result<[u8; SALT_BYTES], DatabaseError> {
        let bytes = hex::decode(&self.salt_hex).map_err(|e| {
            DatabaseError::InvalidKdfMetadata(format!("Invalid salt hex in metadata: {e}"))
        })?;

        if bytes.len() != SALT_BYTES {
            return Err(DatabaseError::InvalidKdfMetadata(format!(
                "Invalid salt length: expected {} bytes, got {}",
                SALT_BYTES,
                bytes.len()
            )));
        }

        let mut salt = [0u8; SALT_BYTES];
        salt.copy_from_slice(&bytes);
        Ok(salt)
    }
}

/// Helper to get the path of the KDF metadata file corresponding to a database path.
pub fn get_kdf_metadata_path(db_path: &Path) -> PathBuf {
    let mut path = db_path.to_path_buf();
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "lifeos.db".to_string());
    path.set_file_name(format!("{file_name}.kdf"));
    path
}

/// Helper to read existing KDF metadata with retries if another process is mid-write.
fn read_kdf_metadata_with_retry(
    kdf_path: &Path,
) -> Result<(KdfMetadata, [u8; SALT_BYTES]), DatabaseError> {
    for _ in 0..50 {
        if let Ok(content) = fs::read_to_string(kdf_path) {
            if let Ok(metadata) = serde_json::from_str::<KdfMetadata>(&content) {
                if let Ok(salt) = metadata.salt_bytes() {
                    return Ok((metadata, salt));
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let content = fs::read_to_string(kdf_path).map_err(DatabaseError::Io)?;
    let metadata: KdfMetadata = serde_json::from_str(&content).map_err(|e| {
        DatabaseError::InvalidKdfMetadata(format!("Failed to parse KDF metadata JSON: {e}"))
    })?;
    let salt = metadata.salt_bytes()?;
    Ok((metadata, salt))
}

/// Load existing KDF metadata or generate and persist new metadata atomically (O_CREAT | O_EXCL) if missing.
/// Ensures first-run initialization is completely race-safe across concurrent processes/threads.
pub fn load_or_create_kdf_metadata(
    db_path: &Path,
) -> Result<(KdfMetadata, [u8; SALT_BYTES]), DatabaseError> {
    let kdf_path = get_kdf_metadata_path(db_path);

    if let Some(parent) = kdf_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let salt = generate_salt();
    let metadata = KdfMetadata::new(&salt);
    let content = serde_json::to_string_pretty(&metadata).map_err(|e| {
        DatabaseError::InvalidKdfMetadata(format!("Failed to serialize KDF metadata: {e}"))
    })?;

    let create_result = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&kdf_path);

    match create_result {
        Ok(mut file) => {
            file.write_all(content.as_bytes())?;
            Ok((metadata, salt))
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            read_kdf_metadata_with_retry(&kdf_path)
        }
        Err(e) => Err(DatabaseError::Io(e)),
    }
}

/// Open an encrypted SQLite database using SQLCipher and an Argon2id-derived key.
pub fn open_database(db_path: &Path, passcode: &str) -> Result<Connection, DatabaseError> {
    if passcode.is_empty() {
        return Err(DatabaseError::InvalidPasscode);
    }

    let (_metadata, salt) = load_or_create_kdf_metadata(db_path)?;
    let key = derive_key(passcode, &salt)?;

    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(db_path)?;

    // Configure SQLCipher key using raw key syntax x'HEX' to avoid double-KDF.
    // Wrap PRAGMA statement in Zeroizing<String> to sanitize heap memory on drop.
    let pragma_key = format_pragma_key(&key);
    if conn.execute_batch(&pragma_key).is_err() {
        return Err(DatabaseError::InvalidPasscode);
    }

    // Configure standard database pragmas
    if conn
        .execute_batch(
            "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA busy_timeout = 5000;
        ",
        )
        .is_err()
    {
        return Err(DatabaseError::InvalidPasscode);
    }

    // Validate key & database decryption by querying sqlite_master
    let test_query: Result<i64, rusqlite::Error> =
        conn.query_row("SELECT count(*) FROM sqlite_master;", [], |r| r.get(0));

    match test_query {
        Ok(_) => Ok(conn),
        Err(_e) => Err(DatabaseError::InvalidPasscode),
    }
}

/// Initialize the database connection, execute pending migrations, and run mandatory startup
/// verification of the audit log hash chain. Returns the ready Connection if valid.
///
/// Note: Full system lockdown and local vault directory freezing (as described in ADR 0025)
/// is deferred as a system-level runtime orchestration follow-up.
pub fn initialize_and_verify_database(
    db_path: &Path,
    passcode: &str,
) -> Result<Connection, DatabaseError> {
    let mut conn = open_database(db_path, passcode)?;
    run_migrations(&mut conn)?;
    verify_audit_chain(&conn)?;
    Ok(conn)
}
