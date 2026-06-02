use framkey_core::{FramkeyError, Result};
use framkey_crypto::SecretBytes;
use k256::ecdsa::{RecoveryId, Signature, SigningKey};
use sha3::{Digest, Keccak256};

use crate::{
    EvmSignedTransaction, EvmTransaction, EvmTransactionKind,
    encoding::{
        keccak256, minimal_integer_bytes, parse_address_bytes, parse_data_bytes,
        parse_quantity_bytes,
    },
    keys::{address_from_verifying_key, signing_key_from_secret},
};

pub fn validate_transaction(transaction: &EvmTransaction) -> Result<()> {
    ParsedTransaction::parse(transaction).map(|_| ())
}

pub fn sign_transaction(
    secret: &SecretBytes<32>,
    transaction: &EvmTransaction,
) -> Result<EvmSignedTransaction> {
    let signing_key = signing_key_from_secret(secret.expose())?;
    let parsed = ParsedTransaction::parse(transaction)?;
    let (kind, signed_payload) = parsed.signing_payload(&signing_key)?;
    let transaction_hash = keccak256(&signed_payload);

    Ok(EvmSignedTransaction {
        address: address_from_verifying_key(signing_key.verifying_key()),
        kind,
        transaction_hash,
        raw_transaction: signed_payload,
    })
}

#[derive(Debug, Clone)]
struct ParsedTransaction {
    chain_id: u64,
    nonce: Vec<u8>,
    gas_limit: Vec<u8>,
    to: Option<[u8; 20]>,
    value: Vec<u8>,
    data: Vec<u8>,
    gas_price: Option<Vec<u8>>,
    max_fee_per_gas: Option<Vec<u8>>,
    max_priority_fee_per_gas: Option<Vec<u8>>,
}

impl ParsedTransaction {
    fn parse(transaction: &EvmTransaction) -> Result<Self> {
        if transaction.chain_id == 0 {
            return Err(FramkeyError::invalid_data(
                "transaction chain id must be nonzero",
            ));
        }
        if transaction.gas_price.is_some()
            && (transaction.max_fee_per_gas.is_some()
                || transaction.max_priority_fee_per_gas.is_some())
        {
            return Err(FramkeyError::invalid_data(
                "transaction cannot mix gasPrice with EIP-1559 fee fields",
            ));
        }

        Ok(Self {
            chain_id: transaction.chain_id,
            nonce: parse_quantity_bytes(&transaction.nonce, "transaction nonce")?,
            gas_limit: parse_quantity_bytes(&transaction.gas_limit, "transaction gas limit")?,
            to: transaction
                .to
                .as_deref()
                .map(|to| parse_address_bytes(to, "transaction to"))
                .transpose()?,
            value: parse_quantity_bytes(&transaction.value, "transaction value")?,
            data: parse_data_bytes(&transaction.data, "transaction data")?,
            gas_price: transaction
                .gas_price
                .as_deref()
                .map(|value| parse_quantity_bytes(value, "transaction gasPrice"))
                .transpose()?,
            max_fee_per_gas: transaction
                .max_fee_per_gas
                .as_deref()
                .map(|value| parse_quantity_bytes(value, "transaction maxFeePerGas"))
                .transpose()?,
            max_priority_fee_per_gas: transaction
                .max_priority_fee_per_gas
                .as_deref()
                .map(|value| parse_quantity_bytes(value, "transaction maxPriorityFeePerGas"))
                .transpose()?,
        })
    }

    fn signing_payload(&self, signing_key: &SigningKey) -> Result<(EvmTransactionKind, Vec<u8>)> {
        if self.max_fee_per_gas.is_some() || self.max_priority_fee_per_gas.is_some() {
            self.eip1559_payload(signing_key)
        } else {
            self.legacy_payload(signing_key)
        }
    }

    fn legacy_payload(&self, signing_key: &SigningKey) -> Result<(EvmTransactionKind, Vec<u8>)> {
        let gas_price = self
            .gas_price
            .as_ref()
            .ok_or_else(|| FramkeyError::invalid_data("legacy transaction requires gasPrice"))?;
        let unsigned = rlp_encode_list(&[
            rlp_encode_integer(&self.nonce),
            rlp_encode_integer(gas_price),
            rlp_encode_integer(&self.gas_limit),
            rlp_encode_address(self.to.as_ref()),
            rlp_encode_integer(&self.value),
            rlp_encode_bytes(&self.data),
            rlp_encode_u64(self.chain_id),
            rlp_encode_integer(&[]),
            rlp_encode_integer(&[]),
        ]);
        let (signature, recovery_id) = sign_payload(signing_key, &unsigned)?;
        let signature = signature.to_bytes();
        let r = minimal_integer_bytes(&signature[..32]);
        let s = minimal_integer_bytes(&signature[32..64]);
        let v = self
            .chain_id
            .checked_mul(2)
            .and_then(|value| value.checked_add(35 + u64::from(recovery_id.to_byte())))
            .ok_or_else(|| FramkeyError::invalid_data("transaction chain id is too large"))?;
        let signed = rlp_encode_list(&[
            rlp_encode_integer(&self.nonce),
            rlp_encode_integer(gas_price),
            rlp_encode_integer(&self.gas_limit),
            rlp_encode_address(self.to.as_ref()),
            rlp_encode_integer(&self.value),
            rlp_encode_bytes(&self.data),
            rlp_encode_u64(v),
            rlp_encode_integer(&r),
            rlp_encode_integer(&s),
        ]);
        Ok((EvmTransactionKind::Legacy, signed))
    }

