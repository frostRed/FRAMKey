use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy)]
pub struct TransactionSimulationRequest<'a> {
    pub method: &'a str,
    pub params: &'a Value,
    pub default_chain_id: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReviewReport {
    pub simulation: TransactionSimulationReport,
    pub policy: TransactionPolicyEvaluation,
    pub risk: TransactionRiskSummary,
    pub impact: TransactionImpactSummary,
    pub trust: TransactionTrustSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSimulationReport {
    pub mode: SimulationMode,
    pub status: SimulationStatus,
    pub chain_id: String,
    pub transaction: NormalizedTransaction,
    pub native_value: Option<TokenAmount>,
    pub decoded_call: Option<DecodedCall>,
    pub asset_transfers: Vec<AssetTransfer>,
    pub approvals: Vec<ApprovalChange>,
    pub warnings: Vec<SimulationWarning>,
    // Sanitized provider metadata only; never store the full RPC response body here.
    #[serde(default, alias = "rawProviderResponse")]
    pub provider_evidence: Option<Value>,
    // Sanitized protocol-specific read-only evidence, separate from live simulation status.
    #[serde(default)]
    pub protocol_evidence: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulationMode {
    LocalDecoderOnly,
    AlchemyRpc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulationStatus {
    LocalDecoded,
    LocalWarnings,
    InvalidRequest,
    ProviderSimulated,
    ProviderFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedTransaction {
    pub method: String,
    pub chain_id: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: Option<TokenAmount>,
    pub data_bytes: Option<usize>,
    pub selector: Option<String>,
    pub data_preview: Option<String>,
    pub gas: Option<String>,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub nonce: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub hex: String,
    pub decimal: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedCall {
    pub standard: String,
    pub function: String,
    pub selector: Option<String>,
    pub contract: Option<String>,
    pub arguments: Vec<DecodedArgument>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedArgument {
    pub name: String,
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetTransfer {
    pub asset_kind: String,
    pub contract: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub amount: Option<TokenAmount>,
    pub token_id: Option<TokenAmount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalChange {
    pub asset_kind: String,
    pub contract: Option<String>,
    pub owner: Option<String>,
    pub spender: Option<String>,
    pub operator: Option<String>,
    pub amount: Option<TokenAmount>,
    pub approved: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationWarning {
    pub severity: WarningSeverity,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionPolicyEvaluation {
    pub decision: TransactionPolicyDecision,
    pub can_sign: bool,
    pub can_broadcast: bool,
    pub override_allowed: bool,
    pub blockers: Vec<PolicyBlocker>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionPolicyDecision {
    Allowed,
    RequiresUserOverride,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyBlocker {
    pub code: String,
    pub message: String,
    pub overrideable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRiskSummary {
    pub level: TransactionRiskLevel,
    pub action: TransactionRiskAction,
    pub title: String,
    pub message: String,
    pub reasons: Vec<TransactionRiskReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionRiskLevel {
    Low,
    Caution,
    High,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionRiskAction {
    OrdinaryApproval,
    HighRiskApproval,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRiskReason {
    pub source: String,
    pub code: String,
    pub title: String,
    pub message: String,
    pub severity: WarningSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionImpactSummary {
    pub title: String,
    pub native_value: bool,
    pub transfer_count: usize,
    pub approval_count: usize,
    pub live_simulated: bool,
    pub provider_asset_changes: bool,
    pub items: Vec<TransactionImpactItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionImpactItem {
    pub kind: TransactionImpactKind,
    pub title: String,
    pub message: String,
    pub severity: WarningSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionImpactKind {
    NativeValue,
    AssetTransfer,
    Approval,
    LiveSimulation,
    NoAssetMovement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTrustSummary {
    pub title: String,
    pub level: TransactionTrustLevel,
    pub known_count: usize,
    pub unknown_count: usize,
    pub items: Vec<TransactionTrustItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionTrustLevel {
    NoCounterparty,
    Recognized,
    Mixed,
    Unrecognized,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTrustItem {
    pub role: TransactionTrustRole,
    pub address: Option<String>,
    pub label: Option<String>,
    pub protocol: Option<String>,
    pub status: TransactionTrustStatus,
    pub message: String,
    pub severity: WarningSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownProtocolCounterparty {
    pub chain_id: &'static str,
    pub address: &'static str,
    pub label: &'static str,
    pub protocol: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionTrustRole {
    TransactionTo,
    ApprovalSpender,
    ApprovalOperator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionTrustStatus {
    Known,
    Unknown,
    Missing,
}
