use std::path::PathBuf;

use anyhow::Result;
use framkey_ch347::{Ch347Config, Ch347Device};
use framkey_device::{FileImageDevice, SaveImage, SaveImageHash, VaultDevice};
use framkey_gbxcart::{GbxCartConfig, GbxCartDevice};
use serde_json::json;

use crate::{
    args::{DeviceCommand, DeviceTargetArgs, DeviceTargetKind},
    files::write_new_file,
};

pub(crate) fn run_device(command: DeviceCommand) -> Result<()> {
    match command {
        DeviceCommand::Probe(target) => {
            let device = open_device(&target)?;
            let info = device.probe()?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "kind": info.kind.as_str(),
                    "label": info.label,
                    "save_size": info.save_size,
                }))?
            );
        }
        DeviceCommand::ReadSave(args) => {
            let device = open_device(&args.target)?;
            let image = device.read_save_image()?;
            write_new_file(&args.out, image.as_bytes())?;
            print_save_image_report("read_save", image, Some(args.out))?;
        }
        DeviceCommand::WriteSave(args) => {
            let mut device = open_device(&args.target)?;
            let image = SaveImage::new(std::fs::read(&args.input)?);
            device.write_save_image(&image)?;
            print_save_image_report("write_save", image, None)?;
        }
        DeviceCommand::VerifySave(args) => {
            let device = open_device(&args.target)?;
            let image = device.read_save_image()?;
            let actual = image.blake3_hash();
            let expected = SaveImageHash::from_hex(&args.blake3)?;

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": "verify_save",
                    "save_size": image.len(),
                    "blake3": actual.to_string(),
                    "expected_blake3": expected.to_string(),
                    "match": actual == expected,
                }))?
            );

            if actual != expected {
                anyhow::bail!("save image hash mismatch");
            }
        }
    }

    Ok(())
}

pub(crate) fn open_device(args: &DeviceTargetArgs) -> Result<Box<dyn VaultDevice>> {
    match args.device {
        DeviceTargetKind::Ch347 => Ok(Box::new(Ch347Device::new(Ch347Config {
            chip: args.chip.clone(),
            flashrom_path: args.flashrom.clone(),
            spi_speed: args.spispeed.map(Into::into),
            expected_size: args.expected_save_size,
        }))),
        DeviceTargetKind::File => {
            let path = args
                .path
                .clone()
                .ok_or_else(|| anyhow::anyhow!("--path is required for --device file"))?;
            Ok(Box::new(FileImageDevice::new(path)))
        }
        DeviceTargetKind::GbxCart => Ok(Box::new(GbxCartDevice::new(GbxCartConfig {
            port_hint: args.port.clone(),
            expected_save_size: args.expected_save_size,
            save_type: args.save_type.map(Into::into),
        }))),
    }
}

fn print_save_image_report(operation: &str, image: SaveImage, out: Option<PathBuf>) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "operation": operation,
            "save_size": image.len(),
            "blake3": image.blake3_hash().to_string(),
            "out": out.map(|path| path.display().to_string()),
        }))?
    );

    Ok(())
}
