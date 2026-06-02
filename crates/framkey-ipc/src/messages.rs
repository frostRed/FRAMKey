use framkey_recovery::{RecoveryBackupFile, RecoveryBackupPack};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IpcRequest {
    pub id: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    #[serde(default)]
    pub origin: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IpcResponse {
    Result { id: String, result: Value },
    Error { id: String, error: IpcError },
}

impl IpcResponse {
    pub fn error(id: impl Into<String>, code: IpcErrorCode, message: impl Into<String>) -> Self {
        Self::Error {
            id: id.into(),
            error: IpcError {
                code,
                message: message.into(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcError {
    pub code: IpcErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum IpcErrorCode {
    UserRejected,
    CardNotFound,
    CardReadFailed,
    VaultCorrupted,
    TouchIdFailed,
    KeychainItemNotFound,
    RecoveryRequired,
    UnsupportedChain,
    UnsupportedMethod,
    DangerousSignatureBlocked,
    UnknownContractCall,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "method")]
pub enum SignerHelperRequest {
    BuildKeychainVault(SignerBuildKeychainVaultRequest),
    RecoverKeychainVault(SignerRecoverKeychainVaultRequest),
    ValidateRecoveryFiles(SignerValidateRecoveryFilesRequest),
    OpenKeychainVault(SignerOpenKeychainVaultRequest),
    PersonalSign(SignerPersonalSignRequest),
    SignTypedData(SignerSignTypedDataRequest),
    SignTransaction(SignerSignTransactionRequest),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerBuildKeychainVaultRequest {
    pub image_size: usize,
    pub generation: u64,
    pub keychain_service: String,
    pub keychain_account: String,
    #[serde(default)]
    pub recovery_backups: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerRecoverKeychainVaultRequest {
    pub save_image: Vec<u8>,
    pub keychain_service: String,
    pub keychain_account: String,
    pub recovery_files: Vec<RecoveryBackupFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerValidateRecoveryFilesRequest {
    pub recovery_files: Vec<RecoveryBackupFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerOpenKeychainVaultRequest {
    pub save_image: Vec<u8>,
    pub keychain_service: String,
    pub keychain_account: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerPersonalSignRequest {
    pub save_image: Vec<u8>,
    pub keychain_service: String,
    pub keychain_account: String,
    pub message: Vec<u8>,
    #[serde(default)]
    pub expected_address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerSignTypedDataRequest {
    pub save_image: Vec<u8>,
    pub keychain_service: String,
    pub keychain_account: String,
    pub typed_data: Value,
    #[serde(default)]
    pub expected_address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerSignTransactionRequest {
    pub save_image: Vec<u8>,
    pub keychain_service: String,
    pub keychain_account: String,
    pub transaction: SignerEvmTransaction,
    #[serde(default)]
    pub expected_address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerEvmTransaction {
    pub chain_id: u64,
    pub nonce: String,
    pub gas_limit: String,
    pub to: Option<String>,
    pub value: String,
    pub data: String,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum SignerHelperResponse {
    Ok { result: Box<SignerHelperResult> },
    Error { error: IpcError },
}

impl SignerHelperResponse {
    pub fn ok(result: SignerHelperResult) -> Self {
        Self::Ok {
            result: Box::new(result),
        }
    }

    pub fn error(error: IpcError) -> Self {
        Self::Error { error }
    }

    pub fn into_result(self) -> std::result::Result<SignerHelperResult, IpcError> {
        match self {
            Self::Ok { result } => Ok(*result),
            Self::Error { error } => Err(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum SignerHelperResult {
    BuildKeychainVault(SignerBuildKeychainVaultResponse),
    RecoverKeychainVault(SignerRecoverKeychainVaultResponse),
    ValidateRecoveryFiles(SignerValidateRecoveryFilesResponse),
    OpenKeychainVault(SignerOpenKeychainVaultResponse),
    PersonalSign(SignerPersonalSignResponse),
    SignTypedData(SignerSignTypedDataResponse),
    SignTransaction(SignerSignTransactionResponse),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerBuildKeychainVaultResponse {
    pub save_image: Vec<u8>,
    pub keychain_service: String,
    pub keychain_account: String,
    pub keychain_item_id: String,
    pub keychain_access_policy: String,
    pub device_id: String,
    pub kek_id: String,
    pub created_keychain_kek: bool,
    pub metadata: SignerVaultMetadata,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_backup_pack: Option<RecoveryBackupPack>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerRecoverKeychainVaultResponse {
    pub save_image: Vec<u8>,
    pub keychain_service: String,
    pub keychain_account: String,
    pub keychain_item_id: String,
    pub keychain_access_policy: String,
    pub device_id: String,
    pub kek_id: String,
    pub created_keychain_kek: bool,
    pub metadata: SignerVaultMetadata,
    pub recovery_share_file_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerValidateRecoveryFilesResponse {
    pub backup_set_id: String,
    pub wallet_id: String,
    pub generation: u64,
    pub policy_id: String,
    pub recovery_share_file_count: usize,
    pub satisfied_groups: Vec<String>,
    pub can_recover: bool,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerOpenKeychainVaultResponse {
    pub keychain_service: String,
    pub keychain_account: String,
    pub keychain_item_id: String,
    pub keychain_access_policy: String,
    pub device_id: String,
    pub kek_id: String,
    pub metadata: SignerVaultMetadata,
    pub address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerPersonalSignResponse {
    pub keychain_service: String,
    pub keychain_account: String,
    pub keychain_item_id: String,
    pub keychain_access_policy: String,
    pub device_id: String,
    pub kek_id: String,
    pub metadata: SignerVaultMetadata,
    pub address: String,
    pub message_hash: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerSignTypedDataResponse {
    pub keychain_service: String,
    pub keychain_account: String,
    pub keychain_item_id: String,
    pub keychain_access_policy: String,
    pub device_id: String,
    pub kek_id: String,
    pub metadata: SignerVaultMetadata,
    pub address: String,
    pub typed_data_hash: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerSignTransactionResponse {
    pub keychain_service: String,
    pub keychain_account: String,
    pub keychain_item_id: String,
    pub keychain_access_policy: String,
    pub device_id: String,
    pub kek_id: String,
    pub metadata: SignerVaultMetadata,
    pub address: String,
    pub transaction_kind: String,
    pub transaction_hash: String,
    pub raw_transaction: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerVaultMetadata {
    pub image_size: usize,
    pub slot_size: usize,
    pub wallet_id: String,
    pub generation: u64,
    pub wallet_type: String,
    pub active_slot_hash_valid: bool,
    pub active_slot_payload_hash_valid: bool,
    pub wallet_secret_hash: Option<String>,
}
