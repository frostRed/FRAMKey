use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use framkey_crypto::encode_hex;

use crate::{
    CH347_HELPER_OPERATION, CH347_HELPER_READ_OPERATION, Ch347HelperReadRequest,
    Ch347HelperWriteRequest, MAX_CH347_HELPER_IMAGE_BYTES, PHYSICAL_BACKUP_ROM_HEADER_BYTES,
    execute_read_request, execute_write_request, extract_physical_backup_from_rom_image,
    parse_spi_speed, prepare_physical_backup_rom_image,
};

#[test]
fn request_rejects_relative_flashrom_path_for_privileged_execution() {
    let request = Ch347HelperWriteRequest {
        operation: CH347_HELPER_OPERATION.to_owned(),
        input_path: "/tmp/backup.bin".into(),
        flashrom_path: "flashrom".into(),
        chip: None,
        spispeed: None,
        expected_size: 4,
        expected_blake3: "0".repeat(64),
    };

    let error = request.validate().unwrap_err().to_string();

    assert!(error.contains("flashrom path must be an absolute path"));
}

#[test]
fn request_rejects_unbounded_image_size() {
    let request = Ch347HelperWriteRequest {
        operation: CH347_HELPER_OPERATION.to_owned(),
        input_path: "/tmp/backup.bin".into(),
        flashrom_path: "/opt/homebrew/sbin/flashrom".into(),
        chip: None,
        spispeed: None,
        expected_size: MAX_CH347_HELPER_IMAGE_BYTES + 1,
        expected_blake3: "0".repeat(64),
    };

    let error = request.validate().unwrap_err().to_string();

    assert!(error.contains("input file size must be between"));
}

#[test]
fn request_accepts_512_mib_input_limit() {
    let request = Ch347HelperWriteRequest {
        operation: CH347_HELPER_OPERATION.to_owned(),
        input_path: "/tmp/backup.bin".into(),
        flashrom_path: "/opt/homebrew/sbin/flashrom".into(),
        chip: None,
        spispeed: None,
        expected_size: MAX_CH347_HELPER_IMAGE_BYTES,
        expected_blake3: "0".repeat(64),
    };

    request.validate().unwrap();
}

#[test]
fn parses_helper_spi_speed_values() {
    assert_eq!(
        parse_spi_speed(Some("937.5k"))
            .unwrap()
            .unwrap()
            .as_flashrom_value(),
        "937.5K"
    );
    assert!(parse_spi_speed(Some("fast")).is_err());
}

#[test]
fn error_response_keeps_root_cause_context() {
    let error = anyhow::anyhow!("flashrom CH347 write failed with status 1")
        .context("CH347 helper write/readback verification failed");
    let response = crate::error_response(&error);
    let crate::Ch347HelperResponse::Error { error } = response else {
        panic!("expected helper error response");
    };

    assert!(
        error
            .message
            .contains("CH347 helper write/readback verification failed")
    );
    assert!(
        error
            .message
            .contains("flashrom CH347 write failed with status 1")
    );
}

#[test]
fn smaller_backup_payload_is_wrapped_as_full_rom_image() {
    let prepared = prepare_physical_backup_rom_image(b"backup-bundle", Some(8 * 1024)).unwrap();

    assert_eq!(prepared.payload_size, b"backup-bundle".len());
    assert_eq!(prepared.image.len(), 8 * 1024);
    assert_eq!(prepared.storage_format, "framkey_physical_backup_v1");
    assert_eq!(&prepared.image[..16], b"FRAMKEYPHYSBK01!");
    assert_eq!(
        &prepared.image[PHYSICAL_BACKUP_ROM_HEADER_BYTES
            ..PHYSICAL_BACKUP_ROM_HEADER_BYTES + b"backup-bundle".len()],
        b"backup-bundle"
    );
    assert!(
        prepared.image[PHYSICAL_BACKUP_ROM_HEADER_BYTES + b"backup-bundle".len()..]
            .iter()
            .all(|byte| *byte == 0xff)
    );
}

#[test]
fn physical_backup_rom_image_extracts_original_payload() {
    let prepared = prepare_physical_backup_rom_image(b"backup-bundle", Some(8 * 1024)).unwrap();
    let extracted = extract_physical_backup_from_rom_image(&prepared.image).unwrap();

    assert_eq!(extracted.bytes, b"backup-bundle");
    assert_eq!(extracted.payload_size, b"backup-bundle".len());
    assert_eq!(extracted.storage_format, "framkey_physical_backup_v1");
    assert!(extracted.layout_parsed);
    assert!(extracted.verified);
}

