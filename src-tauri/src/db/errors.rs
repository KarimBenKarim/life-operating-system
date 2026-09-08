use thiserror::Error;

/// Core domain error types for database operations.
/// All error messages are sanitized to prevent leaking secret key material or passcodes.
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Invalid master passcode or database key")]
    InvalidPasscode,

    #[error("Missing KDF metadata file required to derive database key")]
    MissingKdfMetadata,

    #[error("Invalid KDF metadata: {0}")]
    InvalidKdfMetadata(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Migration failed: {0}")]
    MigrationError(String),

    #[error("Audit log tampering detected at record ID {record_id}: {reason}")]
    AuditTampered { record_id: i64, reason: String },

    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),
}
