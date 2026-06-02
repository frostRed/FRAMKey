use super::*;
use framkey_crypto::{SecretBytes, decode_hex_array};

#[test]
fn address_derives_from_known_ethereum_private_key() {
    let secret = SecretBytes::new(
        decode_hex_array::<32>("4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318")
            .unwrap(),
    );

    assert_eq!(
        address_from_secret(&secret).unwrap().to_string(),
        "0x2c7536e3605d9c16a7a3d7b1898e529396a65c23"
    );
}

#[test]
fn personal_sign_recovers_signer() {
    let secret = SecretBytes::new(
        decode_hex_array::<32>("4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318")
            .unwrap(),
    );
    let message = b"FRAMKey personal_sign smoke";

    let signed = personal_sign(&secret, message).unwrap();
    let recovered = recover_personal_signer(message, &signed.signature).unwrap();

    assert_eq!(recovered, signed.address);
    assert_eq!(signed.signature.len(), 65);
    assert!(matches!(signed.signature[64], 27 | 28));
}

#[test]
fn signs_eip155_legacy_transaction_vector() {
    let secret = SecretBytes::new(
        decode_hex_array::<32>("4646464646464646464646464646464646464646464646464646464646464646")
            .unwrap(),
    );
    let signed = sign_transaction(
        &secret,
        &EvmTransaction {
            chain_id: 1,
            nonce: "0x9".to_owned(),
            gas_limit: "0x5208".to_owned(),
            to: Some("0x3535353535353535353535353535353535353535".to_owned()),
            value: "0xde0b6b3a7640000".to_owned(),
            data: "0x".to_owned(),
            gas_price: Some("0x4a817c800".to_owned()),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
        },
    )
    .unwrap();

    assert_eq!(signed.kind, EvmTransactionKind::Legacy);
    assert_eq!(
        signed.raw_transaction_hex(),
        "0xf86c098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a76400008025a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83"
    );
}

#[test]
fn signs_basic_eip1559_transaction() {
    let secret = SecretBytes::new(
        decode_hex_array::<32>("4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318")
            .unwrap(),
    );
    let signed = sign_transaction(
        &secret,
        &EvmTransaction {
            chain_id: 1,
            nonce: "0x0".to_owned(),
            gas_limit: "0x5208".to_owned(),
            to: Some("0x0000000000000000000000000000000000000001".to_owned()),
            value: "0x0".to_owned(),
            data: "0x".to_owned(),
            gas_price: None,
            max_fee_per_gas: Some("0x3b9aca00".to_owned()),
            max_priority_fee_per_gas: Some("0x3b9aca00".to_owned()),
        },
    )
    .unwrap();

    assert_eq!(signed.kind, EvmTransactionKind::Eip1559);
    assert_eq!(
        signed.address.to_string(),
        "0x2c7536e3605d9c16a7a3d7b1898e529396a65c23"
    );
    assert!(signed.raw_transaction_hex().starts_with("0x02"));
    assert_eq!(signed.transaction_hash.len(), 32);
}

#[test]
fn signs_erc20_permit_typed_data_and_recovers_signer() {
    let secret = SecretBytes::new(
        decode_hex_array::<32>("4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318")
            .unwrap(),
    );
    let typed_data = serde_json::json!({
        "domain": {
            "name": "USD Coin",
            "version": "2",
            "chainId": 1,
            "verifyingContract": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        },
        "primaryType": "Permit",
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "Permit": [
                {"name": "owner", "type": "address"},
                {"name": "spender", "type": "address"},
                {"name": "value", "type": "uint256"},
                {"name": "nonce", "type": "uint256"},
                {"name": "deadline", "type": "uint256"}
            ]
        },
        "message": {
            "owner": "0x2c7536e3605d9c16a7a3d7b1898e529396a65c23",
            "spender": "0x000000000022d473030f116ddee9f6b43ac78ba3",
            "value": "1000000",
            "nonce": "0",
            "deadline": "1900000000"
        }
    });

    let digest = typed_data_v4_hash(&typed_data).unwrap();
    let signed = sign_typed_data_v4(&secret, &typed_data).unwrap();
    let recovered = recover_typed_data_signer(&typed_data, &signed.signature).unwrap();

    assert_eq!(signed.typed_data_hash, digest);
    assert_eq!(signed.address, recovered);
    assert_eq!(
        signed.address.to_string(),
        "0x2c7536e3605d9c16a7a3d7b1898e529396a65c23"
    );
    assert!(matches!(signed.signature[64], 27 | 28));
}

#[test]
fn signs_permit2_batch_typed_data_with_array_fields() {
    let secret = SecretBytes::new(
        decode_hex_array::<32>("4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318")
            .unwrap(),
    );
    let typed_data = serde_json::json!({
        "domain": {
            "name": "Permit2",
            "chainId": 1,
            "verifyingContract": "0x000000000022d473030f116ddee9f6b43ac78ba3"
        },
        "primaryType": "PermitBatch",
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "PermitDetails": [
                {"name": "token", "type": "address"},
                {"name": "amount", "type": "uint160"},
                {"name": "expiration", "type": "uint48"},
                {"name": "nonce", "type": "uint48"}
            ],
            "PermitBatch": [
                {"name": "details", "type": "PermitDetails[]"},
                {"name": "spender", "type": "address"},
                {"name": "sigDeadline", "type": "uint256"}
            ]
        },
        "message": {
            "details": [
                {
                    "token": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                    "amount": "1461501637330902918203684832716283019655932542975",
                    "expiration": "1900000000",
                    "nonce": "9"
                },
                {
                    "token": "0x0000000000000000000000000000000000000001",
                    "amount": "1",
                    "expiration": "1900000100",
                    "nonce": "10"
                }
            ],
            "spender": "0x3333333333333333333333333333333333333333",
            "sigDeadline": "1900000200"
        }
    });

    let signed = sign_typed_data_v4(&secret, &typed_data).unwrap();
    let recovered = recover_typed_data_signer(&typed_data, &signed.signature).unwrap();

    assert_eq!(signed.address, recovered);
    assert_eq!(signed.typed_data_hash.len(), 32);
}

#[test]
fn rejects_typed_data_uint_overflow() {
    let typed_data = serde_json::json!({
        "domain": {
            "name": "Permit2",
            "chainId": 1,
            "verifyingContract": "0x000000000022d473030f116ddee9f6b43ac78ba3"
        },
        "primaryType": "PermitSingle",
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "PermitDetails": [
                {"name": "token", "type": "address"},
                {"name": "amount", "type": "uint160"},
                {"name": "expiration", "type": "uint48"},
                {"name": "nonce", "type": "uint48"}
            ],
            "PermitSingle": [
                {"name": "details", "type": "PermitDetails"},
                {"name": "spender", "type": "address"},
                {"name": "sigDeadline", "type": "uint256"}
            ]
        },
        "message": {
            "details": {
                "token": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                "amount": "1",
                "expiration": "281474976710656",
                "nonce": "1"
            },
            "spender": "0x3333333333333333333333333333333333333333",
            "sigDeadline": "1900000200"
        }
    });

    assert!(typed_data_v4_hash(&typed_data).is_err());
}

#[test]
fn rejects_invalid_private_keys() {
    assert!(validate_private_key_bytes(&[0_u8; 32]).is_err());
}
