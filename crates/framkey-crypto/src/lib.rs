mod aead;
mod hex;
mod random;
mod secret;

pub use aead::{AeadAlg, AeadBox};
pub use hex::{decode_hex_array, encode_hex};
pub use random::random_array;
pub use secret::SecretBytes;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aead_box_round_trips_with_aad() {
        let key = SecretBytes::new([7_u8; 32]);
        let boxed = AeadBox::encrypt_with_nonce(&key, [3_u8; 24], b"aad", b"plaintext").unwrap();

        assert_eq!(boxed.decrypt(&key, b"aad").unwrap(), b"plaintext");
        assert!(boxed.decrypt(&key, b"wrong").is_err());
    }

    #[test]
    fn hex_arrays_round_trip() {
        let bytes = [0xAB_u8; 32];
        assert_eq!(decode_hex_array::<32>(&encode_hex(&bytes)).unwrap(), bytes);
    }
}
