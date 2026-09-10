use std::path::{Component, Path, PathBuf};

use crate::vault::errors::VaultError;

/// Validate that a given relative path stays strictly within the configured vault root directory.
/// Rejects absolute paths, traversal (`..`), Windows drive/UNC/device paths, null bytes,
/// and symlink/reparse-point escape attempts.
pub fn validate_and_resolve_relative_path(
    vault_root: &Path,
    rel_path_str: &str,
) -> Result<PathBuf, VaultError> {
    if rel_path_str.trim().is_empty() {
        return Err(VaultError::InvalidPath(
            rel_path_str.to_string(),
            "Relative path cannot be empty".to_string(),
        ));
    }

    if rel_path_str.contains('\0') {
        return Err(VaultError::InvalidPath(
            rel_path_str.to_string(),
            "Path contains null bytes".to_string(),
        ));
    }

    let normalized = rel_path_str.replace('\\', "/");

    // Explicit checks for POSIX absolute, Windows drive, UNC, device namespace paths
    let bytes = rel_path_str.as_bytes();
    let norm_bytes = normalized.as_bytes();

    if rel_path_str.starts_with('/')
        || rel_path_str.starts_with('\\')
        || (bytes.len() >= 2 && bytes[1] == b':' && (bytes[0] as char).is_ascii_alphabetic())
        || (norm_bytes.len() >= 2
            && norm_bytes[1] == b':'
            && (norm_bytes[0] as char).is_ascii_alphabetic())
        || rel_path_str.starts_with("//")
        || rel_path_str.starts_with("\\\\")
        || rel_path_str.contains(":\\")
        || rel_path_str.contains(":/")
        || normalized.contains(":/")
    {
        return Err(VaultError::InvalidPath(
            rel_path_str.to_string(),
            "Absolute, UNC, device namespace, or drive paths are rejected".to_string(),
        ));
    }

    let rel_path = Path::new(&normalized);

    // Inspect individual path components
    for comp in rel_path.components() {
        match comp {
            Component::ParentDir => {
                return Err(VaultError::PathTraversal(rel_path_str.to_string()));
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err(VaultError::InvalidPath(
                    rel_path_str.to_string(),
                    "Absolute path component detected".to_string(),
                ));
            }
            Component::Normal(os_str) => {
                let s = os_str.to_string_lossy();
                if s == ".." || s == "." {
                    return Err(VaultError::PathTraversal(rel_path_str.to_string()));
                }
            }
            Component::CurDir => {}
        }
    }

    // Ensure vault root is canonicalized
    let canonical_root = vault_root.canonicalize().map_err(|e| {
        VaultError::InvalidPath(
            vault_root.display().to_string(),
            format!("Vault root canonicalization failed: {e}"),
        )
    })?;

    let full_target = vault_root.join(rel_path);

    // Find closest existing ancestor directory to verify canonical path hierarchy
    let mut current = full_target.clone();
    let mut existing_ancestor = None;

    while let Some(parent) = current.parent() {
        if parent.exists() {
            existing_ancestor = Some(parent.to_path_buf());
            break;
        }
        current = parent.to_path_buf();
    }

    let ancestor = existing_ancestor.unwrap_or_else(|| vault_root.to_path_buf());

    let canonical_ancestor = ancestor
        .canonicalize()
        .map_err(|e| VaultError::SymlinkEscape(format!("Failed to canonicalize ancestor: {e}")))?;

    if !canonical_ancestor.starts_with(&canonical_root) {
        return Err(VaultError::SymlinkEscape(format!(
            "Path ancestor '{}' escapes vault root '{}'",
            canonical_ancestor.display(),
            canonical_root.display()
        )));
    }

    // If target path itself exists, verify its canonical path does not escape
    if full_target.exists() {
        let canonical_target = full_target.canonicalize().map_err(|e| {
            VaultError::SymlinkEscape(format!("Failed to canonicalize target path: {e}"))
        })?;

        if !canonical_target.starts_with(&canonical_root) {
            return Err(VaultError::SymlinkEscape(format!(
                "Target path '{}' escapes vault root '{}'",
                canonical_target.display(),
                canonical_root.display()
            )));
        }
    }

    Ok(full_target)
}
