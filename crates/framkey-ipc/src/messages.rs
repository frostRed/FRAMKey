use std::fmt;

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
    LocalAuthenticationFailed,
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
    KeychainAccessProbe(SignerKeychainAccessProbeRequest),
    BuildKeychainVault(SignerBuildKeychainVaultRequest),
    RecoverKeychainVault(SignerRecoverKeychainVaultRequest),
    ValidateRecoveryFiles(SignerValidateRecoveryFilesRequest),
    OpenKeychainVault(SignerOpenKeychainVaultRequest),
    PersonalSign(SignerPersonalSignRequest),
    SignTypedData(SignerSignTypedDataRequest),
    SignTransaction(SignerSignTransactionRequest),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerKeychainAccessProbeRequest {
    pub keychain_service: String,
    pub keychain_account: String,
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerRecoverKeychainVaultRequest {
    pub save_image: Vec<u8>,
    pub keychain_service: String,
    pub keychain_account: String,
    pub recovery_files: Vec<RecoveryBackupFile>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerValidateRecoveryFilesRequest {
    pub recovery_files: Vec<RecoveryBackupFile>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerOpenKeychainVaultRequest {
    pub save_image: Vec<u8>,
    pub keychain_service: String,
    pub keychain_account: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerPersonalSignRequest {
    pub save_image: Vec<u8>,
    pub keychain_service: String,
    pub keychain_account: String,
    pub message: Vec<u8>,
    #[serde(default)]
    pub expected_address: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerSignTypedDataRequest {
    pub save_image: Vec<u8>,
    pub keychain_service: String,
    pub keychain_account: String,
    pub typed_data: Value,
    #[serde(default)]
    pub expected_address: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerSignTransactionRequest {
    pub save_image: Vec<u8>,
    pub keychain_service: String,
    pub keychain_account: String,
    pub transaction: SignerEvmTransaction,
    #[serde(default)]
    pub expected_address: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    KeychainAccessProbe(SignerKeychainAccessProbeResponse),
    BuildKeychainVault(SignerBuildKeychainVaultResponse),
    RecoverKeychainVault(SignerRecoverKeychainVaultResponse),
    ValidateRecoveryFiles(SignerValidateRecoveryFilesResponse),
    OpenKeychainVault(SignerOpenKeychainVaultResponse),
    PersonalSign(SignerPersonalSignResponse),
    SignTypedData(SignerSignTypedDataResponse),
    SignTransaction(SignerSignTransactionResponse),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerKeychainAccessProbeResponse {
    pub keychain_service: String,
    pub keychain_account: String,
    pub keychain_item_id: String,
    pub keychain_access_policy: String,
    pub device_id: String,
    pub kek_id: String,
    pub card_touched: bool,
    pub vault_image_touched: bool,
    pub wallet_secret_touched: bool,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl fmt::Debug for SignerRecoverKeychainVaultRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerRecoverKeychainVaultRequest")
            .field("save_image_len", &self.save_image.len())
            .field("keychain_service", &self.keychain_service)
            .field("keychain_account", &self.keychain_account)
            .field("recovery_file_count", &self.recovery_files.len())
            .finish()
    }
}

impl fmt::Debug for SignerValidateRecoveryFilesRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerValidateRecoveryFilesRequest")
            .field("recovery_file_count", &self.recovery_files.len())
            .finish()
    }
}

impl fmt::Debug for SignerKeychainAccessProbeRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerKeychainAccessProbeRequest")
            .field("keychain_service", &self.keychain_service)
            .field("keychain_account", &self.keychain_account)
            .finish()
    }
}

impl fmt::Debug for SignerOpenKeychainVaultRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerOpenKeychainVaultRequest")
            .field("save_image_len", &self.save_image.len())
            .field("keychain_service", &self.keychain_service)
            .field("keychain_account", &self.keychain_account)
            .finish()
    }
}

impl fmt::Debug for SignerPersonalSignRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerPersonalSignRequest")
            .field("save_image_len", &self.save_image.len())
            .field("keychain_service", &self.keychain_service)
            .field("keychain_account", &self.keychain_account)
            .field("message_len", &self.message.len())
            .field("expected_address", &self.expected_address)
            .finish()
    }
}

impl fmt::Debug for SignerSignTypedDataRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerSignTypedDataRequest")
            .field("save_image_len", &self.save_image.len())
            .field("keychain_service", &self.keychain_service)
            .field("keychain_account", &self.keychain_account)
            .field("typed_data_json_len", &json_len(&self.typed_data))
            .field("expected_address", &self.expected_address)
            .finish()
    }
}

impl fmt::Debug for SignerSignTransactionRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerSignTransactionRequest")
            .field("save_image_len", &self.save_image.len())
            .field("keychain_service", &self.keychain_service)
            .field("keychain_account", &self.keychain_account)
            .field("transaction", &self.transaction)
            .field("expected_address", &self.expected_address)
            .finish()
    }
}

