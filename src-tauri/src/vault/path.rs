use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::vault::errors::VaultError;

/// Strongly-typed container for a canonicalized, verified vault root directory path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultRoot {
    path: PathBuf,
}

impl VaultRoot {
    pub fn new(root_path: impl AsRef<Path>) -> Result<Self, VaultError> {
        let p = root_path.as_ref();
        fs::create_dir_all(p)?;
        let canonical = p.canonicalize().map_err(|e| {
            VaultError::InvalidPath(format!("Failed to canonicalize vault root: {e}"))
        })?;
        Ok(Self { path: canonical })
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

/// Strongly-typed container for a normalized relative path inside the vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativePath {
    path: PathBuf,
}

impl RelativePath {
    pub fn parse(rel_path: impl AsRef<Path>) -> Result<Self, VaultError> {
        let p = rel_path.as_ref();
        let path_str = p.to_string_lossy();

        // 1. Reject absolute paths and drive prefixes across all platforms
        if p.is_absolute()
            || (path_str.len() >= 2 && path_str.chars().nth(1) == Some(':'))
            || path_str.starts_with('\\')
            || path_str.starts_with('/')
        {
            return Err(VaultError::PathTraversal(format!(
                "Absolute paths or drive prefixes not permitted: {path_str}"
            )));
        }

        // 2. Reject parent directory components ('..') or root prefixes
        for component in p.components() {
            match component {
                Component::ParentDir => {
                    return Err(VaultError::PathTraversal(format!(
                        "Parent directory traversal '..' not permitted: {path_str}"
                    )));
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(VaultError::PathTraversal(format!(
                        "Root prefix or drive letter not permitted in relative path: {path_str}"
                    )));
                }
                _ => (),
            }
        }

        // 3. Normalize path: convert Windows backslashes if present and collect clean Normal components
        let normalized_str = path_str.replace('\\', "/");
        let mut clean = PathBuf::new();
        for comp in Path::new(&normalized_str).components() {
            if let Component::Normal(s) = comp {
                clean.push(s);
            }
        }

        if clean.as_os_str().is_empty() {
            return Err(VaultError::InvalidPath(
                "Relative path cannot be empty".to_string(),
            ));
        }

        Ok(Self { path: clean })
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }

    pub fn to_string_lossy(&self) -> String {
        self.path.to_string_lossy().replace('\\', "/")
    }
}

/// Strongly-typed container for a fully resolved, boundary-verified absolute filesystem path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPath {
    path: PathBuf,
}

impl ResolvedPath {
    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

/// Boundary validator that guarantees target paths reside strictly inside the configured vault root.
pub struct PathValidator {
    root: VaultRoot,
}

impl PathValidator {
    pub fn new(root: VaultRoot) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &VaultRoot {
        &self.root
    }

    /// Resolve a relative path against the vault root and verify that it does not escape the boundary.
    pub fn resolve(&self, rel_path: &RelativePath) -> Result<ResolvedPath, VaultError> {
        let candidate = self.root.as_path().join(rel_path.as_path());

        if candidate.exists() {
            let canonical_candidate = candidate.canonicalize().map_err(|e| {
                VaultError::InvalidPath(format!("Failed to canonicalize candidate path: {e}"))
            })?;

            if !canonical_candidate.starts_with(self.root.as_path()) {
                return Err(VaultError::PathTraversal(format!(
                    "Resolved target path escapes vault root boundary: {}",
                    rel_path.to_string_lossy()
                )));
            }

            Ok(ResolvedPath {
                path: canonical_candidate,
            })
        } else {
            // Target file does not exist yet. Validate that its parent directory is inside the vault.
            let file_name = candidate
                .file_name()
                .ok_or_else(|| VaultError::InvalidPath("Invalid document file name".to_string()))?;

            let parent = candidate.parent().ok_or_else(|| {
                VaultError::InvalidPath("Invalid document parent directory".to_string())
            })?;

            fs::create_dir_all(parent)?;

            let canonical_parent = parent.canonicalize().map_err(|e| {
                VaultError::InvalidPath(format!("Failed to canonicalize parent directory: {e}"))
            })?;

            if !canonical_parent.starts_with(self.root.as_path()) {
                return Err(VaultError::PathTraversal(format!(
                    "Resolved parent directory escapes vault root boundary: {}",
                    rel_path.to_string_lossy()
                )));
            }

            let resolved = canonical_parent.join(file_name);

            // Double check symlink escape on target path
            if resolved.is_symlink() {
                let canonical_target = resolved.canonicalize().map_err(|e| {
                    VaultError::InvalidPath(format!("Failed to resolve target symlink: {e}"))
                })?;
                if !canonical_target.starts_with(self.root.as_path()) {
                    return Err(VaultError::PathTraversal(format!(
                        "Target symlink escapes vault root boundary: {}",
                        rel_path.to_string_lossy()
                    )));
                }
            }

            Ok(ResolvedPath { path: resolved })
        }
    }
}
