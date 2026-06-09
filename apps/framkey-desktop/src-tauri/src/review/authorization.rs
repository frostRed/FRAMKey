use anyhow::{Result, bail};
use serde_json::{Map, Value};

use super::*;

pub fn dangerous_method_kind(method: &str) -> Option<ReviewMethodKind> {
    match method {
        "eth_requestAccounts" | "wallet_requestPermissions" => {
            Some(ReviewMethodKind::AccountConnection)
        }
        "wallet_addEthereumChain" | "wallet_switchEthereumChain" => {
            Some(ReviewMethodKind::NetworkSwitch)
        }
        "wallet_watchAsset" => Some(ReviewMethodKind::WatchAsset),
        "personal_sign" => Some(ReviewMethodKind::PersonalSign),
        "eth_sign" => Some(ReviewMethodKind::EthSign),
        "eth_signTypedData"
        | "eth_signTypedData_v1"
        | "eth_signTypedData_v3"
        | "eth_signTypedData_v4" => Some(ReviewMethodKind::TypedData),
        "eth_sendTransaction" | "eth_signTransaction" => Some(ReviewMethodKind::Transaction),
        "framkey_btcSendTransaction" => Some(ReviewMethodKind::BtcTransaction),
        _ => None,
    }
}

pub(crate) fn blocked_reason(kind: ReviewMethodKind) -> &'static str {
    match kind {
        ReviewMethodKind::AccountConnection => "account access requires trusted approval",
        ReviewMethodKind::NetworkSwitch => "network switching requires trusted approval",
        ReviewMethodKind::WatchAsset => "adding watched assets requires trusted approval",
        ReviewMethodKind::Transaction => "transaction signing requires trusted policy approval",
        ReviewMethodKind::BtcTransaction => {
            "BTC transaction signing requires trusted PSBT/UTXO policy approval"
        }
        ReviewMethodKind::PersonalSign
        | ReviewMethodKind::EthSign
        | ReviewMethodKind::TypedData => "message signing requires trusted approval",
    }
}

pub(crate) fn decision_broker_mode(
    request: &ReviewRequest,
    decision: ReviewDecision,
) -> Result<&'static str> {
    match (request.kind, decision) {
        (_, ReviewDecision::Reject) => Ok("dry_run"),
        (ReviewMethodKind::AccountConnection, ReviewDecision::Approve) => Ok("account_connection"),
        (ReviewMethodKind::AccountConnection, ReviewDecision::ApproveWithRisk) => {
            bail!("account connection does not support high-risk approval")
        }
        (ReviewMethodKind::NetworkSwitch, ReviewDecision::Approve) => Ok("network_switch"),
        (ReviewMethodKind::NetworkSwitch, ReviewDecision::ApproveWithRisk) => {
            bail!("network switching does not support high-risk approval")
        }
        (ReviewMethodKind::WatchAsset, ReviewDecision::Approve) => Ok("watch_asset"),
        (ReviewMethodKind::WatchAsset, ReviewDecision::ApproveWithRisk) => {
            bail!("watch asset does not support high-risk approval")
        }
        (ReviewMethodKind::PersonalSign, ReviewDecision::Approve) => {
            personal_sign_broker_mode_for_decision(request, decision)
        }
        (ReviewMethodKind::PersonalSign, ReviewDecision::ApproveWithRisk) => {
            bail!("personal_sign does not support high-risk approval")
        }
        (ReviewMethodKind::TypedData, ReviewDecision::Approve) => {
            typed_data_broker_mode_for_decision(request, decision)
        }
        (ReviewMethodKind::TypedData, ReviewDecision::ApproveWithRisk) => {
            bail!("typed-data signing does not support high-risk approval")
        }
        (
            ReviewMethodKind::Transaction,
            ReviewDecision::Approve | ReviewDecision::ApproveWithRisk,
        ) => transaction_broker_mode_for_decision(request, decision),
        (ReviewMethodKind::BtcTransaction, ReviewDecision::Approve) => {
            btc_transaction_broker_mode_for_decision(request, decision)
        }
        (ReviewMethodKind::BtcTransaction, ReviewDecision::ApproveWithRisk) => {
            bail!("BTC transaction signing does not support high-risk approval")
        }
        (_, ReviewDecision::Approve) => Ok("dry_run"),
        (_, ReviewDecision::ApproveWithRisk) => {
            bail!("high-risk approval is only valid for transactions")
        }
    }
}

pub fn transaction_signing_authorization(request: &ReviewRequest) -> Result<&'static str> {
    if request.status != ReviewStatus::Approved {
        bail!("transaction review request {} is not approved", request.id);
    }
    let decision = request
        .decision
        .as_ref()
        .ok_or_else(|| {
            anyhow::anyhow!("transaction review request {} has no decision", request.id)
        })?
        .decision;
    transaction_broker_mode_for_decision(request, decision)
}

pub fn btc_transaction_signing_authorization(request: &ReviewRequest) -> Result<&'static str> {
    if request.status != ReviewStatus::Approved {
        bail!(
            "BTC transaction review request {} is not approved",
            request.id
        );
    }
    let decision = request
        .decision
        .as_ref()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "BTC transaction review request {} has no decision",
                request.id
            )
        })?
        .decision;
    btc_transaction_broker_mode_for_decision(request, decision)
}

