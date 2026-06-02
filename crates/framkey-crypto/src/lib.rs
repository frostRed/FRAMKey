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
    fn aead_box_rejects_tampered_ciphertext() {
        let key = SecretBytes::new([7_u8; 32]);
        let mut boxed =
            AeadBox::encrypt_with_nonce(&key, [3_u8; 24], b"aad", b"plaintext").unwrap();
        boxed.ciphertext[0] ^= 0x01;

        assert!(boxed.decrypt(&key, b"aad").is_err());
    }

    #[test]
    fn aead_box_debug_redacts_sealed_material() {
        let key = SecretBytes::new([7_u8; 32]);
        let boxed = AeadBox::encrypt_with_nonce(&key, [3_u8; 24], b"aad", b"plaintext").unwrap();
        let debug = format!("{boxed:?}");

        assert!(debug.contains("AeadBox"));
        assert!(debug.contains("ciphertext_len"));
        assert!(!debug.contains("plaintext"));
        assert!(!debug.contains("[3, 3, 3"));
        assert!(!debug.contains(&format!("{:?}", boxed.ciphertext)));
    }

    #[test]
    fn hex_arrays_round_trip() {
        let bytes = [0xAB_u8; 32];
        assert_eq!(decode_hex_array::<32>(&encode_hex(&bytes)).unwrap(), bytes);
    }

    #[test]
    fn hex_array_rejects_wrong_length_and_bad_digits() {
        assert!(decode_hex_array::<2>("0xabc").is_err());
        assert!(decode_hex_array::<2>("0xzzzz").is_err());
    }

    #[test]
    fn secret_debug_never_prints_secret_bytes() {
        let secret = SecretBytes::new([0xAB; 32]);
        let debug = format!("{secret:?}");

        assert!(debug.contains("SecretBytes"));
        assert!(debug.contains("len"));
        assert!(!debug.contains("ab"));
        assert!(!debug.contains("171"));
    }
}
