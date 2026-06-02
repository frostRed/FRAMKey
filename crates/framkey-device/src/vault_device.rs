use framkey_core::Result;

use crate::{DeviceInfo, SaveImage};

pub trait VaultDevice {
    fn probe(&self) -> Result<DeviceInfo>;
    fn read_save_image(&self) -> Result<SaveImage>;
    fn write_save_image(&mut self, image: &SaveImage) -> Result<()>;
}
