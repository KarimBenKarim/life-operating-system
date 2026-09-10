use thiserror::Error;

use crate::db::DatabaseError;

/// Domain error types for vault storage and persistence operations.
#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Invalid relative path '{0}': {1}")]
    InvalidPath(String, String),

    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),

    #[error("Symlink or reparse escape attempt detected: {0}")]
    SymlinkEscape(String),

    #[error("Malformed frontmatter in document '{0}': {1}")]
    MalformedFrontmatter(String, String),

    #[error("Document not found: {0}")]
    NotFound(String),

    #[error("Document already exists: {0}")]
    AlreadyExists(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("YAML serialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}
