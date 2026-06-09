mod constants;
mod framing;
mod messages;

pub use constants::{
    MAX_NATIVE_MESSAGE_BYTES, MAX_SIGNER_HELPER_BTC_PSBT_BYTES, MAX_SIGNER_HELPER_JSON_BYTES,
    MAX_SIGNER_HELPER_PERSONAL_SIGN_MESSAGE_BYTES, MAX_SIGNER_HELPER_SAVE_IMAGE_BYTES,
    MAX_SIGNER_HELPER_TRANSACTION_DATA_BYTES, MAX_SIGNER_HELPER_TYPED_DATA_BYTES,
    MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES,
};
pub use framing::{read_native_message, write_native_message};
pub use messages::{
    IpcError, IpcErrorCode, IpcRequest, IpcResponse, SignerBtcPsbt,
    SignerBuildKeychainVaultRequest, SignerBuildKeychainVaultResponse, SignerChainAccount,
    SignerEvmTransaction, SignerHelperRequest, SignerHelperResponse, SignerHelperResult,
    SignerKeychainAccessProbeRequest, SignerKeychainAccessProbeResponse,
    SignerOpenKeychainVaultRequest, SignerOpenKeychainVaultResponse, SignerPersonalSignRequest,
    SignerPersonalSignResponse, SignerRecoverKeychainVaultRequest,
    SignerRecoverKeychainVaultResponse, SignerSignBtcPsbtRequest, SignerSignBtcPsbtResponse,
    SignerSignTransactionRequest, SignerSignTransactionResponse, SignerSignTypedDataRequest,
    SignerSignTypedDataResponse, SignerValidateRecoveryFilesRequest,
    SignerValidateRecoveryFilesResponse, SignerVaultMetadata,
};

#[cfg(test)]
mod tests;
