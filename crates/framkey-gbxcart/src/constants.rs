use std::time::Duration;

pub(crate) const GBXCART_VID: u16 = 0x1A86;
pub(crate) const GBXCART_PID: u16 = 0x7523;
pub(crate) const DEFAULT_BAUD: u32 = 1_000_000;
pub(crate) const SERIAL_TIMEOUT: Duration = Duration::from_millis(1_000);
pub(crate) const MACOS_WRITE_DELAY: Duration = Duration::from_micros(1_250);

pub(crate) const CMD_OFW_FW_VER: u8 = 0x56;
pub(crate) const CMD_OFW_PCB_VER: u8 = 0x68;
pub(crate) const CMD_OFW_CART_PWR_ON: u8 = 0x2F;
pub(crate) const CMD_OFW_QUERY_CART_PWR: u8 = 0x5D;
pub(crate) const CMD_QUERY_FW_INFO: u8 = 0xA1;
pub(crate) const CMD_SET_MODE_AGB: u8 = 0xA2;
pub(crate) const CMD_SET_VOLTAGE_3_3V: u8 = 0xA4;
pub(crate) const CMD_SET_VARIABLE: u8 = 0xA6;
pub(crate) const CMD_SET_ADDR_AS_INPUTS: u8 = 0xA8;
pub(crate) const CMD_DISABLE_PULLUPS: u8 = 0xAC;
pub(crate) const CMD_AGB_CART_READ: u8 = 0xC1;
pub(crate) const CMD_AGB_CART_WRITE: u8 = 0xC2;
pub(crate) const CMD_AGB_CART_READ_SRAM: u8 = 0xC3;
pub(crate) const CMD_AGB_CART_WRITE_SRAM: u8 = 0xC4;
pub(crate) const CMD_AGB_CART_READ_EEPROM: u8 = 0xC5;
pub(crate) const CMD_AGB_CART_WRITE_EEPROM: u8 = 0xC6;

pub(crate) const VAR_ADDRESS: u32 = 0x00;
pub(crate) const VAR_TRANSFER_SIZE: u32 = 0x00;
pub(crate) const VAR_CART_MODE: u32 = 0x00;
pub(crate) const VAR_AGB_READ_METHOD: u32 = 0x0C;

pub(crate) const CART_MODE_AGB: u32 = 2;
pub(crate) const GBA_HEADER_SIZE: usize = 0x180;
pub(crate) const EEPROM64K_SIZE: usize = 8 * 1024;
pub(crate) const EEPROM64K_READ_BLOCK: usize = 0x100;
pub(crate) const EEPROM64K_WRITE_BLOCK: usize = 0x40;
pub(crate) const EEPROM_TRANSFER_CHUNK: usize = 0x40;
pub(crate) const SRAM_FRAM_256K_SIZE: usize = 32 * 1024;
pub(crate) const SRAM_FRAM_512KBIT_SIZE: usize = 64 * 1024;
pub(crate) const SRAM_FRAM_1MBIT_SIZE: usize = 128 * 1024;
pub(crate) const SRAM_FRAM_BANK_SIZE: usize = 0x10000;
pub(crate) const SRAM_FRAM_READ_BLOCK: usize = 0x40;
pub(crate) const SRAM_FRAM_WRITE_BLOCK: usize = 0x100;
pub(crate) const SRAM_FRAM_1MBIT_BANK_SELECT_ADDR: u32 = 0x0100_0000;
