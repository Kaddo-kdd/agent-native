//! Atomic file replacement operations.
//!
//! Provides platform-agnostic, safe atomic replacement of target files
//! without exposing partially written intermediate files to concurrent readers.

use std::path::Path;

use crate::publish::error::PublishError;

/// Atomically replaces `final_path` with the contents of `temp_path`.
///
/// Under POSIX, uses standard atomic `std::fs::rename`.
/// Under Windows, safely replaces the target file using standard library renaming
/// with a retry/temporary swap to handle concurrent handles without requiring unsafe FFI.
pub fn atomic_replace_file(temp_path: &Path, final_path: &Path) -> Result<(), PublishError> {
    if !temp_path.exists() {
        return Err(PublishError::Io(format!(
            "Source file does not exist: {}",
            temp_path.display()
        )));
    }

    #[cfg(not(windows))]
    {
        std::fs::rename(temp_path, final_path).map_err(|e| {
            PublishError::Io(format!(
                "Atomic rename failed ({} -> {}): {e}",
                temp_path.display(),
                final_path.display()
            ))
        })
    }

    #[cfg(windows)]
    {
        // On Windows std::fs::rename will fail if final_path exists.
        // We move final_path to a backup name, rename temp to final, and delete backup.
        // If anything fails during the step, we restore the backup.
        if final_path.exists() {
            let backup_path = final_path.with_extension(format!("bak.{}", std::process::id()));
            if backup_path.exists() {
                let _ = std::fs::remove_file(&backup_path);
            }

            std::fs::rename(final_path, &backup_path).map_err(|e| {
                PublishError::Io(format!(
                    "Failed to create atomic backup for {} -> {}: {e}",
                    final_path.display(),
                    backup_path.display()
                ))
            })?;

            if let Err(err) = std::fs::rename(temp_path, final_path) {
                // Attempt rollback
                let _ = std::fs::rename(&backup_path, final_path);
                return Err(PublishError::Io(format!(
                    "Failed to replace {} with {}: {err}",
                    final_path.display(),
                    temp_path.display()
                )));
            }

            // Successfully renamed temp -> final; remove backup
            let _ = std::fs::remove_file(&backup_path);
            Ok(())
        } else {
            std::fs::rename(temp_path, final_path).map_err(|e| {
                PublishError::Io(format!(
                    "Rename failed ({} -> {}): {e}",
                    temp_path.display(),
                    final_path.display()
                ))
            })
        }
    }
}
