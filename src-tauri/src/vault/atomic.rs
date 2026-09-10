use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use uuid::Uuid;

use crate::vault::errors::VaultError;

/// Atomically write content to target path by writing to a temporary file in the same directory,
/// flushing buffers, syncing to disk, and performing an atomic rename.
/// Cleans up temporary files if any error occurs during write/sync/rename.
pub fn write_atomic(target_path: &Path, content: &[u8]) -> Result<(), VaultError> {
    let parent = target_path.parent().ok_or_else(|| {
        VaultError::InvalidPath(
            target_path.display().to_string(),
            "Target path has no parent directory".to_string(),
        )
    })?;

    fs::create_dir_all(parent)?;

    let temp_name = format!(".tmp_{}.tmp", Uuid::new_v4());
    let temp_path = parent.join(temp_name);

    let write_res = (|| -> Result<(), VaultError> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;

        file.write_all(content)?;
        file.flush()?;
        file.sync_all()?;
        drop(file);

        fs::rename(&temp_path, target_path)?;
        Ok(())
    })();

    if write_res.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    write_res
}
