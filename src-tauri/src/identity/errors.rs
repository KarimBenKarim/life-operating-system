use thiserror::Error;

use crate::db::DatabaseError;

/// Domain error types for user identity and profile operations.
#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("User profile not found: {0}")]
    NotFound(String),

    #[error("User profile already exists: {0}")]
    AlreadyExists(String),

    #[error("User ID is immutable: cannot change ID from '{expected}' to '{found}'")]
    IdMismatch { expected: String, found: String },

    #[error("Invalid profile input: {0}")]
    InvalidInput(String),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}
