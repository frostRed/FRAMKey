use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use framkey_core::Result;

#[derive(Debug)]
pub(crate) struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    pub(crate) fn new() -> Result<Self> {
        let base = std::env::temp_dir();
        let mut last_error = None;

        for attempt in 0..16 {
            let path = base.join(format!(
                "framkey-ch347-{}-{}-{attempt}",
                std::process::id(),
                timestamp_nanos()
            ));

            match create_private_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    last_error = Some(error);
                }
                Err(error) => return Err(error.into()),
            }
        }

        Err(last_error
            .unwrap_or_else(|| std::io::Error::other("failed to create CH347 temp workspace"))
            .into())
    }

    pub(crate) fn path(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }

    pub(crate) fn write_private_file(&self, name: &str, bytes: &[u8]) -> Result<PathBuf> {
        let path = self.path(name);
        write_private_file(&path, bytes)?;
        Ok(path)
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[cfg(unix)]
fn create_private_dir(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    let mut builder = std::fs::DirBuilder::new();
    builder.mode(0o700).create(path)
}

#[cfg(not(unix))]
fn create_private_dir(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir(path)
}

#[cfg(unix)]
fn write_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::{io::Write, os::unix::fs::OpenOptionsExt};

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)
}

#[cfg(not(unix))]
fn write_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)
}

fn timestamp_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}