pub fn typed_data_signing_authorization(request: &ReviewRequest) -> Result<&'static str> {
    if request.status != ReviewStatus::Approved {
        bail!("typed-data review request {} is not approved", request.id);
    }
    let decision = request
        .decision
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("typed-data review request {} has no decision", request.id))?
        .decision;
    typed_data_broker_mode_for_decision(request, decision)
}

pub fn personal_sign_signing_authorization(request: &ReviewRequest) -> Result<&'static str> {
    if request.status != ReviewStatus::Approved {
        bail!(
            "personal_sign review request {} is not approved",
            request.id
        );
    }
    let decision = request
        .decision
        .as_ref()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "personal_sign review request {} has no decision",
                request.id
            )
        })?
        .decision;
    personal_sign_broker_mode_for_decision(request, decision)
}

pub fn network_switch_authorization(request: &ReviewRequest) -> Result<()> {
    if request.kind != ReviewMethodKind::NetworkSwitch {
        bail!("review request {} is not a network switch", request.id);
    }
    if request.status != ReviewStatus::Approved {
        bail!(
            "network switch review request {} is not approved",
            request.id
        );
    }
    let decision = request
        .decision
        .as_ref()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "network switch review request {} has no decision",
                request.id
            )
        })?
        .decision;
    match decision {
        ReviewDecision::Approve => Ok(()),
        ReviewDecision::ApproveWithRisk => {
            bail!("network switching does not support high-risk approval")
        }
        ReviewDecision::Reject => {
            bail!("network switch review request {} was rejected", request.id)
        }
    }
}

pub(crate) fn personal_sign_broker_mode_for_decision(
    request: &ReviewRequest,
    decision: ReviewDecision,
) -> Result<&'static str> {
    if request.kind != ReviewMethodKind::PersonalSign {
        bail!("review request {} is not personal_sign", request.id);
    }
    match decision {
        ReviewDecision::Approve => {
            ensure_signable_personal_sign(request)?;
            Ok("controlled_personal_sign")
        }
        ReviewDecision::ApproveWithRisk => {
            bail!("personal_sign does not support high-risk approval")
        }
        ReviewDecision::Reject => bail!("personal_sign review request {} was rejected", request.id),
    }
}

pub fn signable_personal_sign_intent(request: &ReviewRequest) -> Option<&'static str> {
    if personal_sign_policy_can_sign(request) != Some(true) {
        return None;
    }
    let siwe_status = request
        .summary
        .get("siwe")
        .and_then(Value::as_object)
        .and_then(|siwe| siwe.get("status"))
        .and_then(Value::as_str)?;
    match siwe_status {
        "ok" => Some("siwe"),
        _ => None,
    }
}

pub(crate) fn ensure_signable_personal_sign(request: &ReviewRequest) -> Result<&'static str> {
    let Some(intent) = signable_personal_sign_intent(request) else {
        let blocker = first_personal_sign_policy_blocker(request);
        let message = blocker
            .as_ref()
            .and_then(|blocker| blocker.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("personal_sign signing is enabled only for short-lived SIWE messages");
        let code = blocker
            .as_ref()
            .and_then(|blocker| blocker.get("code"))
            .and_then(Value::as_str)
            .unwrap_or("personal_sign_policy_blocked");
        bail!("personal_sign policy blocks signing: {code}: {message}");
    };
    Ok(intent)
}

pub(crate) fn personal_sign_policy_can_sign(request: &ReviewRequest) -> Option<bool> {
    request
        .summary
        .get("policy")
        .and_then(Value::as_object)
        .and_then(|policy| policy.get("canSign"))
        .and_then(Value::as_bool)
}

pub(crate) fn first_personal_sign_policy_blocker(request: &ReviewRequest) -> Option<Value> {
    request
        .summary
        .get("policy")
        .and_then(Value::as_object)
        .and_then(|policy| policy.get("blockers"))
        .and_then(Value::as_array)
        .and_then(|blockers| blockers.first())
        .cloned()
}

pub(crate) fn typed_data_broker_mode_for_decision(
    request: &ReviewRequest,
    decision: ReviewDecision,
) -> Result<&'static str> {
    if request.kind != ReviewMethodKind::TypedData {
        bail!("review request {} is not typed data", request.id);
    }
    match decision {
        ReviewDecision::Approve if request.method == "eth_signTypedData_v4" => {
            ensure_signable_typed_data(request)?;
            Ok("controlled_typed_data_signing")
        }
        ReviewDecision::Approve => {
            bail!("typed-data signing is only enabled for eth_signTypedData_v4")
        }
        ReviewDecision::ApproveWithRisk => {
            bail!("typed-data signing does not support high-risk approval")
        }
        ReviewDecision::Reject => bail!("typed-data review request {} was rejected", request.id),
    }
}

