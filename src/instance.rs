//! Single-instance guard: two processes using the same SQLite database can overwrite each
//! other's notes, so the data directory is locked for the lifetime of the app.

use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::Path;

use anyhow::{bail, Context, Result};

pub struct InstanceLock {
    _file: File,
}

pub fn acquire(root: &Path) -> Result<InstanceLock> {
    std::fs::create_dir_all(root).context("create the data directory")?;
    let path = root.join(".lock");
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .with_context(|| format!("open the instance lock {}", path.display()))?;
    crate::fsutil::restrict_file(&path);

    // Safety: `file` owns a valid file descriptor for the lifetime of the call.
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::EWOULDBLOCK) {
            bail!(
                "Logpane is already running (lock held on {})",
                path.display()
            );
        }
        bail!("could not lock {}: {error}", path.display());
    }
    Ok(InstanceLock { _file: file })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_instance_is_refused_until_the_first_is_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let first = acquire(dir.path()).unwrap();
        assert!(acquire(dir.path()).is_err());
        drop(first);
        assert!(acquire(dir.path()).is_ok());
    }
}
