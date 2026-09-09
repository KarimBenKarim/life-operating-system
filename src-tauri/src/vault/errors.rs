use thiserror::Error;

/// Core domain error types for vault filesystem and Markdown operations.
#[derive(Error, Debug)]
pub enum VaultError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] crate::db::DatabaseError),

    #[error("Path traversal detected: {0}")]
    PathTraversal(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Invalid frontmatter format: {0}")]
    InvalidFrontmatter(String),

    #[error("Document not found at path: {0}")]
    DocumentNotFound(String),

    #[error("Document already exists at path: {0}")]
    DocumentAlreadyExists(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
