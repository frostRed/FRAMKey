mod in_memory_device;

pub use in_memory_device::InMemoryDevice;

#[cfg(test)]
mod tests {
    use framkey_device::{DeviceKind, SaveImage, VaultDevice};

    use super::InMemoryDevice;

    #[test]
    fn in_memory_device_preserves_vault_device_semantics() {
        let mut device = InMemoryDevice::new("fixture", SaveImage::new(b"initial".to_vec()));

        let info = device.probe().unwrap();
        assert_eq!(info.kind, DeviceKind::InMemory);
        assert_eq!(info.label, "fixture");
        assert_eq!(info.save_size, Some(7));
        assert_eq!(device.read_save_image().unwrap().as_bytes(), b"initial");

        device
            .write_save_image(&SaveImage::new(b"updated-save".to_vec()))
            .unwrap();
        assert_eq!(
            device.read_save_image().unwrap().as_bytes(),
            b"updated-save"
        );
        assert_eq!(device.probe().unwrap().save_size, Some(12));
    }
}
