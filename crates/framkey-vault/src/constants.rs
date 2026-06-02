pub const VAULT_MAGIC: [u8; 8] = *b"FRAMKEY\0";
pub const SAVE_MAGIC: [u8; 8] = *b"FRKSAVE\0";
pub const SAVE_SLOT_MAGIC: [u8; 8] = *b"FRKSLOT\0";
pub const VAULT_FORMAT_VERSION: u16 = 1;
pub const SAVE_IMAGE_FORMAT_VERSION: u16 = 1;
pub const SAVE_IMAGE_HEADER_LEN: usize = 128;
pub const SAVE_SLOT_HEADER_LEN: usize = 64;
pub const DEFAULT_FRAM_SAVE_IMAGE_SIZE: usize = 64 * 1024;
