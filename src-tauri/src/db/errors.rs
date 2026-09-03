use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(#[from] rusqlite::Error),

    #[error("Key derivation error: {0}")]
    KeyDerivation(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Audit log error: {0}")]
    AuditLog(String),

    #[error("Audit log tampering detected: {0}")]
    AuditTampering(String),

    #[error("IO/Path error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, DbError>;
