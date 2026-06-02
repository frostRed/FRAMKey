use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use framkey_core::Result;

use crate::{DeviceInfo, DeviceKind, SaveImage, VaultDevice};

#[derive(Debug, Clone)]
pub struct FileImageDevice {
    path: PathBuf,
}

impl FileImageDevice {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl VaultDevice for FileImageDevice {
    fn probe(&self) -> Result<DeviceInfo> {
        let metadata = std::fs::metadata(&self.path)?;
        Ok(DeviceInfo {
            kind: DeviceKind::FileImage,
            label: self.path.display().to_string(),
            save_size: Some(metadata.len() as usize),
        })
    }

    fn read_save_image(&self) -> Result<SaveImage> {
        Ok(SaveImage::new(std::fs::read(&self.path)?))
    }

    fn write_save_image(&mut self, image: &SaveImage) -> Result<()> {
        let mut file = open_save_image_for_write(&self.path)?;
        file.write_all(image.as_bytes())?;
        Ok(())
    }
}

#[cfg(unix)]
fn open_save_image_for_write(path: &Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn open_save_image_for_write(path: &Path) -> std::io::Result<std::fs::File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
}