pub fn signable_typed_data_intent(request: &ReviewRequest) -> Option<&'static str> {
    if typed_data_policy_can_sign(request) != Some(true) {
        return None;
    }
    let intent = request
        .summary
        .get("typedData")
        .and_then(Value::as_object)
        .and_then(|typed_data| typed_data.get("intent"))
        .and_then(Value::as_str)?;
    match intent {
        "erc20_permit" => Some("erc20_permit"),
        "permit2_permit_single" => Some("permit2_permit_single"),
        "permit2_permit_batch" => Some("permit2_permit_batch"),
        "permit2_transfer_from" => Some("permit2_transfer_from"),
        "permit2_batch_transfer_from" => Some("permit2_batch_transfer_from"),
        _ => None,
    }
}

pub(crate) fn ensure_signable_typed_data(request: &ReviewRequest) -> Result<&'static str> {
    let intent = request
        .summary
        .get("typedData")
        .and_then(Value::as_object)
        .and_then(|typed_data| typed_data.get("intent"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow::anyhow!("typed-data review request {} is missing intent", request.id)
        })?;
    let Some(signable_intent) = signable_typed_data_intent(request) else {
        let blocker = first_typed_data_policy_blocker(request);
        let message = blocker
            .as_ref()
            .and_then(|blocker| blocker.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("typed-data signing is only enabled for recognized Permit requests");
        let code = blocker
            .as_ref()
            .and_then(|blocker| blocker.get("code"))
            .and_then(Value::as_str)
            .unwrap_or("typed_data_policy_blocked");
        bail!("typed-data policy blocks signing: {code}: {message}");
    };
    if signable_intent != intent {
        bail!("typed-data signing is only enabled for recognized Permit requests");
    }
    Ok(signable_intent)
}

pub(crate) fn typed_data_policy_can_sign(request: &ReviewRequest) -> Option<bool> {
    request
        .summary
        .get("typedData")
        .and_then(Value::as_object)
        .and_then(|typed_data| typed_data.get("policy"))
        .and_then(Value::as_object)
        .and_then(|policy| policy.get("canSign"))
        .and_then(Value::as_bool)
}

pub(crate) fn first_typed_data_policy_blocker(request: &ReviewRequest) -> Option<Value> {
    request
        .summary
        .get("typedData")
        .and_then(Value::as_object)
        .and_then(|typed_data| typed_data.get("policy"))
        .and_then(Value::as_object)
        .and_then(|policy| policy.get("blockers"))
        .and_then(Value::as_array)
        .and_then(|blockers| blockers.first())
        .cloned()
}

pub(crate) fn transaction_broker_mode_for_decision(
    request: &ReviewRequest,
    decision: ReviewDecision,
) -> Result<&'static str> {
    if request.kind != ReviewMethodKind::Transaction {
        bail!("review request {} is not a transaction", request.id);
    }

    let (can_sign, override_allowed) = transaction_policy_flags(request)?;
    match decision {
        ReviewDecision::Approve if can_sign => Ok("controlled_transaction_signing"),
        ReviewDecision::Approve if override_allowed => {
            bail!("transaction requires explicit high-risk approval")
        }
        ReviewDecision::Approve => bail!("transaction policy blocks signing"),
        ReviewDecision::ApproveWithRisk if override_allowed => {
            Ok("controlled_transaction_high_risk_override")
        }
        ReviewDecision::ApproveWithRisk => {
            bail!("transaction policy does not allow high-risk override")
        }
        ReviewDecision::Reject => bail!("transaction review request {} was rejected", request.id),
    }
}

pub(crate) fn transaction_policy_flags(request: &ReviewRequest) -> Result<(bool, bool)> {
    let policy = request
        .summary
        .get("policy")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "transaction review request {} is missing policy evaluation",
                request.id
            )
        })?;
    let can_sign = policy_bool(policy, "canSign", request)?;
    let override_allowed = policy_bool(policy, "overrideAllowed", request)?;
    Ok((can_sign, override_allowed))
}

pub(crate) fn btc_transaction_broker_mode_for_decision(
    request: &ReviewRequest,
    decision: ReviewDecision,
) -> Result<&'static str> {
    if request.kind != ReviewMethodKind::BtcTransaction {
        bail!("review request {} is not a BTC transaction", request.id);
    }
    let can_sign = request
        .summary
        .get("policy")
        .and_then(Value::as_object)
        .and_then(|policy| policy.get("canSign"))
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "BTC transaction review request {} is missing policy.canSign",
                request.id
            )
        })?;
    match decision {
        ReviewDecision::Approve if can_sign => Ok("controlled_btc_transaction_signing"),
        ReviewDecision::Approve => bail!("BTC transaction policy blocks signing"),
        ReviewDecision::ApproveWithRisk => {
            bail!("BTC transaction signing does not support high-risk approval")
        }
        ReviewDecision::Reject => {
            bail!("BTC transaction review request {} was rejected", request.id)
        }
    }
}

pub(crate) fn policy_bool(
    policy: &Map<String, Value>,
    field: &str,
    request: &ReviewRequest,
) -> Result<bool> {
    policy.get(field).and_then(Value::as_bool).ok_or_else(|| {
        anyhow::anyhow!(
            "transaction review request {} policy field {field} is missing or not boolean",
            request.id
        )
    })
}
