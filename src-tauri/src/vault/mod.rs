pub mod atomic;
pub mod errors;
pub mod model;
pub mod path;

#[cfg(test)]
pub mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use atomic::write_atomic;
pub use errors::VaultError;
pub use model::{VaultDocument, VaultDocumentMetadata};
pub use path::validate_and_resolve_relative_path;

use crate::db::{log_audit_event, NewAuditEvent};

/// Record schema for the SQLite vault documents metadata index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultIndexRecord {
    pub id: String,
    pub relative_path: String,
    pub title: String,
    pub size_bytes: i64,
    pub content_hash: String,
    pub schema_version: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Core Vault handle managing local Markdown document persistence, safe path validation,
/// atomic filesystem updates, SQLite metadata indexing, and cryptographic audit logging.
pub struct Vault {
    vault_root: PathBuf,
}

impl Vault {
    /// Initialize a Vault handle with the configured vault root path.
    pub fn new(vault_root: impl AsRef<Path>) -> Result<Self, VaultError> {
        let root = vault_root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self { vault_root: root })
    }

    /// Access configured vault root directory path.
    pub fn root_path(&self) -> &Path {
        &self.vault_root
    }

    /// Save (create or update) a document in the vault.
    /// Performs path validation, atomic filesystem write, SHA-256 calculation,
    /// SQLite metadata index upsert, and audit event logging (excluding raw Markdown content).
    pub fn save_document(
        &self,
        conn: &mut Connection,
        relative_path: &str,
        doc: &mut VaultDocument,
    ) -> Result<VaultIndexRecord, VaultError> {
        let full_path = validate_and_resolve_relative_path(&self.vault_root, relative_path)?;

        let existing_res = self.get_document_metadata(conn, relative_path);

        let existing = match existing_res {
            Ok(record) => Some(record),
            Err(VaultError::NotFound(_)) => None,
            Err(err) => return Err(err),
        };

        if let Some(ref prior_record) = existing {
            if doc.metadata.id != prior_record.id {
                return Err(VaultError::IdMismatch {
                    path: relative_path.to_string(),
                    expected: prior_record.id.clone(),
                    found: doc.metadata.id.clone(),
                });
            }
        }

        let serialized = doc.to_markdown_string()?;
        let size_bytes = serialized.len() as i64;

        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let content_hash = hex::encode(hasher.finalize());

        // Atomically write serialized Markdown document to disk
        write_atomic(&full_path, serialized.as_bytes())?;

        let index_record = VaultIndexRecord {
            id: doc.metadata.id.clone(),
            relative_path: relative_path.to_string(),
            title: doc.metadata.title.clone(),
            size_bytes,
            content_hash,
            schema_version: doc.metadata.schema_version as i32,
            created_at: doc.metadata.created_at.clone(),
            updated_at: doc.metadata.updated_at.clone(),
        };

        let json_after = serde_json::to_string(&index_record)?;

        match existing {
            Some(prior_record) => {
                let json_before = serde_json::to_string(&prior_record)?;

                conn.execute(
                    "UPDATE vault_documents SET
                        id = ?1,
                        title = ?2,
                        size_bytes = ?3,
                        content_hash = ?4,
                        schema_version = ?5,
                        updated_at = ?6
                     WHERE relative_path = ?7;",
                    rusqlite::params![
                        index_record.id,
                        index_record.title,
                        index_record.size_bytes,
                        index_record.content_hash,
                        index_record.schema_version,
                        index_record.updated_at,
                        relative_path,
                    ],
                )?;

                log_audit_event(
                    conn,
                    NewAuditEvent {
                        user_id: Some("system".to_string()),
                        action_type: "VAULT_DOCUMENT_UPDATE".to_string(),
                        entity_name: "vault_document".to_string(),
                        entity_id: Some(index_record.id.clone()),
                        before_state: Some(json_before),
                        after_state: Some(json_after),
                        client_info: Some("desktop_app".to_string()),
                    },
                )?;
            }
            None => {
                conn.execute(
                    "INSERT INTO vault_documents (
                        id, relative_path, title, size_bytes, content_hash, schema_version, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
                    rusqlite::params![
                        index_record.id,
                        index_record.relative_path,
                        index_record.title,
                        index_record.size_bytes,
                        index_record.content_hash,
                        index_record.schema_version,
                        index_record.created_at,
                        index_record.updated_at,
                    ],
                )?;

                log_audit_event(
                    conn,
                    NewAuditEvent {
                        user_id: Some("system".to_string()),
                        action_type: "VAULT_DOCUMENT_CREATE".to_string(),
                        entity_name: "vault_document".to_string(),
                        entity_id: Some(index_record.id.clone()),
                        before_state: None,
                        after_state: Some(json_after),
                        client_info: Some("desktop_app".to_string()),
                    },
                )?;
            }
        }

        Ok(index_record)
    }

    /// Read and parse a Markdown document from the vault by relative path.
    pub fn read_document(&self, relative_path: &str) -> Result<VaultDocument, VaultError> {
        let full_path = validate_and_resolve_relative_path(&self.vault_root, relative_path)?;

        if !full_path.exists() {
            return Err(VaultError::NotFound(relative_path.to_string()));
        }

        let raw_content = fs::read_to_string(&full_path)?;
        VaultDocument::parse(&raw_content, relative_path)
    }

    /// Delete a Markdown document from the vault and remove its entry from SQLite metadata index.
    pub fn delete_document(
        &self,
        conn: &mut Connection,
        relative_path: &str,
    ) -> Result<(), VaultError> {
        let full_path = validate_and_resolve_relative_path(&self.vault_root, relative_path)?;

        let prior_record = self.get_document_metadata(conn, relative_path)?;
        let json_before = serde_json::to_string(&prior_record)?;

        if full_path.exists() {
            fs::remove_file(&full_path)?;
        }

        conn.execute(
            "DELETE FROM vault_documents WHERE relative_path = ?1;",
            rusqlite::params![relative_path],
        )?;

        log_audit_event(
            conn,
            NewAuditEvent {
                user_id: Some("system".to_string()),
                action_type: "VAULT_DOCUMENT_DELETE".to_string(),
                entity_name: "vault_document".to_string(),
                entity_id: Some(prior_record.id),
                before_state: Some(json_before),
                after_state: None,
                client_info: Some("desktop_app".to_string()),
            },
        )?;

        Ok(())
    }

    /// Query metadata index record for a relative path.
    pub fn get_document_metadata(
        &self,
        conn: &Connection,
        relative_path: &str,
    ) -> Result<VaultIndexRecord, VaultError> {
        let mut stmt = conn.prepare(
            "SELECT id, relative_path, title, size_bytes, content_hash, schema_version, created_at, updated_at
             FROM vault_documents WHERE relative_path = ?1;",
        )?;

        stmt.query_row(rusqlite::params![relative_path], |row| {
            Ok(VaultIndexRecord {
                id: row.get(0)?,
                relative_path: row.get(1)?,
                title: row.get(2)?,
                size_bytes: row.get(3)?,
                content_hash: row.get(4)?,
                schema_version: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => VaultError::NotFound(relative_path.to_string()),
            other => VaultError::Database(other.into()),
        })
    }

    /// List all indexed document metadata records from SQLite.
    pub fn list_documents(&self, conn: &Connection) -> Result<Vec<VaultIndexRecord>, VaultError> {
        let mut stmt = conn.prepare(
            "SELECT id, relative_path, title, size_bytes, content_hash, schema_version, created_at, updated_at
             FROM vault_documents ORDER BY relative_path ASC;",
        )?;

        let records = stmt
            .query_map([], |row| {
                Ok(VaultIndexRecord {
                    id: row.get(0)?,
                    relative_path: row.get(1)?,
                    title: row.get(2)?,
                    size_bytes: row.get(3)?,
                    content_hash: row.get(4)?,
                    schema_version: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(records)
    }
}
