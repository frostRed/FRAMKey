use std::path::PathBuf;

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
        std::fs::write(&self.path, image.as_bytes())?;
        Ok(())
    }
}
