mod encoding;
mod keys;
mod personal_sign;
mod signature;
mod transaction;
mod typed_data;
mod types;

pub use encoding::{decode_signature_hex, encode_prefixed_hex};
pub use keys::{address_from_secret, validate_private_key_bytes};
pub use personal_sign::{personal_sign, recover_personal_signer};
pub use transaction::{sign_transaction, validate_transaction};
pub use typed_data::{recover_typed_data_signer, sign_typed_data_v4, typed_data_v4_hash};
pub use types::{
    EvmAddress, EvmChainId, EvmPersonalSignature, EvmSignedTransaction, EvmSigningKind,
    EvmTransaction, EvmTransactionKind, EvmTypedDataSignature,
};

#[cfg(test)]
mod tests;