#[cfg(unix)]
#[test]
fn fake_flashrom_write_round_trip_returns_privileged_metadata() {
    use std::os::unix::fs::PermissionsExt;

    let dir = unique_temp_dir("ch347-helper-round-trip");
    fs::create_dir_all(&dir).unwrap();
    let script = dir.join("fake-flashrom");
    let state = dir.join("fake-flashrom.state");
    let backup = dir.join("backup.bin");
    fs::write(&backup, b"new!").unwrap();
    fs::write(&state, vec![0xff; 8 * 1024]).unwrap();
    fs::write(
        &script,
        b"#!/bin/sh\nstate=\"$0.state\"\nprev=\"\"\nfor arg in \"$@\"; do\n  if [ \"$prev\" = \"-r\" ]; then cp \"$state\" \"$arg\"; exit 0; fi\n  if [ \"$prev\" = \"-w\" ]; then cp \"$arg\" \"$state\"; exit 0; fi\n  prev=\"$arg\"\ndone\necho 'Found Test flash chip \"T25Q64\" (8 kB, SPI) on ch347_spi.'\nexit 0\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&script, permissions).unwrap();

    let result = execute_write_request(Ch347HelperWriteRequest {
        operation: CH347_HELPER_OPERATION.to_owned(),
        input_path: backup.clone(),
        flashrom_path: script.clone(),
        chip: None,
        spispeed: Some("15M".to_owned()),
        expected_size: 4,
        expected_blake3: encode_hex(blake3::hash(b"new!").as_bytes()),
    })
    .unwrap();

    assert!(result.privileged);
    assert_eq!(result.helper_process, "framkey-ch347-helper");
    assert_eq!(result.storage_format, "framkey_physical_backup_v1");
    assert_eq!(result.payload_size, 4);
    assert_eq!(result.rom_image_size, 8 * 1024);
    assert_eq!(result.backup_blake3, result.readback_blake3);
    assert_eq!(result.write_count, 1);
    assert_eq!(result.readback_count, 1);
    let written = fs::read(&state).unwrap();
    assert_eq!(written.len(), 8 * 1024);
    assert_eq!(
        &written[PHYSICAL_BACKUP_ROM_HEADER_BYTES..PHYSICAL_BACKUP_ROM_HEADER_BYTES + 4],
        b"new!"
    );

    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn fake_flashrom_read_extracts_backup_payload_to_output_dir() {
    use std::os::unix::fs::PermissionsExt;

    let dir = unique_temp_dir("ch347-helper-read");
    fs::create_dir_all(&dir).unwrap();
    let script = dir.join("fake-flashrom");
    let state = dir.join("fake-flashrom.state");
    let output_dir = dir.join("out");
    fs::create_dir_all(&output_dir).unwrap();
    let prepared = prepare_physical_backup_rom_image(b"backup-bundle", Some(8 * 1024)).unwrap();
    fs::write(&state, &prepared.image).unwrap();
    fs::write(
        &script,
        b"#!/bin/sh\nstate=\"$0.state\"\nprev=\"\"\nfor arg in \"$@\"; do\n  if [ \"$prev\" = \"-r\" ]; then cp \"$state\" \"$arg\"; exit 0; fi\n  if [ \"$prev\" = \"-w\" ]; then cp \"$arg\" \"$state\"; exit 0; fi\n  prev=\"$arg\"\ndone\necho 'Found Test flash chip \"T25Q64\" (8 kB, SPI) on ch347_spi.'\nexit 0\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&script, permissions).unwrap();

    let result = execute_read_request(Ch347HelperReadRequest {
        operation: CH347_HELPER_READ_OPERATION.to_owned(),
        output_dir: output_dir.clone(),
        flashrom_path: script.clone(),
        chip: None,
        spispeed: Some("15M".to_owned()),
    })
    .unwrap();

    assert!(result.privileged);
    assert_eq!(result.helper_process, "framkey-ch347-helper");
    assert_eq!(result.storage_format, "framkey_physical_backup_v1");
    assert_eq!(result.output_kind, "physical_backup_payload");
    assert_eq!(result.payload_size, b"backup-bundle".len());
    assert_eq!(result.rom_image_size, 8 * 1024);
    assert_eq!(result.read_count, 1);
    assert!(result.layout_parsed);
    assert!(result.verified);
    assert_eq!(fs::read(result.output_path).unwrap(), b"backup-bundle");

    fs::remove_dir_all(dir).unwrap();
}

fn unique_temp_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "framkey-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
