use std::{io::Write, path::Path};

use anyhow::Result;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

pub(crate) fn write_new_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);

    let mut file = options
        .open(path)
        .map_err(|error| anyhow::anyhow!("failed to create {}: {error}", path.display()))?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("framkey-cli-{name}-{}", std::process::id()))
    }

    #[test]
    fn write_new_file_refuses_to_overwrite() {
        let path = test_path("create-new");
        let _ = std::fs::remove_file(&path);

        write_new_file(&path, b"first").unwrap();
        let error = write_new_file(&path, b"second").unwrap_err();

        assert!(error.to_string().contains("failed to create"));
        assert_eq!(std::fs::read(&path).unwrap(), b"first");
        let _ = std::fs::remove_file(&path);
    }

    #[cfg(unix)]
    #[test]
    fn write_new_file_creates_owner_only_files() {
        use std::os::unix::fs::PermissionsExt;

        let path = test_path("mode");
        let _ = std::fs::remove_file(&path);

        write_new_file(&path, b"secret-ish").unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        let _ = std::fs::remove_file(&path);
    }
}
