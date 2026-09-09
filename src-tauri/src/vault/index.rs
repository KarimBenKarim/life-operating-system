use rusqlite::Connection;
use sha2::{Digest, Sha256};

use crate::vault::document::VaultDocument;
use crate::vault::errors::VaultError;

/// Calculate SHA-256 hash of Markdown document content for tracking state changes.
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Metadata index synchronization helper connecting the Vault to SQLite.
pub struct VaultIndex;

impl VaultIndex {
    /// Upsert document metadata into SQLite vault_documents table.
    pub fn sync_document_metadata(
        conn: &Connection,
        rel_path: &str,
        doc: &VaultDocument,
        file_size_bytes: u64,
        content_hash: &str,
    ) -> Result<(), VaultError> {
        conn.execute(
            "INSERT INTO vault_documents (
                id, relative_path, title, file_size_bytes, content_hash, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(relative_path) DO UPDATE SET
                id = excluded.id,
                title = excluded.title,
                file_size_bytes = excluded.file_size_bytes,
                content_hash = excluded.content_hash,
                updated_at = excluded.updated_at;",
            rusqlite::params![
                doc.frontmatter.id,
                rel_path,
                doc.frontmatter.title,
                file_size_bytes as i64,
                content_hash,
                doc.frontmatter.created_at,
                doc.frontmatter.updated_at,
            ],
        )
        .map_err(crate::db::DatabaseError::Sqlite)?;

        Ok(())
    }

    /// Delete document metadata entry from SQLite vault_documents table upon document removal.
    pub fn delete_document_metadata(conn: &Connection, rel_path: &str) -> Result<(), VaultError> {
        conn.execute(
            "DELETE FROM vault_documents WHERE relative_path = ?1;",
            rusqlite::params![rel_path],
        )
        .map_err(crate::db::DatabaseError::Sqlite)?;

        Ok(())
    }
}