    fn eip1559_payload(&self, signing_key: &SigningKey) -> Result<(EvmTransactionKind, Vec<u8>)> {
        let max_priority_fee_per_gas = self.max_priority_fee_per_gas.as_ref().ok_or_else(|| {
            FramkeyError::invalid_data("EIP-1559 transaction requires maxPriorityFeePerGas")
        })?;
        let max_fee_per_gas = self.max_fee_per_gas.as_ref().ok_or_else(|| {
            FramkeyError::invalid_data("EIP-1559 transaction requires maxFeePerGas")
        })?;
        let access_list = rlp_encode_list(&[]);
        let unsigned_body = rlp_encode_list(&[
            rlp_encode_u64(self.chain_id),
            rlp_encode_integer(&self.nonce),
            rlp_encode_integer(max_priority_fee_per_gas),
            rlp_encode_integer(max_fee_per_gas),
            rlp_encode_integer(&self.gas_limit),
            rlp_encode_address(self.to.as_ref()),
            rlp_encode_integer(&self.value),
            rlp_encode_bytes(&self.data),
            access_list.clone(),
        ]);
        let mut unsigned = Vec::with_capacity(1 + unsigned_body.len());
        unsigned.push(0x02);
        unsigned.extend_from_slice(&unsigned_body);

        let (signature, recovery_id) = sign_payload(signing_key, &unsigned)?;
        let signature = signature.to_bytes();
        let r = minimal_integer_bytes(&signature[..32]);
        let s = minimal_integer_bytes(&signature[32..64]);
        let signed_body = rlp_encode_list(&[
            rlp_encode_u64(self.chain_id),
            rlp_encode_integer(&self.nonce),
            rlp_encode_integer(max_priority_fee_per_gas),
            rlp_encode_integer(max_fee_per_gas),
            rlp_encode_integer(&self.gas_limit),
            rlp_encode_address(self.to.as_ref()),
            rlp_encode_integer(&self.value),
            rlp_encode_bytes(&self.data),
            access_list,
            rlp_encode_u64(u64::from(recovery_id.to_byte())),
            rlp_encode_integer(&r),
            rlp_encode_integer(&s),
        ]);
        let mut signed = Vec::with_capacity(1 + signed_body.len());
        signed.push(0x02);
        signed.extend_from_slice(&signed_body);
        Ok((EvmTransactionKind::Eip1559, signed))
    }
}

fn sign_payload(signing_key: &SigningKey, payload: &[u8]) -> Result<(Signature, RecoveryId)> {
    signing_key
        .sign_digest_recoverable(Keccak256::new_with_prefix(payload))
        .map_err(|_| FramkeyError::invalid_data("EVM transaction signing failed"))
}

fn rlp_encode_address(address: Option<&[u8; 20]>) -> Vec<u8> {
    match address {
        Some(address) => rlp_encode_bytes(address),
        None => rlp_encode_bytes(&[]),
    }
}

fn rlp_encode_u64(value: u64) -> Vec<u8> {
    rlp_encode_integer(&minimal_integer_bytes(&value.to_be_bytes()))
}

fn rlp_encode_integer(bytes: &[u8]) -> Vec<u8> {
    rlp_encode_bytes(bytes)
}

fn rlp_encode_bytes(bytes: &[u8]) -> Vec<u8> {
    if bytes.len() == 1 && bytes[0] < 0x80 {
        return vec![bytes[0]];
    }
    let mut output = rlp_length_prefix(0x80, bytes.len());
    output.extend_from_slice(bytes);
    output
}

fn rlp_encode_list(items: &[Vec<u8>]) -> Vec<u8> {
    let payload_len = items.iter().map(Vec::len).sum();
    let mut output = rlp_length_prefix(0xc0, payload_len);
    for item in items {
        output.extend_from_slice(item);
    }
    output
}

fn rlp_length_prefix(offset: u8, len: usize) -> Vec<u8> {
    if len < 56 {
        return vec![offset + len as u8];
    }
    let len_bytes = minimal_integer_bytes(&len.to_be_bytes());
    let mut output = Vec::with_capacity(1 + len_bytes.len());
    output.push(offset + 55 + len_bytes.len() as u8);
    output.extend_from_slice(&len_bytes);
    output
}
