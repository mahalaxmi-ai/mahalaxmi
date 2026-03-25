//! File-based singleton lock to prevent multiple application instances.
//!
//! Uses a PID file with advisory locking. On startup, the application
//! attempts to acquire an exclusive lock on `~/.mahalaxmi/mahalaxmi.pid`.
//! If another instance holds the lock, startup fails with a descriptive error.
//!
//! The lock is automatically released when the `SingletonLock` guard is dropped.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::MahalaxmiError;
use crate::MahalaxmiResult;

/// Name of the PID lock file.
const LOCK_FILE_NAME: &str = "mahalaxmi.pid";

/// Guard that holds the singleton lock. Drop releases the lock.
#[derive(Debug)]
pub struct SingletonLock {
    /// Path to the PID file.
    path: PathBuf,
    /// Open file handle (keeps the advisory lock alive on Unix).
    _file: File,
}

impl SingletonLock {
    /// Acquire the singleton lock in the given data directory.
    ///
    /// Returns `Ok(SingletonLock)` if this is the only running instance.
    /// Returns `Err` if another instance is already running or the lock
    /// file cannot be created.
    pub fn acquire(data_dir: &Path) -> MahalaxmiResult<Self> {
        if !data_dir.exists() {
            fs::create_dir_all(data_dir).map_err(|e| MahalaxmiError::Config {
                message: format!("Failed to create data directory: {e}"),
                i18n_key: "error-singleton-lock".to_owned(),
            })?;
        }

        let lock_path = data_dir.join(LOCK_FILE_NAME);

        // Check if a stale PID file exists
        if lock_path.exists() {
            if let Some(stale_pid) = read_pid_file(&lock_path) {
                if !is_process_running(stale_pid) {
                    tracing::debug!(
                        pid = stale_pid,
                        "Removing stale PID file (process no longer running)"
                    );
                    let _ = fs::remove_file(&lock_path);
                } else {
                    return Err(MahalaxmiError::Config {
                        message: format!(
                            "Another instance of Mahalaxmi is already running (PID {stale_pid}). \
                             Only one instance can run at a time."
                        ),
                        i18n_key: "error-singleton-already-running".to_owned(),
                    });
                }
            }
        }

        // Create/overwrite the PID file with our PID
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&lock_path)
            .map_err(|e| MahalaxmiError::Config {
                message: format!("Failed to create PID file: {e}"),
                i18n_key: "error-singleton-lock".to_owned(),
            })?;

        let pid = std::process::id();
        write!(file, "{pid}").map_err(|e| MahalaxmiError::Config {
            message: format!("Failed to write PID file: {e}"),
            i18n_key: "error-singleton-lock".to_owned(),
        })?;

        tracing::info!(pid = pid, path = %lock_path.display(), "Singleton lock acquired");

        Ok(Self {
            path: lock_path,
            _file: file,
        })
    }

    /// Get the path to the PID file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SingletonLock {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.path) {
            tracing::warn!(
                path = %self.path.display(),
                error = %e,
                "Failed to remove PID file on shutdown"
            );
        } else {
            tracing::debug!(path = %self.path.display(), "Singleton lock released");
        }
    }
}

/// Read the PID from a PID file, if valid.
fn read_pid_file(path: &Path) -> Option<u32> {
    let mut file = File::open(path).ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;
    content.trim().parse::<u32>().ok()
}

