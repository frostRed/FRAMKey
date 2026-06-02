use anyhow::Result;
use framkey_core::FramkeyError;
use framkey_evm::{EvmAddress, EvmTransactionKind};
use framkey_ipc::{
    MAX_SIGNER_HELPER_PERSONAL_SIGN_MESSAGE_BYTES, MAX_SIGNER_HELPER_SAVE_IMAGE_BYTES,
    MAX_SIGNER_HELPER_TRANSACTION_DATA_BYTES, MAX_SIGNER_HELPER_TYPED_DATA_BYTES,
    MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES,
};

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
    Ok(())
}

pub(crate) fn validate_expected_address(
    actual: EvmAddress,
    expected: Option<&str>,
) -> framkey_core::Result<()> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let expected: EvmAddress = expected.parse()?;
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
