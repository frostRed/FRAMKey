use framkey_core::{FramkeyError, Result};
use framkey_device::{DeviceInfo, DeviceKind, SaveImage, VaultDevice};

use crate::{Ch347Config, flashrom::FlashromTool, temp::TempWorkspace};

#[derive(Debug, Clone)]
pub struct Ch347Device {
    config: Ch347Config,
}

impl Ch347Device {
    pub fn new(config: Ch347Config) -> Self {
        Self { config }
    }

    pub fn write_save_image_verified(
        &mut self,
        image: &SaveImage,
    ) -> Result<Ch347WriteVerifyReport> {
        self.validate_write_size(image)?;

        let tool = self.tool()?;
        let temp = TempWorkspace::new()?;
        let input = temp.write_private_file("write-input.bin", image.as_bytes())?;
        let readback = temp.path("fresh-readback.bin");

        tool.write_from(&input)?;
        tool.read_to(&readback)?;

        let readback = SaveImage::new(read_flashrom_output(&readback)?);
        if readback.as_bytes() != image.as_bytes() {
            return Err(FramkeyError::invalid_data(
                "CH347 write completed but fresh readback verification failed",
            ));
        }

        Ok(Ch347WriteVerifyReport {
            save_size: image.len(),
            input_blake3: image.blake3_hash().to_string(),
            readback_blake3: readback.blake3_hash().to_string(),
            exact_match: true,
        })
    }

    fn tool(&self) -> Result<FlashromTool> {
        self.config.validate()?;
        FlashromTool::new(&self.config)
    }

    fn validate_read_size(&self, image: &SaveImage) -> Result<()> {
        if let Some(expected) = self.config.expected_size
            && image.len() != expected
        {
            return Err(FramkeyError::invalid_data(format!(
                "CH347 read returned {} bytes, expected {expected}",
                image.len()
            )));
        }

        Ok(())
    }

    fn validate_write_size(&self, image: &SaveImage) -> Result<()> {
        if let Some(expected) = self.config.expected_size
            && image.len() != expected
        {
            return Err(FramkeyError::invalid_data(format!(
                "CH347 write requires {expected} bytes, got {}",
                image.len()
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ch347WriteVerifyReport {
    pub save_size: usize,
    pub input_blake3: String,
    pub readback_blake3: String,
    pub exact_match: bool,
}

impl VaultDevice for Ch347Device {
    fn probe(&self) -> Result<DeviceInfo> {
        let tool = self.tool()?;
        let report = tool.probe()?;
        let chip = report
            .chip_name
            .as_deref()
            .or_else(|| self.config.chip_name().ok().flatten())
            .unwrap_or("auto-detect");

        Ok(DeviceInfo {
            kind: DeviceKind::Ch347,
            label: format!(
                "CH347 SPI ROM via flashrom {}; chip {}",
                self.config.programmer_arg(),
                chip
            ),
            save_size: report.size_bytes.or(self.config.expected_size),
        })
    }

    fn read_save_image(&self) -> Result<SaveImage> {
        let tool = self.tool()?;
        let temp = TempWorkspace::new()?;
        let output = temp.path("readback.bin");

        tool.read_to(&output)?;
        let image = SaveImage::new(read_flashrom_output(&output)?);
        self.validate_read_size(&image)?;
        Ok(image)
    }

    fn write_save_image(&mut self, image: &SaveImage) -> Result<()> {
        self.write_save_image_verified(image).map(|_| ())
    }
}

fn read_flashrom_output(path: &std::path::Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|error| {
        FramkeyError::invalid_data(format!(
            "flashrom CH347 read did not produce an output image at {}: {error}",
            path.display()
        ))
    })
}
