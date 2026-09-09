pub mod audit;
pub mod document;
pub mod errors;
pub mod index;
pub mod path;
pub mod storage;

#[cfg(test)]
pub mod tests;

pub use document::{Frontmatter, VaultDocument};
pub use errors::VaultError;

use std::fs;
use std::path::Path;

use rusqlite::Connection;

pub use index::{compute_content_hash, VaultIndex};
pub use path::{PathValidator, RelativePath, ResolvedPath, VaultRoot};
pub use storage::VaultStorage;

/// Master interface for the Life OS Local Markdown Vault.
/// Coordinates path validation, atomic filesystem operations, SQLite metadata indexing, and audit logging.
pub struct MarkdownVault {
    validator: PathValidator,
}

impl MarkdownVault {
    pub fn new(root_path: impl AsRef<Path>) -> Result<Self, VaultError> {
        let root = VaultRoot::new(root_path)?;
        let validator = PathValidator::new(root);
        Ok(Self { validator })
    }

    pub fn validator(&self) -> &PathValidator {
        &self.validator
    }

    pub fn create_document(
        &self,
        db_conn: Option<&mut Connection>,
        rel_path_str: &str,
        doc: &VaultDocument,
    ) -> Result<ResolvedPath, VaultError> {
        let rel_path = RelativePath::parse(rel_path_str)?;
        let resolved = VaultStorage::create_document(&self.validator, &rel_path, doc)?;

        let content_text = doc.to_markdown()?;
        let content_hash = compute_content_hash(&content_text);
        let file_size = fs::metadata(resolved.as_path())
            .map(|m| m.len())
            .unwrap_or(0);

        if let Some(conn) = db_conn {
            VaultIndex::sync_document_metadata(
                conn,
                &rel_path.to_string_lossy(),
                doc,
                file_size,
                &content_hash,
            )?;

            audit::VaultAudit::log_create(
                conn,
                doc,
                &rel_path.to_string_lossy(),
                file_size,
                &content_hash,
            )?;
        }

        Ok(resolved)
    }

    pub fn read_document(&self, rel_path_str: &str) -> Result<VaultDocument, VaultError> {
        let rel_path = RelativePath::parse(rel_path_str)?;
        VaultStorage::read_document(&self.validator, &rel_path)
    }

    pub fn update_document(
        &self,
        db_conn: Option<&mut Connection>,
        rel_path_str: &str,
        doc: &VaultDocument,
    ) -> Result<ResolvedPath, VaultError> {
        let rel_path = RelativePath::parse(rel_path_str)?;

        let old_hash =
            if let Ok(existing_doc) = VaultStorage::read_document(&self.validator, &rel_path) {
                let text = existing_doc.to_markdown()?;
                compute_content_hash(&text)
            } else {
                String::new()
            };

        let resolved = VaultStorage::update_document(&self.validator, &rel_path, doc)?;

        let new_text = doc.to_markdown()?;
        let new_hash = compute_content_hash(&new_text);
        let file_size = fs::metadata(resolved.as_path())
            .map(|m| m.len())
            .unwrap_or(0);

        if let Some(conn) = db_conn {
            VaultIndex::sync_document_metadata(
                conn,
                &rel_path.to_string_lossy(),
                doc,
                file_size,
                &new_hash,
            )?;

            audit::VaultAudit::log_update(
                conn,
                doc,
                &rel_path.to_string_lossy(),
                file_size,
                &old_hash,
                &new_hash,
            )?;
        }

        Ok(resolved)
    }

    pub fn delete_document(
        &self,
        db_conn: Option<&mut Connection>,
        rel_path_str: &str,
    ) -> Result<(), VaultError> {
        let rel_path = RelativePath::parse(rel_path_str)?;

        let (doc_id, old_hash) =
            if let Ok(existing_doc) = VaultStorage::read_document(&self.validator, &rel_path) {
                let text = existing_doc.to_markdown()?;
                (
                    existing_doc.frontmatter.id.clone(),
                    compute_content_hash(&text),
                )
            } else {
                ("unknown".to_string(), String::new())
            };

        VaultStorage::delete_document(&self.validator, &rel_path)?;

        if let Some(conn) = db_conn {
            VaultIndex::delete_document_metadata(conn, &rel_path.to_string_lossy())?;

            audit::VaultAudit::log_delete(
                conn,
                &doc_id,
                &rel_path.to_string_lossy(),
                Some(&old_hash),
            )?;
        }

        Ok(())
    }

    pub fn exists(&self, rel_path_str: &str) -> bool {
        if let Ok(rel_path) = RelativePath::parse(rel_path_str) {
            VaultStorage::exists(&self.validator, &rel_path)
        } else {
            false
        }
    }

    pub fn list_documents(&self) -> Result<Vec<RelativePath>, VaultError> {
        VaultStorage::list_documents(&self.validator)
    }
}
