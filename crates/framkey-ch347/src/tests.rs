use std::{
    ffi::OsString,
    time::{SystemTime, UNIX_EPOCH},
};

use framkey_device::{SaveImage, VaultDevice};

use crate::{
    Ch347Config, Ch347Device, Ch347SpiSpeed,
    flashrom::{FlashromOperation, FlashromTool, parse_flashrom_probe_output},
};

#[test]
fn programmer_arg_uses_flashrom_ch347_spi_speed_values() {
    let config = Ch347Config {
        chip: Some("W25Q64JV-.Q".to_owned()),
        flashrom_path: None,
        spi_speed: Some(Ch347SpiSpeed::M7_5),
        expected_size: Some(8 * 1024 * 1024),
    };

    assert_eq!(config.programmer_arg(), "ch347_spi:spispeed=7.5M");
}

#[test]
fn flashrom_args_are_argv_not_shell_strings() {
    let config = Ch347Config {
        chip: Some("W25Q64JV-.Q".to_owned()),
        flashrom_path: None,
        spi_speed: Some(Ch347SpiSpeed::M15),
        expected_size: None,
    };
    let tool = FlashromTool::new(&config).unwrap();
    let args = tool.args_for(FlashromOperation::Read(std::path::Path::new(
        "/tmp/readback.bin",
    )));

    assert_eq!(
        args,
        vec![
            OsString::from("-p"),
            OsString::from("ch347_spi:spispeed=15M"),
            OsString::from("-c"),
            OsString::from("W25Q64JV-.Q"),
            OsString::from("-r"),
            OsString::from("/tmp/readback.bin"),
        ]
    );
}

#[test]
fn flashrom_args_allow_auto_detect_without_chip_name() {
    let config = Ch347Config {
        chip: None,
        flashrom_path: None,
        spi_speed: None,
        expected_size: None,
    };
    let tool = FlashromTool::new(&config).unwrap();
    let args = tool.args_for(FlashromOperation::Probe);

    assert_eq!(
        args,
        vec![
            OsString::from("-p"),
            OsString::from("ch347_spi:spispeed=15M"),
        ]
    );
}

#[test]
fn flashrom_probe_output_extracts_chip_size() {
    let output = std::process::Output {
        status: success_status(),
        stdout: b"CH347 SPI clock set to 15MHz.\nFound Winbond flash chip \"W25Q128.V\" (16384 kB, SPI) on ch347_spi.\nNo operations were specified.\n".to_vec(),
        stderr: b"libusb: info [darwin_detach_kernel_driver] no capture entitlements.\n".to_vec(),
    };

    let report = parse_flashrom_probe_output(&output);

    assert_eq!(report.chip_name.as_deref(), Some("W25Q128.V"));
    assert_eq!(report.size_bytes, Some(16 * 1024 * 1024));
}

#[test]
fn rejects_blank_chip_name_before_running_flashrom() {
    let mut device = Ch347Device::new(Ch347Config {
        chip: Some(" ".to_owned()),
        flashrom_path: None,
        spi_speed: None,
        expected_size: Some(4),
    });

    let error = device
        .write_save_image(&SaveImage::new(vec![1, 2, 3, 4]))
        .unwrap_err()
        .to_string();

    assert!(error.contains("chip name must not be empty"));
}

#[test]
fn write_rejects_wrong_size_before_running_flashrom() {
    let mut device = Ch347Device::new(Ch347Config {
        chip: Some("W25Q64JV-.Q".to_owned()),
        flashrom_path: Some("/dev/does-not-exist/flashrom".into()),
        spi_speed: None,
        expected_size: Some(4),
    });

    let error = device
        .write_save_image(&SaveImage::new(vec![1, 2, 3]))
        .unwrap_err()
        .to_string();

    assert!(error.contains("requires 4 bytes"));
}

#[cfg(unix)]
#[test]
fn fake_flashrom_read_write_and_fresh_verify_round_trip() {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir().join(format!(
        "framkey-ch347-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&dir).unwrap();
    let script = dir.join("fake-flashrom");
    let state = dir.join("fake-flashrom.state");

    std::fs::write(&state, b"old!").unwrap();
    std::fs::write(
        &script,
        b"#!/bin/sh\nstate=\"$0.state\"\nprev=\"\"\nfor arg in \"$@\"; do\n  if [ \"$prev\" = \"-r\" ]; then cp \"$state\" \"$arg\"; exit 0; fi\n  if [ \"$prev\" = \"-w\" ]; then cp \"$arg\" \"$state\"; exit 0; fi\n  prev=\"$arg\"\ndone\nexit 0\n",
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();

    let mut device = Ch347Device::new(Ch347Config {
        chip: None,
        flashrom_path: Some(script.clone()),
        spi_speed: None,
        expected_size: Some(4),
    });

    let initial = device.read_save_image().unwrap();
    assert_eq!(initial.as_bytes(), b"old!");

    let report = device
        .write_save_image_verified(&SaveImage::new(b"new!".to_vec()))
        .unwrap();
    assert_eq!(report.save_size, 4);
    assert!(report.exact_match);
    assert_eq!(report.input_blake3, report.readback_blake3);
    assert_eq!(std::fs::read(&state).unwrap(), b"new!");

    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
fn success_status() -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;

    std::process::ExitStatus::from_raw(0)
}

#[cfg(windows)]
fn success_status() -> std::process::ExitStatus {
    use std::os::windows::process::ExitStatusExt;

    std::process::ExitStatus::from_raw(0)
}
