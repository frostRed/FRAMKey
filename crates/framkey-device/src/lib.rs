mod file_image;
mod info;
mod save_image;
mod vault_device;

pub use file_image::FileImageDevice;
pub use info::{DeviceInfo, DeviceKind};
pub use save_image::{SaveImage, SaveImageHash};
pub use vault_device::VaultDevice;

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn save_image_hash_display_round_trips() {
        let image = SaveImage::new(b"framkey fixture".to_vec());
        let hash = image.blake3_hash();

        assert_eq!(SaveImageHash::from_hex(&hash.to_string()).unwrap(), hash);
    }

    #[test]
    fn save_image_hash_rejects_malformed_hex() {
        assert!(SaveImageHash::from_hex("0x1234").is_err());
        assert!(
            SaveImageHash::from_hex(
                "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"
            )
            .is_err()
        );
    }

    #[test]
    fn file_image_device_reads_and_writes_opaque_bytes() {
        let path = std::env::temp_dir().join(format!(
            "framkey-device-test-{}-{}.sav",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        std::fs::write(&path, b"initial save image").unwrap();

        let mut device = FileImageDevice::new(&path);
        let info = device.probe().unwrap();
        assert_eq!(info.kind, DeviceKind::FileImage);
        assert_eq!(info.save_size, Some(18));

        let initial = device.read_save_image().unwrap();
        assert_eq!(initial.as_bytes(), b"initial save image");

        let updated = SaveImage::new(b"updated save image".to_vec());
        device.write_save_image(&updated).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"updated save image");

        std::fs::remove_file(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn file_image_device_creates_owner_only_save_files() {
        use std::os::unix::fs::PermissionsExt;

        let path = std::env::temp_dir().join(format!(
            "framkey-device-permissions-test-{}-{}.sav",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_file(&path);

        let mut device = FileImageDevice::new(&path);
        device
            .write_save_image(&SaveImage::new(b"owner only".to_vec()))
            .unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        std::fs::remove_file(path).unwrap();
    }
}
