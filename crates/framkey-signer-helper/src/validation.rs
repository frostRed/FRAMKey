use anyhow::Result;
use framkey_btc::{BtcNetwork, validate_p2wpkh_address};
use framkey_core::FramkeyError;
use framkey_evm::{
    EvmAddress, EvmTransaction, EvmTransactionKind, typed_data_v4_hash, validate_transaction,
};
use framkey_ipc::{
    MAX_SIGNER_HELPER_BTC_PSBT_BYTES, MAX_SIGNER_HELPER_PERSONAL_SIGN_MESSAGE_BYTES,
    MAX_SIGNER_HELPER_SAVE_IMAGE_BYTES, MAX_SIGNER_HELPER_TRANSACTION_DATA_BYTES,
    MAX_SIGNER_HELPER_TYPED_DATA_BYTES, MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES, SignerBtcPsbt,
};
use std::str::FromStr;

pub(crate) fn validate_save_image_size(size: usize) -> Result<()> {
    if !(MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES..=MAX_SIGNER_HELPER_SAVE_IMAGE_BYTES).contains(&size) {
        return Err(FramkeyError::invalid_data(format!(
            "signer helper save image size must be between {} and {} bytes, got {}",
            MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES, MAX_SIGNER_HELPER_SAVE_IMAGE_BYTES, size
        ))
        .into());
    }
    Ok(())
}

pub(crate) fn validate_personal_sign_message(message: &[u8]) -> Result<()> {
    if message.len() > MAX_SIGNER_HELPER_PERSONAL_SIGN_MESSAGE_BYTES {
        return Err(FramkeyError::invalid_data(format!(
            "personal_sign message exceeds {} bytes",
            MAX_SIGNER_HELPER_PERSONAL_SIGN_MESSAGE_BYTES
        ))
        .into());
    }
    Ok(())
}

pub(crate) fn validate_typed_data_request(typed_data: &serde_json::Value) -> Result<()> {
    let bytes = serde_json::to_vec(typed_data)?;
    if bytes.len() > MAX_SIGNER_HELPER_TYPED_DATA_BYTES {
        return Err(FramkeyError::invalid_data(format!(
            "typed-data payload exceeds {} bytes",
            MAX_SIGNER_HELPER_TYPED_DATA_BYTES
        ))
        .into());
    }
    typed_data_v4_hash(typed_data)?;
    Ok(())
}

pub(crate) fn validate_recovery_files(count: usize) -> Result<()> {
    if count == 0 {
        return Err(
            FramkeyError::invalid_data("at least one recovery share file is required").into(),
        );
    }
    if count > 4 {
        return Err(FramkeyError::invalid_data(
            "standard recovery rewrap accepts at most four backup files",
        )
        .into());
    }
    Ok(())
}

pub(crate) fn validate_sign_transaction_request(
    transaction: &framkey_ipc::SignerEvmTransaction,
) -> Result<()> {
    if transaction.data.len() > 2 + (MAX_SIGNER_HELPER_TRANSACTION_DATA_BYTES * 2) {
        return Err(FramkeyError::invalid_data(format!(
            "transaction data exceeds {} bytes",
            MAX_SIGNER_HELPER_TRANSACTION_DATA_BYTES
        ))
        .into());
    }
    if transaction.chain_id == 0 {
        return Err(FramkeyError::invalid_data("transaction chain id must be nonzero").into());
    }
    validate_transaction(&EvmTransaction {
        chain_id: transaction.chain_id,
        nonce: transaction.nonce.clone(),
        gas_limit: transaction.gas_limit.clone(),
        to: transaction.to.clone(),
        value: transaction.value.clone(),
        data: transaction.data.clone(),
        gas_price: transaction.gas_price.clone(),
        max_fee_per_gas: transaction.max_fee_per_gas.clone(),
        max_priority_fee_per_gas: transaction.max_priority_fee_per_gas.clone(),
    })?;
    Ok(())
}

pub(crate) fn validate_sign_btc_psbt_request(
    psbt: &SignerBtcPsbt,
    expected_address: &str,
) -> Result<BtcNetwork> {
    if psbt.bytes.is_empty() {
        return Err(FramkeyError::invalid_data("BTC PSBT cannot be empty").into());
    }
    if psbt.bytes.len() > MAX_SIGNER_HELPER_BTC_PSBT_BYTES {
        return Err(FramkeyError::invalid_data(format!(
            "BTC PSBT exceeds {} bytes",
            MAX_SIGNER_HELPER_BTC_PSBT_BYTES
        ))
        .into());
    }
    let network = BtcNetwork::from_str(&psbt.network)?;
    validate_p2wpkh_address(expected_address, network)?;
    Ok(network)
}

pub(crate) fn parse_expected_address(
    expected: Option<&str>,
) -> framkey_core::Result<Option<EvmAddress>> {
    expected.map(str::parse).transpose()
}

pub(crate) fn validate_expected_address(
    actual: EvmAddress,
    expected: Option<EvmAddress>,
) -> framkey_core::Result<()> {
    let Some(expected) = expected else {
        return Ok(());
    };
    if actual != expected {
        return Err(FramkeyError::invalid_data(format!(
            "signing account mismatch: requested {expected}, vault {actual}"
        )));
    }
    Ok(())
}

pub(crate) fn transaction_kind_name(kind: EvmTransactionKind) -> &'static str {
    match kind {
        EvmTransactionKind::Legacy => "legacy",
        EvmTransactionKind::Eip1559 => "eip1559",
    }
}
