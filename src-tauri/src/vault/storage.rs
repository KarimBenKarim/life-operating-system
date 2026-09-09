use std::fs;
use std::io::Write;
use uuid::Uuid;

use crate::vault::document::VaultDocument;
use crate::vault::errors::VaultError;
use crate::vault::path::{PathValidator, RelativePath, ResolvedPath};

/// Atomic filesystem writer for vault Markdown documents.
pub struct VaultStorage;

impl VaultStorage {
    /// Create a new Markdown document on disk using atomic temporary file replacement.
    pub fn create_document(
        validator: &PathValidator,
        rel_path: &RelativePath,
        doc: &VaultDocument,
    ) -> Result<ResolvedPath, VaultError> {
        let resolved = validator.resolve(rel_path)?;

        if resolved.as_path().exists() {
            return Err(VaultError::DocumentAlreadyExists(
                rel_path.to_string_lossy(),
            ));
        }

        Self::atomic_write(&resolved, doc)?;
        Ok(resolved)
    }

    /// Read and parse an existing Markdown document from disk.
    pub fn read_document(
        validator: &PathValidator,
        rel_path: &RelativePath,
    ) -> Result<VaultDocument, VaultError> {
        let resolved = validator.resolve(rel_path)?;

        if !resolved.as_path().exists() {
            return Err(VaultError::DocumentNotFound(rel_path.to_string_lossy()));
        }

        let content = fs::read_to_string(resolved.as_path()).map_err(VaultError::Io)?;
        VaultDocument::parse(&content)
    }

    /// Update an existing Markdown document on disk using atomic temporary file replacement.
    pub fn update_document(
        validator: &PathValidator,
        rel_path: &RelativePath,
        doc: &VaultDocument,
    ) -> Result<ResolvedPath, VaultError> {
        let resolved = validator.resolve(rel_path)?;

        if !resolved.as_path().exists() {
            return Err(VaultError::DocumentNotFound(rel_path.to_string_lossy()));
        }

        Self::atomic_write(&resolved, doc)?;
        Ok(resolved)
    }

    /// Delete an existing Markdown document from disk.
    pub fn delete_document(
        validator: &PathValidator,
        rel_path: &RelativePath,
    ) -> Result<(), VaultError> {
        let resolved = validator.resolve(rel_path)?;

        if !resolved.as_path().exists() {
            return Err(VaultError::DocumentNotFound(rel_path.to_string_lossy()));
        }

        fs::remove_file(resolved.as_path()).map_err(VaultError::Io)?;
        Ok(())
    }

    /// Check whether a document exists at the given relative path.
    pub fn exists(validator: &PathValidator, rel_path: &RelativePath) -> bool {
        if let Ok(resolved) = validator.resolve(rel_path) {
            resolved.as_path().exists()
        } else {
            false
        }
    }

    /// List all Markdown documents inside the vault root.
    pub fn list_documents(validator: &PathValidator) -> Result<Vec<RelativePath>, VaultError> {
        let root = validator.root().as_path();
        let mut results = Vec::new();

        fn walk_dir(
            dir: &std::path::Path,
            root: &std::path::Path,
            results: &mut Vec<RelativePath>,
        ) -> Result<(), VaultError> {
            if !dir.exists() {
                return Ok(());
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                let file_name = entry.file_name();
                let name_str = file_name.to_string_lossy();

                // Skip hidden files and temporary artifacts
                if name_str.starts_with('.') {
                    continue;
                }

                if path.is_dir() {
                    walk_dir(&path, root, results)?;
                } else if path.is_file()
                    && (name_str.ends_with(".md") || name_str.ends_with(".markdown"))
                {
                    if let Ok(rel) = path.strip_prefix(root) {
                        if let Ok(rel_path) = RelativePath::parse(rel) {
                            results.push(rel_path);
                        }
                    }
                }
            }
            Ok(())
        }

        walk_dir(root, root, &mut results)?;
        Ok(results)
    }

    /// Atomic write helper: writes document content to a temporary file in the same parent directory,
    /// flushes/syncs to disk, and atomically renames (`fs::rename`) the temporary file to the target path.
    fn atomic_write(resolved: &ResolvedPath, doc: &VaultDocument) -> Result<(), VaultError> {
        let target_path = resolved.as_path();
        let parent = target_path.parent().ok_or_else(|| {
            VaultError::InvalidPath("Target path has no valid parent directory".to_string())
        })?;

        fs::create_dir_all(parent)?;

        let temp_path = parent.join(format!(".tmp_{}", Uuid::new_v4()));
        let markdown_text = doc.to_markdown()?;

        let write_res = (|| -> Result<(), std::io::Error> {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)?;

            file.write_all(markdown_text.as_bytes())?;
            file.flush()?;
            file.sync_all()?;
            Ok(())
        })();

        if let Err(e) = write_res {
            let _ = fs::remove_file(&temp_path);
            return Err(VaultError::Io(e));
        }

        if let Err(e) = fs::rename(&temp_path, target_path) {
            let _ = fs::remove_file(&temp_path);
            return Err(VaultError::Io(e));
        }

        Ok(())
    }
}
