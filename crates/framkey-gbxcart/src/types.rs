use std::fmt;

use framkey_core::{FramkeyError, Result};

use crate::constants::{
    EEPROM64K_SIZE, SRAM_FRAM_1MBIT_SIZE, SRAM_FRAM_256K_SIZE, SRAM_FRAM_512KBIT_SIZE,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GbxCartConfig {
    pub port_hint: Option<String>,
    pub expected_save_size: Option<usize>,
    pub save_type: Option<GbaSaveType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GbaSaveType {
    Eeprom64k,
    SramFram256k,
    SramFram512Kbit,
    SramFram1Mbit,
}

impl GbaSaveType {
    pub fn save_size(self) -> usize {
        match self {
            Self::Eeprom64k => EEPROM64K_SIZE,
            Self::SramFram256k => SRAM_FRAM_256K_SIZE,
            Self::SramFram512Kbit => SRAM_FRAM_512KBIT_SIZE,
            Self::SramFram1Mbit => SRAM_FRAM_1MBIT_SIZE,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Eeprom64k => "GBA EEPROM 64K (8 KiB)",
            Self::SramFram256k => "GBA SRAM/FRAM 256K (32 KiB)",
            Self::SramFram512Kbit => "GBA SRAM/FRAM 512 Kbit (64 KiB)",
            Self::SramFram1Mbit => "GBA SRAM/FRAM 1 Mbit (128 KiB)",
        }
    }
}

impl fmt::Display for GbaSaveType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GbaHeader {
    pub title: String,
    pub game_code: String,
    pub maker_code: String,
    pub revision: u8,
    pub header_checksum_valid: bool,
}

impl GbaHeader {
    pub fn parse(header: &[u8]) -> Result<Self> {
        if header.len() < 0xBE {
            return Err(FramkeyError::invalid_data(
                "GBA header must contain at least 0xBE bytes",
            ));
        }

        let checksum = header[0xA0..0xBD]
            .iter()
            .fold(0_u8, |acc, byte| acc.wrapping_sub(*byte))
            .wrapping_sub(0x19);

        Ok(Self {
            title: ascii_field(&header[0xA0..0xAC]),
            game_code: ascii_field(&header[0xAC..0xB0]),
            maker_code: ascii_field(&header[0xB0..0xB2]),
            revision: header[0xBC],
            header_checksum_valid: checksum == header[0xBD],
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FirmwareInfo {
    pub(crate) cfw_id: char,
    pub(crate) fw_version: u16,
    pub(crate) pcb_version: u8,
    pub(crate) firmware_timestamp: u32,
    pub(crate) official_fw_version: u8,
}

impl FirmwareInfo {
    pub(crate) fn pcb_label(&self) -> &'static str {
        match self.pcb_version {
            4 => "v1.3",
            5 => "v1.4",
            6 => "v1.4a/b/c",
            101 => "Mini v1.0d",
            _ => "unknown PCB",
        }
    }

    pub(crate) fn firmware_label(&self) -> String {
        if matches!(self.pcb_version, 5 | 6 | 101) {
            format!(
                "R{}+{}{}",
                self.official_fw_version, self.cfw_id, self.fw_version
            )
        } else {
            format!("{}{}", self.cfw_id, self.fw_version)
        }
    }
}

fn ascii_field(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    bytes[..end]
        .iter()
        .map(|byte| match *byte {
            b' '..=b'~' => char::from(*byte),
            _ => '?',
        })
        .collect::<String>()
        .trim()
        .to_owned()
}
