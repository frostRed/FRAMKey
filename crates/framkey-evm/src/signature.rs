use framkey_core::{FramkeyError, Result};
use k256::ecdsa::{RecoveryId, Signature};

pub(crate) fn eth_signature_bytes(signature: Signature, recovery_id: RecoveryId) -> [u8; 65] {
    let mut signature_bytes = [0_u8; 65];
    signature_bytes[..64].copy_from_slice(signature.to_bytes().as_slice());
    signature_bytes[64] = recovery_id.to_byte() + 27;
    signature_bytes
}

pub(crate) fn recovery_id_from_eth_v(v: u8) -> Result<RecoveryId> {
    let normalized = match v {
        27 | 28 => v - 27,
        0 | 1 => v,
        _ => {
            return Err(FramkeyError::invalid_data(
                "EVM signature recovery id must be 0/1 or 27/28",
            ));
        }
    };

    RecoveryId::from_byte(normalized)
        .ok_or_else(|| FramkeyError::invalid_data("invalid EVM signature recovery id"))
}
