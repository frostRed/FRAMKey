use std::{fmt, str::FromStr};

use framkey_core::{FramkeyError, Result};
use framkey_crypto::decode_hex_array;
use serde::{Deserialize, Serialize};

use crate::encode_prefixed_hex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmChainId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvmAddress(pub [u8; 20]);

impl fmt::Display for EvmAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("0x")?;
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for EvmAddress {
    type Err = FramkeyError;

    fn from_str(input: &str) -> Result<Self> {
        let bytes = decode_hex_array::<20>(input)?;
        Ok(Self(bytes))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum EvmSigningKind {
    PersonalSign,
    TypedDataV4,
    SendTransaction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmPersonalSignature {
    pub address: EvmAddress,
    pub message_hash: [u8; 32],
    pub signature: [u8; 65],
}

impl EvmPersonalSignature {
    pub fn signature_hex(&self) -> String {
        encode_prefixed_hex(&self.signature)
    }

    pub fn message_hash_hex(&self) -> String {
        encode_prefixed_hex(&self.message_hash)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmTypedDataSignature {
    pub address: EvmAddress,
    pub typed_data_hash: [u8; 32],
    pub signature: [u8; 65],
}

impl EvmTypedDataSignature {
    pub fn signature_hex(&self) -> String {
        encode_prefixed_hex(&self.signature)
    }

    pub fn typed_data_hash_hex(&self) -> String {
        encode_prefixed_hex(&self.typed_data_hash)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmTransaction {
    pub chain_id: u64,
    pub nonce: String,
    pub gas_limit: String,
    pub to: Option<String>,
    pub value: String,
    pub data: String,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvmTransactionKind {
    Legacy,
    Eip1559,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmSignedTransaction {
    pub address: EvmAddress,
    pub kind: EvmTransactionKind,
    pub transaction_hash: [u8; 32],
    pub raw_transaction: Vec<u8>,
}

impl EvmSignedTransaction {
    pub fn raw_transaction_hex(&self) -> String {
        encode_prefixed_hex(&self.raw_transaction)
    }

    pub fn transaction_hash_hex(&self) -> String {
        encode_prefixed_hex(&self.transaction_hash)
    }
}
