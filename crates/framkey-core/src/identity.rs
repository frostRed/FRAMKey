use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WalletId(pub [u8; 16]);

impl WalletId {
    pub const ZERO: Self = Self([0; 16]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyId(pub [u8; 16]);

impl PolicyId {
    pub const ZERO: Self = Self([0; 16]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Generation(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnixTimestamp(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WalletType {
    EvmEoaSecp256k1,
    Secp256k1SingleKey,
}

impl WalletType {
    pub fn uses_secp256k1_secret(self) -> bool {
        matches!(self, Self::EvmEoaSecp256k1 | Self::Secp256k1SingleKey)
    }

    pub fn supports_evm_eoa(self) -> bool {
        self.uses_secp256k1_secret()
    }

    pub fn supports_btc_single_key(self) -> bool {
        self.uses_secp256k1_secret()
    }
}