impl fmt::Debug for SignerEvmTransaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerEvmTransaction")
            .field("chain_id", &self.chain_id)
            .field("nonce", &self.nonce)
            .field("gas_limit", &self.gas_limit)
            .field("to", &self.to)
            .field("value", &self.value)
            .field("data_len", &self.data.len())
            .field("gas_price", &self.gas_price)
            .field("max_fee_per_gas", &self.max_fee_per_gas)
            .field("max_priority_fee_per_gas", &self.max_priority_fee_per_gas)
            .finish()
    }
}

impl fmt::Debug for SignerKeychainAccessProbeResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerKeychainAccessProbeResponse")
            .field("keychain_service", &self.keychain_service)
            .field("keychain_account", &self.keychain_account)
            .field("keychain_item_id", &self.keychain_item_id)
            .field("keychain_access_policy", &self.keychain_access_policy)
            .field("device_id", &self.device_id)
            .field("kek_id", &self.kek_id)
            .field("card_touched", &self.card_touched)
            .field("vault_image_touched", &self.vault_image_touched)
            .field("wallet_secret_touched", &self.wallet_secret_touched)
            .finish()
    }
}

impl fmt::Debug for SignerBuildKeychainVaultResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerBuildKeychainVaultResponse")
            .field("save_image_len", &self.save_image.len())
            .field("keychain_service", &self.keychain_service)
            .field("keychain_account", &self.keychain_account)
            .field("keychain_item_id", &self.keychain_item_id)
            .field("keychain_access_policy", &self.keychain_access_policy)
            .field("device_id", &self.device_id)
            .field("kek_id", &self.kek_id)
            .field("created_keychain_kek", &self.created_keychain_kek)
            .field("metadata", &self.metadata)
            .field(
                "recovery_backup_file_count",
                &self
                    .recovery_backup_pack
                    .as_ref()
                    .map(|pack| pack.files.len())
                    .unwrap_or(0),
            )
            .finish()
    }
}

impl fmt::Debug for SignerRecoverKeychainVaultResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerRecoverKeychainVaultResponse")
            .field("save_image_len", &self.save_image.len())
            .field("keychain_service", &self.keychain_service)
            .field("keychain_account", &self.keychain_account)
            .field("keychain_item_id", &self.keychain_item_id)
            .field("keychain_access_policy", &self.keychain_access_policy)
            .field("device_id", &self.device_id)
            .field("kek_id", &self.kek_id)
            .field("created_keychain_kek", &self.created_keychain_kek)
            .field("metadata", &self.metadata)
            .field("recovery_share_file_count", &self.recovery_share_file_count)
            .finish()
    }
}

impl fmt::Debug for SignerPersonalSignResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerPersonalSignResponse")
            .field("keychain_service", &self.keychain_service)
            .field("keychain_account", &self.keychain_account)
            .field("keychain_item_id", &self.keychain_item_id)
            .field("keychain_access_policy", &self.keychain_access_policy)
            .field("device_id", &self.device_id)
            .field("kek_id", &self.kek_id)
            .field("metadata", &self.metadata)
            .field("address", &self.address)
            .field("message_hash", &self.message_hash)
            .field("signature_len", &self.signature.len())
            .finish()
    }
}

impl fmt::Debug for SignerSignTypedDataResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerSignTypedDataResponse")
            .field("keychain_service", &self.keychain_service)
            .field("keychain_account", &self.keychain_account)
            .field("keychain_item_id", &self.keychain_item_id)
            .field("keychain_access_policy", &self.keychain_access_policy)
            .field("device_id", &self.device_id)
            .field("kek_id", &self.kek_id)
            .field("metadata", &self.metadata)
            .field("address", &self.address)
            .field("typed_data_hash", &self.typed_data_hash)
            .field("signature_len", &self.signature.len())
            .finish()
    }
}

impl fmt::Debug for SignerSignTransactionResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerSignTransactionResponse")
            .field("keychain_service", &self.keychain_service)
            .field("keychain_account", &self.keychain_account)
            .field("keychain_item_id", &self.keychain_item_id)
            .field("keychain_access_policy", &self.keychain_access_policy)
            .field("device_id", &self.device_id)
            .field("kek_id", &self.kek_id)
            .field("metadata", &self.metadata)
            .field("address", &self.address)
            .field("transaction_kind", &self.transaction_kind)
            .field("transaction_hash", &self.transaction_hash)
            .field("raw_transaction_len", &self.raw_transaction.len())
            .finish()
    }
}

fn json_len(value: &Value) -> usize {
    serde_json::to_vec(value).map_or(0, |bytes| bytes.len())
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
