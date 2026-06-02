use framkey_core::Result;
use framkey_device::{DeviceInfo, DeviceKind, SaveImage, VaultDevice};

#[derive(Debug, Clone)]
pub struct InMemoryDevice {
    label: String,
    image: SaveImage,
}

impl InMemoryDevice {
    pub fn new(label: impl Into<String>, image: SaveImage) -> Self {
        Self {
            label: label.into(),
            image,
        }
    }
}

impl VaultDevice for InMemoryDevice {
    fn probe(&self) -> Result<DeviceInfo> {
        Ok(DeviceInfo {
            kind: DeviceKind::InMemory,
            label: self.label.clone(),
            save_size: Some(self.image.len()),
        })
    }

    fn read_save_image(&self) -> Result<SaveImage> {
        Ok(self.image.clone())
    }

    fn write_save_image(&mut self, image: &SaveImage) -> Result<()> {
        self.image = image.clone();
        Ok(())
    }
}
