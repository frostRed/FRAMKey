use super::*;
use crate::{
    constants::{GBA_HEADER_SIZE, SRAM_FRAM_1MBIT_SIZE, SRAM_FRAM_BANK_SIZE},
    transport::{candidate_ports, sram_fram_1mbit_banks_match},
};
use framkey_device::{SaveImage, VaultDevice};

#[test]
fn gba_save_types_report_sizes() {
    assert_eq!(GbaSaveType::Eeprom64k.save_size(), 8192);
    assert_eq!(GbaSaveType::SramFram256k.save_size(), 32768);
    assert_eq!(GbaSaveType::SramFram512Kbit.save_size(), 65536);
    assert_eq!(GbaSaveType::SramFram1Mbit.save_size(), 131072);
}

#[test]
fn detects_mirrored_1mbit_banks() {
    let mut image = vec![0xFF; SRAM_FRAM_1MBIT_SIZE];
    image[0x7000] = 0xA5;
    image[SRAM_FRAM_BANK_SIZE + 0x7000] = 0xA5;
    assert!(sram_fram_1mbit_banks_match(&image));

    image[SRAM_FRAM_BANK_SIZE + 0x7000] = 0x5A;
    assert!(!sram_fram_1mbit_banks_match(&image));
    assert!(!sram_fram_1mbit_banks_match(&image[..SRAM_FRAM_BANK_SIZE]));
}

#[test]
fn write_fails_before_connecting_when_save_size_is_wrong() {
    let mut device = GbxCartDevice::new(GbxCartConfig {
        port_hint: Some("/dev/does-not-exist".to_owned()),
        expected_save_size: None,
        save_type: Some(GbaSaveType::SramFram256k),
    });
    let image = SaveImage::new(vec![0; 1]);
    let error = device.write_save_image(&image).unwrap_err().to_string();

    assert!(error.contains("requires 32768 bytes"));
}

#[test]
fn gba_header_parser_extracts_fields_and_validates_checksum() {
    let mut header = vec![0_u8; GBA_HEADER_SIZE];
    header[0xA0..0xAC].copy_from_slice(b"MARIO&LUIGIJ");
    header[0xAC..0xB0].copy_from_slice(b"A88J");
    header[0xB0..0xB2].copy_from_slice(b"01");
    header[0xB2] = 0x96;
    header[0xBC] = 0;
    header[0xBD] = header[0xA0..0xBD]
        .iter()
        .fold(0_u8, |acc, byte| acc.wrapping_sub(*byte))
        .wrapping_sub(0x19);

    let parsed = GbaHeader::parse(&header).unwrap();

    assert_eq!(parsed.title, "MARIO&LUIGIJ");
    assert_eq!(parsed.game_code, "A88J");
    assert_eq!(parsed.maker_code, "01");
    assert_eq!(parsed.revision, 0);
    assert!(parsed.header_checksum_valid);
}

#[test]
fn explicit_port_hint_skips_port_enumeration() {
    assert_eq!(
        candidate_ports(Some("/dev/cu.usbserial-test")).unwrap(),
        vec!["/dev/cu.usbserial-test".to_owned()]
    );
}

#[test]
fn empty_explicit_port_hint_is_rejected() {
    let error = candidate_ports(Some("  ")).unwrap_err();
    assert!(error.to_string().contains("port hint must not be empty"));
}