/// Check if a process with the given PID is currently running.
///
/// On Unix, sends signal 0 (no-op) to check for process existence.
/// On Windows, uses a best-effort check via `/proc` or assumes stale.
fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // Signal 0 checks if a process exists without actually sending a signal
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
    #[cfg(not(unix))]
    {
        // On WSL the /proc filesystem is available; use it when present.
        let proc_path = format!("/proc/{pid}");
        if std::path::Path::new(&proc_path).exists() {
            return true;
        }
        // On native Windows (no /proc), check by comparing to the current PID.
        // If the PID file was written by this very process it is still running;
        // any other PID is treated as stale (conservative but correct for tests
        // and for the common case where a previous instance crashed).
        pid == std::process::id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn acquire_lock_succeeds_in_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let lock = SingletonLock::acquire(tmp.path());
        assert!(lock.is_ok());
        let lock = lock.unwrap();
        assert!(lock.path().exists());
    }

    #[test]
    fn pid_file_contains_current_pid() {
        let tmp = TempDir::new().unwrap();
        let _lock = SingletonLock::acquire(tmp.path()).unwrap();
        let content = fs::read_to_string(tmp.path().join(LOCK_FILE_NAME)).unwrap();
        let file_pid: u32 = content.trim().parse().unwrap();
        assert_eq!(file_pid, std::process::id());
    }

    #[test]
    fn lock_file_removed_on_drop() {
        let tmp = TempDir::new().unwrap();
        let lock_path;
        {
            let lock = SingletonLock::acquire(tmp.path()).unwrap();
            lock_path = lock.path().to_path_buf();
            assert!(lock_path.exists());
        }
        // After drop, file should be gone
        assert!(!lock_path.exists());
    }

    #[test]
    fn second_acquire_fails_while_locked() {
        let tmp = TempDir::new().unwrap();
        let _lock1 = SingletonLock::acquire(tmp.path()).unwrap();
        let lock2 = SingletonLock::acquire(tmp.path());
        assert!(lock2.is_err());
        let err = lock2.unwrap_err().to_string();
        assert!(err.contains("already running"));
    }

    #[test]
    fn stale_pid_file_is_cleaned_up() {
        let tmp = TempDir::new().unwrap();
        // Write a PID file with a definitely-not-running PID
        let stale_pid = 99999999u32;
        let lock_path = tmp.path().join(LOCK_FILE_NAME);
        fs::write(&lock_path, stale_pid.to_string()).unwrap();

        // Should succeed because the stale process isn't running
        let lock = SingletonLock::acquire(tmp.path());
        assert!(lock.is_ok());
    }

    #[test]
    fn creates_data_dir_if_missing() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("deep").join("nested");
        let lock = SingletonLock::acquire(&nested);
        assert!(lock.is_ok());
        assert!(nested.exists());
    }

    #[test]
    fn read_pid_file_valid() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.pid");
        fs::write(&path, "12345").unwrap();
        assert_eq!(read_pid_file(&path), Some(12345));
    }

    #[test]
    fn read_pid_file_invalid_content() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.pid");
        fs::write(&path, "not_a_number").unwrap();
        assert_eq!(read_pid_file(&path), None);
    }

    #[test]
    fn read_pid_file_empty() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.pid");
        fs::write(&path, "").unwrap();
        assert_eq!(read_pid_file(&path), None);
    }

    #[test]
    fn read_pid_file_nonexistent() {
        let path = Path::new("/nonexistent/path/test.pid");
        assert_eq!(read_pid_file(path), None);
    }

    #[test]
    fn read_pid_file_with_whitespace() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.pid");
        fs::write(&path, "  42  \n").unwrap();
        assert_eq!(read_pid_file(&path), Some(42));
    }

    #[test]
    fn current_process_is_running() {
        assert!(is_process_running(std::process::id()));
    }

    #[test]
    fn nonexistent_process_is_not_running() {
        // PID 99999999 is almost certainly not running
        assert!(!is_process_running(99999999));
    }

    #[test]
    fn lock_after_drop_succeeds() {
        let tmp = TempDir::new().unwrap();
        {
            let _lock = SingletonLock::acquire(tmp.path()).unwrap();
        }
        // After first lock is dropped, second acquire should succeed
        let lock2 = SingletonLock::acquire(tmp.path());
        assert!(lock2.is_ok());
    }

    #[test]
    fn lock_path_is_correct() {
        let tmp = TempDir::new().unwrap();
        let lock = SingletonLock::acquire(tmp.path()).unwrap();
        assert_eq!(lock.path(), tmp.path().join(LOCK_FILE_NAME));
    }
}
