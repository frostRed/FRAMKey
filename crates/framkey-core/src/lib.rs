mod error;
mod identity;

pub use error::{FramkeyError, Result};
pub use identity::{Generation, PolicyId, UnixTimestamp, WalletId, WalletType};

#[cfg(test)]
mod tests {
    use super::{FramkeyError, Generation, PolicyId, UnixTimestamp, WalletId, WalletType};

    #[test]
    fn wallet_type_uses_stable_snake_case_wire_name() {
        let encoded = serde_json::to_string(&WalletType::EvmEoaSecp256k1).unwrap();
        assert_eq!(encoded, r#""evm_eoa_secp256k1""#);

        let decoded: WalletType = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, WalletType::EvmEoaSecp256k1);
    }

    #[test]
    fn identity_newtypes_serialize_without_field_names() {
        assert_eq!(
            serde_json::to_string(&WalletId([1; 16])).unwrap(),
            "[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]"
        );
        assert_eq!(
            serde_json::to_string(&PolicyId([2; 16])).unwrap(),
            "[2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2]"
        );
        assert_eq!(serde_json::to_string(&Generation(7)).unwrap(), "7");
        assert_eq!(serde_json::to_string(&UnixTimestamp(42)).unwrap(), "42");
    }

    #[test]
    fn error_display_preserves_actionable_category() {
        assert_eq!(
            FramkeyError::invalid_data("bad vault").to_string(),
            "invalid data: bad vault"
        );
        assert_eq!(
            FramkeyError::unsupported("unsupported format").to_string(),
            "unsupported operation: unsupported format"
        );
    }
}
