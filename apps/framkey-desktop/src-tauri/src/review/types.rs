use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_REVIEW_QUEUE_ITEMS: usize = 32;
pub const REVIEW_REQUEST_TTL_MS: u64 = 120_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewMethodKind {
    AccountConnection,
    NetworkSwitch,
    WatchAsset,
    PersonalSign,
    EthSign,
    TypedData,
    Transaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    Completed,
    Signed,
    SignFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    Approve,
    ApproveWithRisk,
    Reject,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewDecisionRecord {
    pub decision: ReviewDecision,
    pub decided_at_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewDecisionOutcome {
    pub review_request: ReviewRequest,
    pub signing_enabled: bool,
    pub broker_mode: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewExecutionRecord {
    pub completed_at_unix_ms: u64,
    pub address: Option<String>,
    pub message_hash: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewRequest {
    pub id: String,
    pub broker_session_id: String,
    pub provider_request_id: String,
    pub method: String,
    pub kind: ReviewMethodKind,
    pub origin: Option<String>,
    pub received_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub status: ReviewStatus,
    pub decision: Option<ReviewDecisionRecord>,
    pub decision_token: String,
    pub decision_token_consumed: bool,
    pub execution: Option<ReviewExecutionRecord>,
    pub blocked_reason: String,
    pub summary: Value,
    pub params_preview: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalSignPayload {
    pub message: Vec<u8>,
    pub expected_address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedDataPayload {
    pub typed_data: Value,
    pub expected_address: Option<String>,
}
