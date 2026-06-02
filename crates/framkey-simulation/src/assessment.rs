use std::{
    cmp::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    decoder::{is_max_u256, looks_like_eth_address, overrideable_policy_blocker, policy_blocker},
    model::{
        ApprovalChange, AssetTransfer, PolicyBlocker, SimulationMode, SimulationStatus,
        TokenAmount, TransactionImpactItem, TransactionImpactKind, TransactionImpactSummary,
        TransactionPolicyDecision, TransactionPolicyEvaluation, TransactionRiskAction,
        TransactionRiskLevel, TransactionRiskReason, TransactionRiskSummary,
        TransactionSimulationReport, TransactionTrustItem, TransactionTrustLevel,
        TransactionTrustRole, TransactionTrustStatus, TransactionTrustSummary, WarningSeverity,
    },
    registry::known_counterparty,
};

const MAX_SWAP_DEADLINE_SECONDS_FROM_NOW: u64 = 24 * 60 * 60;
const AAVE_BLOCK_HEALTH_FACTOR_WAD: &str = "1200000000000000000";
const AAVE_SAFE_HEALTH_FACTOR_WAD: &str = "1500000000000000000";

pub fn evaluate_transaction_policy(
    report: &TransactionSimulationReport,
) -> TransactionPolicyEvaluation {
    let mut blockers = Vec::new();

    let live_simulated = report.mode == SimulationMode::AlchemyRpc
        && report.status == SimulationStatus::ProviderSimulated
        && report.provider_evidence.is_some();
    if !live_simulated {
        blockers.push(overrideable_policy_blocker(
            "live_simulation_required",
            "transaction signing has no live third-party simulation result",
        ));
    }

    if report.status == SimulationStatus::InvalidRequest {
        blockers.push(policy_blocker(
            "invalid_transaction_request",
            "transaction request or calldata is invalid",
        ));
    }
    if report.status == SimulationStatus::ProviderFailed {
        blockers.push(policy_blocker(
            "simulation_provider_failed",
            "transaction simulation provider failed or returned an error",
        ));
    }

    if report
        .warnings
        .iter()
        .any(|warning| warning.code == "unknown_function_selector")
    {
        blockers.push(overrideable_policy_blocker(
            "unknown_calldata",
            "transaction calldata is not covered by the local decoder",
        ));
    }
    if report
        .warnings
        .iter()
        .any(|warning| warning.code == "unlimited_token_approval")
    {
        blockers.push(overrideable_policy_blocker(
            "high_risk_unlimited_approval",
            "token approval grants the maximum uint256 allowance",
        ));
    }
    if report
        .warnings
        .iter()
        .any(|warning| warning.code == "operator_approval_for_all")
    {
        blockers.push(overrideable_policy_blocker(
            "high_risk_operator_approval",
            "operator approval grants transfer authority for all matching tokens",
        ));
    }
    if report
        .approvals
        .iter()
        .any(|approval| unknown_active_approval_authority(&report.chain_id, approval))
    {
        blockers.push(overrideable_policy_blocker(
            "unknown_approval_authority",
            "approval grants token authority to an unrecognized spender or operator",
        ));
    }
    add_protocol_semantic_blockers(report, &mut blockers);

    let has_blocking = blockers.iter().any(|blocker| !blocker.overrideable);
    let has_overrideable = blockers.iter().any(|blocker| blocker.overrideable);
    let decision = if has_blocking {
        TransactionPolicyDecision::Blocked
    } else if has_overrideable {
        TransactionPolicyDecision::RequiresUserOverride
    } else {
        TransactionPolicyDecision::Allowed
    };
    let can_sign = decision == TransactionPolicyDecision::Allowed;
    let override_allowed = decision == TransactionPolicyDecision::RequiresUserOverride;

    TransactionPolicyEvaluation {
        decision,
        can_sign,
        can_broadcast: can_sign,
        override_allowed,
        blockers,
    }
}

fn add_protocol_semantic_blockers(
    report: &TransactionSimulationReport,
    blockers: &mut Vec<PolicyBlocker>,
) {
    let Some(call) = &report.decoded_call else {
        return;
    };
    match call.standard.as_str() {
        "uniswap_universal_router" => {
            add_universal_router_blockers(report, blockers);
        }
        "multicall" => push_overrideable_blocker(
            blockers,
            "multicall_semantics_incomplete",
            "multicall nested calls are not fully decoded locally",
        ),
        "uniswap_v2_router" | "uniswap_v3_swap_router" => {
            add_uniswap_swap_blockers(report, blockers);
        }
        "aave_v3_pool" => {
            add_aave_blockers(report, blockers);
        }
        _ => {}
    }
}

fn add_universal_router_blockers(
    report: &TransactionSimulationReport,
    blockers: &mut Vec<PolicyBlocker>,
) {
    let Some(call) = &report.decoded_call else {
        return;
    };
    if decoded_arg(call, "unsupportedCommandCount")
        .and_then(parse_decimal_usize)
        .is_some_and(|count| count > 0)
    {
        push_overrideable_blocker(
            blockers,
            "universal_router_semantics_incomplete",
            "Universal Router contains commands the local decoder does not fully support",
        );
    }
    add_uniswap_swap_blockers(report, blockers);
}

fn add_uniswap_swap_blockers(
    report: &TransactionSimulationReport,
    blockers: &mut Vec<PolicyBlocker>,
) {
    let Some(call) = &report.decoded_call else {
        return;
    };
    if decoded_args(call, "amountOutMin")
        .chain(decoded_args(call, "amountOutMinimum"))
        .any(|amount| amount == "0")
    {
        push_overrideable_blocker(
            blockers,
            "uniswap_zero_slippage_floor",
            "Uniswap swap has a zero minimum output amount",
        );
    }

    if let Some(deadline) = decoded_arg(call, "deadline") {
        match parse_decimal_u64(deadline) {
            Some(deadline) => {
                let now = current_unix_seconds();
                if deadline <= now {
                    push_overrideable_blocker(
                        blockers,
                        "swap_deadline_expired",
                        "swap deadline is expired",
                    );
                } else if deadline.saturating_sub(now) > MAX_SWAP_DEADLINE_SECONDS_FROM_NOW {
                    push_overrideable_blocker(
                        blockers,
                        "swap_deadline_too_far",
                        "swap deadline is too far in the future for ordinary approval",
                    );
                }
            }
            None => push_overrideable_blocker(
                blockers,
                "swap_deadline_invalid",
                "swap deadline is not a valid unix timestamp",
            ),
        }
    }

    if let Some(from) = report.transaction.from.as_deref()
        && looks_like_eth_address(from)
        && decoded_args(call, "recipient")
            .chain(decoded_args(call, "to"))
            .any(|recipient| {
                looks_like_eth_address(recipient) && !from.eq_ignore_ascii_case(recipient)
            })
    {
        push_overrideable_blocker(
            blockers,
            "swap_third_party_recipient",
            "swap output recipient differs from the signing account",
        );
    }
}

fn add_aave_blockers(report: &TransactionSimulationReport, blockers: &mut Vec<PolicyBlocker>) {
    let Some(call) = &report.decoded_call else {
        return;
    };
    if let Some(on_behalf_of) = decoded_arg(call, "onBehalfOf")
        && let Some(from) = report.transaction.from.as_deref()
        && looks_like_eth_address(from)
        && looks_like_eth_address(on_behalf_of)
        && !from.eq_ignore_ascii_case(on_behalf_of)
    {
        push_overrideable_blocker(
            blockers,
            "aave_third_party_account",
            "Aave request acts on behalf of an account different from the signer",
        );
    }
    if call.function == "withdraw(address,uint256,address)"
        && let Some(to) = decoded_arg(call, "to")
        && let Some(from) = report.transaction.from.as_deref()
        && looks_like_eth_address(from)
        && looks_like_eth_address(to)
        && !from.eq_ignore_ascii_case(to)
    {
        push_overrideable_blocker(
            blockers,
            "aave_third_party_withdraw_recipient",
            "Aave withdraw sends assets to an account different from the signer",
        );
    }

    match call.function.as_str() {
        "borrow(address,uint256,uint256,uint16,address)" => add_aave_health_factor_blocker(
            report,
            blockers,
            "aave_borrow_health_factor_unknown",
            "Aave borrow requires account health-factor evidence",
        ),
        "withdraw(address,uint256,address)" => add_aave_health_factor_blocker(
            report,
            blockers,
            "aave_withdraw_health_factor_unknown",
            "Aave withdraw requires account health-factor evidence",
        ),
        "setUserUseReserveAsCollateral(address,bool)" => {
            if decoded_arg(call, "useAsCollateral") == Some("false") {
                add_aave_health_factor_blocker(
                    report,
                    blockers,
                    "aave_collateral_disable_risk",
                    "disabling Aave collateral requires account health-factor evidence",
                );
            }
        }
        _ => {}
    }
}

fn add_aave_health_factor_blocker(
    report: &TransactionSimulationReport,
    blockers: &mut Vec<PolicyBlocker>,
    unknown_code: &str,
    unknown_message: &str,
) {
    match aave_health_factor_evidence(report) {
        AaveHealthFactorEvidence::Healthy => push_overrideable_blocker(
            blockers,
            "aave_post_transaction_health_factor_unknown",
            "Aave current health-factor evidence does not prove post-transaction account safety",
        ),
        AaveHealthFactorEvidence::Caution => push_overrideable_blocker(
            blockers,
            "aave_health_factor_caution",
            "Aave account health factor is below the ordinary-approval threshold",
        ),
        AaveHealthFactorEvidence::Unsafe => {
            if blockers
                .iter()
                .any(|blocker| blocker.code == "aave_health_factor_liquidation_risk")
            {
                return;
            }
            blockers.push(policy_blocker(
                "aave_health_factor_liquidation_risk",
                "Aave account health factor is too close to liquidation for signing",
            ));
        }
        AaveHealthFactorEvidence::Missing => {
            push_overrideable_blocker(blockers, unknown_code, unknown_message);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AaveHealthFactorEvidence {
    Healthy,
    Caution,
    Unsafe,
    Missing,
}

fn aave_health_factor_evidence(report: &TransactionSimulationReport) -> AaveHealthFactorEvidence {
    let Some(health_factor) = report
        .protocol_evidence
        .as_ref()
        .and_then(|evidence| evidence.get("aave"))
        .and_then(|aave| {
            (aave.get("status").and_then(serde_json::Value::as_str) == Some("ok")).then_some(aave)
        })
        .and_then(|aave| aave.get("healthFactor").and_then(serde_json::Value::as_str))
    else {
        return AaveHealthFactorEvidence::Missing;
    };
    if compare_decimal_strings(health_factor, AAVE_BLOCK_HEALTH_FACTOR_WAD)
        .is_some_and(|ordering| ordering == Ordering::Less)
    {
        return AaveHealthFactorEvidence::Unsafe;
    }
    if compare_decimal_strings(health_factor, AAVE_SAFE_HEALTH_FACTOR_WAD)
        .is_some_and(|ordering| ordering == Ordering::Less)
    {
        return AaveHealthFactorEvidence::Caution;
    }
    AaveHealthFactorEvidence::Healthy
}

fn push_overrideable_blocker(blockers: &mut Vec<PolicyBlocker>, code: &str, message: &str) {
    if blockers.iter().any(|blocker| blocker.code == code) {
        return;
    }
    blockers.push(overrideable_policy_blocker(code, message));
}

fn decoded_arg<'a>(call: &'a crate::model::DecodedCall, name: &str) -> Option<&'a str> {
    call.arguments
        .iter()
        .find(|argument| argument.name == name)
        .map(|argument| argument.value.as_str())
}

fn decoded_args<'a>(
    call: &'a crate::model::DecodedCall,
    name: &'a str,
) -> impl Iterator<Item = &'a str> + 'a {
    call.arguments
        .iter()
        .filter(move |argument| argument.name == name)
        .map(|argument| argument.value.as_str())
}

fn parse_decimal_u64(value: &str) -> Option<u64> {
    if value.is_empty() || !value.as_bytes().iter().all(u8::is_ascii_digit) {
        return None;
    }
    value.parse().ok()
}

fn parse_decimal_usize(value: &str) -> Option<usize> {
    if value.is_empty() || !value.as_bytes().iter().all(u8::is_ascii_digit) {
        return None;
    }
    value.parse().ok()
}

fn compare_decimal_strings(left: &str, right: &str) -> Option<Ordering> {
    if left.is_empty()
        || right.is_empty()
        || !left.as_bytes().iter().all(u8::is_ascii_digit)
        || !right.as_bytes().iter().all(u8::is_ascii_digit)
    {
        return None;
    }
    let left = left.trim_start_matches('0');
    let right = right.trim_start_matches('0');
    let left = if left.is_empty() { "0" } else { left };
    let right = if right.is_empty() { "0" } else { right };
    Some(match left.len().cmp(&right.len()) {
        Ordering::Equal => left.cmp(right),
        ordering => ordering,
    })
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn evaluate_transaction_impact(
    report: &TransactionSimulationReport,
) -> TransactionImpactSummary {
    let native_value = report.native_value.is_some();
    let transfer_count = report.asset_transfers.len();
    let approval_count = report.approvals.len();
    let live_simulated = report.mode == SimulationMode::AlchemyRpc
        && report.status == SimulationStatus::ProviderSimulated
        && report.provider_evidence.is_some();
    let provider_asset_changes = live_simulated && (transfer_count > 0 || approval_count > 0);

    let mut items = Vec::new();
    if let Some(value) = &report.native_value {
        items.push(TransactionImpactItem {
            kind: TransactionImpactKind::NativeValue,
            title: "Native value transfer".to_owned(),
            message: format!("moves {} wei of native chain value", value.decimal),
            severity: WarningSeverity::Warning,
        });
    }

    for approval in report.approvals.iter().take(4) {
        items.push(approval_impact_item(approval));
    }
    if approval_count > 4 {
        items.push(TransactionImpactItem {
            kind: TransactionImpactKind::Approval,
            title: "More approval changes".to_owned(),
            message: format!(
                "{} additional approval change(s) are shown in details",
                approval_count - 4
            ),
            severity: WarningSeverity::Info,
        });
    }

    for transfer in report.asset_transfers.iter().take(4) {
        items.push(transfer_impact_item(transfer));
    }
    if transfer_count > 4 {
        items.push(TransactionImpactItem {
            kind: TransactionImpactKind::AssetTransfer,
            title: "More asset transfers".to_owned(),
            message: format!(
                "{} additional transfer(s) are shown in details",
                transfer_count - 4
            ),
            severity: WarningSeverity::Info,
        });
    }

    if live_simulated {
        items.push(TransactionImpactItem {
            kind: TransactionImpactKind::LiveSimulation,
            title: "Live simulation attached".to_owned(),
            message: if provider_asset_changes {
                format!(
                    "provider reported {} transfer(s) and {} approval change(s)",
                    transfer_count, approval_count
                )
            } else {
                "provider reported no token transfers or approval changes".to_owned()
            },
            severity: WarningSeverity::Info,
        });
    }

    if items.is_empty() {
        items.push(TransactionImpactItem {
            kind: TransactionImpactKind::NoAssetMovement,
            title: "No decoded asset movement".to_owned(),
            message:
                "local decoder did not find native value, token transfers, or approval changes"
                    .to_owned(),
            severity: WarningSeverity::Info,
        });
    }

    TransactionImpactSummary {
        title: transaction_impact_title(
            native_value,
            transfer_count,
            approval_count,
            live_simulated,
        ),
        native_value,
        transfer_count,
        approval_count,
        live_simulated,
        provider_asset_changes,
        items,
    }
}

fn approval_impact_item(approval: &ApprovalChange) -> TransactionImpactItem {
    if let Some(operator) = &approval.operator {
        let approved = approval.approved.unwrap_or(true);
        return TransactionImpactItem {
            kind: TransactionImpactKind::Approval,
            title: if approved {
                "Operator approval".to_owned()
            } else {
                "Operator approval revocation".to_owned()
            },
            message: format!(
                "{} operator {} for {}",
                if approved { "grants" } else { "revokes" },
                display_address(operator),
                approval.asset_kind
            ),
            severity: if approved {
                WarningSeverity::Warning
            } else {
                WarningSeverity::Info
            },
        };
    }

    let spender = approval
        .spender
        .as_deref()
        .map(display_address)
        .unwrap_or_else(|| "unknown spender".to_owned());
    let amount = approval
        .amount
        .as_ref()
        .map(token_amount_impact_label)
        .unwrap_or_else(|| {
            approval
                .approved
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown amount".to_owned())
        });
    let unlimited = approval
        .amount
        .as_ref()
        .is_some_and(|amount| is_max_u256(&amount.hex));
    TransactionImpactItem {
        kind: TransactionImpactKind::Approval,
        title: if unlimited {
            "Unlimited token approval".to_owned()
        } else {
            "Token approval".to_owned()
        },
        message: format!(
            "allows {spender} to spend {amount} of {}",
            approval.asset_kind
        ),
        severity: if unlimited {
            WarningSeverity::Warning
        } else {
            WarningSeverity::Info
        },
    }
}

fn transfer_impact_item(transfer: &AssetTransfer) -> TransactionImpactItem {
    let from = transfer
        .from
        .as_deref()
        .map(display_address)
        .unwrap_or_else(|| "unknown sender".to_owned());
    let to = transfer
        .to
        .as_deref()
        .map(display_address)
        .unwrap_or_else(|| "unknown recipient".to_owned());
    let amount = transfer
        .amount
        .as_ref()
        .map(token_amount_impact_label)
        .or_else(|| {
            transfer
                .token_id
                .as_ref()
                .map(|token_id| format!("token id {}", token_id.decimal))
        })
        .unwrap_or_else(|| "unknown amount".to_owned());
    TransactionImpactItem {
        kind: TransactionImpactKind::AssetTransfer,
        title: "Asset transfer".to_owned(),
        message: format!(
            "moves {amount} of {} from {from} to {to}",
            transfer.asset_kind
        ),
        severity: WarningSeverity::Info,
    }
}

fn transaction_impact_title(
    native_value: bool,
    transfer_count: usize,
    approval_count: usize,
    live_simulated: bool,
) -> String {
    let mut parts = Vec::new();
    if native_value {
        parts.push("native value".to_owned());
    }
    if transfer_count > 0 {
        parts.push(plural_count(transfer_count, "transfer"));
    }
    if approval_count > 0 {
        parts.push(plural_count(approval_count, "approval"));
    }
    if parts.is_empty() {
        if live_simulated {
            "No asset movement reported".to_owned()
        } else {
            "No decoded asset movement".to_owned()
        }
    } else {
        format!("Impact: {}", parts.join(", "))
    }
}

fn token_amount_impact_label(amount: &TokenAmount) -> String {
    if is_max_u256(&amount.hex) {
        "unlimited amount".to_owned()
    } else {
        amount.decimal.clone()
    }
}

fn display_address(value: &str) -> String {
    if value.len() >= 14 {
        format!("{}...{}", &value[..8], &value[value.len() - 6..])
    } else {
        value.to_owned()
    }
}

fn plural_count(count: usize, label: &str) -> String {
    if count == 1 {
        format!("1 {label}")
    } else {
        format!("{count} {label}s")
    }
}

pub fn evaluate_transaction_trust(report: &TransactionSimulationReport) -> TransactionTrustSummary {
    let mut items = Vec::new();

    if let Some(to) = &report.transaction.to {
        items.push(counterparty_trust_item(
            &report.chain_id,
            TransactionTrustRole::TransactionTo,
            Some(to),
        ));
    } else {
        items.push(counterparty_trust_item(
            &report.chain_id,
            TransactionTrustRole::TransactionTo,
            None,
        ));
    }

    for approval in report.approvals.iter().take(8) {
        if approval_uses_spender(approval) {
            items.push(counterparty_trust_item(
                &report.chain_id,
                TransactionTrustRole::ApprovalSpender,
                approval.spender.as_deref(),
            ));
        }
        if approval_uses_operator(approval) {
            items.push(counterparty_trust_item(
                &report.chain_id,
                TransactionTrustRole::ApprovalOperator,
                approval.operator.as_deref(),
            ));
        }
    }

    let known_count = items
        .iter()
        .filter(|item| item.status == TransactionTrustStatus::Known)
        .count();
    let unknown_count = items
        .iter()
        .filter(|item| item.status == TransactionTrustStatus::Unknown)
        .count();
    let level = if items.is_empty()
        || items
            .iter()
            .all(|item| item.status == TransactionTrustStatus::Missing)
    {
        TransactionTrustLevel::NoCounterparty
    } else if unknown_count == 0 && known_count > 0 {
        TransactionTrustLevel::Recognized
    } else if known_count > 0 {
        TransactionTrustLevel::Mixed
    } else {
        TransactionTrustLevel::Unrecognized
    };

    TransactionTrustSummary {
        title: transaction_trust_title(level, known_count, unknown_count),
        level,
        known_count,
        unknown_count,
        items,
    }
}

fn counterparty_trust_item(
    chain_id: &str,
    role: TransactionTrustRole,
    address: Option<&str>,
) -> TransactionTrustItem {
    let Some(address) = address else {
        return TransactionTrustItem {
            role,
            address: None,
            label: None,
            protocol: None,
            status: TransactionTrustStatus::Missing,
            message: format!(
                "{} is not present in the transaction",
                trust_role_label(role)
            ),
            severity: WarningSeverity::Info,
        };
    };

    if let Some(known) = known_counterparty(chain_id, address) {
        return TransactionTrustItem {
            role,
            address: Some(address.to_ascii_lowercase()),
            label: Some(known.label.to_owned()),
            protocol: Some(known.protocol.to_owned()),
            status: TransactionTrustStatus::Known,
            message: format!(
                "{} matches known {} contract {}",
                trust_role_label(role),
                known.protocol,
                known.label
            ),
            severity: WarningSeverity::Info,
        };
    }

    TransactionTrustItem {
        role,
        address: Some(address.to_ascii_lowercase()),
        label: None,
        protocol: None,
        status: TransactionTrustStatus::Unknown,
        message: format!(
            "{} {} is not in the current known-counterparty registry",
            trust_role_label(role),
            display_address(address)
        ),
        severity: match role {
            TransactionTrustRole::ApprovalSpender | TransactionTrustRole::ApprovalOperator => {
                WarningSeverity::Warning
            }
            TransactionTrustRole::TransactionTo => WarningSeverity::Info,
        },
    }
}

fn transaction_trust_title(
    level: TransactionTrustLevel,
    known_count: usize,
    unknown_count: usize,
) -> String {
    match level {
        TransactionTrustLevel::NoCounterparty => "No counterparty address".to_owned(),
        TransactionTrustLevel::Recognized => format!("Known counterparties: {known_count}"),
        TransactionTrustLevel::Mixed => {
            format!("Mixed counterparties: {known_count} known, {unknown_count} unknown")
        }
        TransactionTrustLevel::Unrecognized => format!("Unknown counterparties: {unknown_count}"),
    }
}

fn trust_role_label(role: TransactionTrustRole) -> &'static str {
    match role {
        TransactionTrustRole::TransactionTo => "Transaction recipient",
        TransactionTrustRole::ApprovalSpender => "Approval spender",
        TransactionTrustRole::ApprovalOperator => "Approval operator",
    }
}

fn unknown_active_approval_authority(chain_id: &str, approval: &ApprovalChange) -> bool {
    if approval_uses_spender(approval) {
        return approval
            .spender
            .as_deref()
            .is_none_or(|spender| known_counterparty(chain_id, spender).is_none());
    }
    if approval_uses_operator(approval) {
        return approval
            .operator
            .as_deref()
            .is_none_or(|operator| known_counterparty(chain_id, operator).is_none());
    }
    false
}

fn approval_uses_spender(approval: &ApprovalChange) -> bool {
    if approval.spender.is_none() {
        return false;
    }
    approval
        .amount
        .as_ref()
        .is_none_or(|amount| amount.decimal != "0")
        && approval.approved != Some(false)
}

fn approval_uses_operator(approval: &ApprovalChange) -> bool {
    approval.operator.is_some() && approval.approved.unwrap_or(true)
}

pub fn evaluate_transaction_risk(
    report: &TransactionSimulationReport,
    policy: &TransactionPolicyEvaluation,
) -> TransactionRiskSummary {
    let mut reasons = Vec::new();
    for blocker in &policy.blockers {
        reasons.push(TransactionRiskReason {
            source: "policy".to_owned(),
            code: blocker.code.clone(),
            title: policy_blocker_title(&blocker.code).to_owned(),
            message: blocker.message.clone(),
            severity: if blocker.overrideable {
                WarningSeverity::Warning
            } else {
                WarningSeverity::Error
            },
        });
    }

    for warning in report
        .warnings
        .iter()
        .filter(|warning| !policy_blockers_cover_warning(&policy.blockers, &warning.code))
    {
        reasons.push(TransactionRiskReason {
            source: "simulation".to_owned(),
            code: warning.code.clone(),
            title: simulation_warning_title(&warning.code).to_owned(),
            message: warning.message.clone(),
            severity: warning.severity,
        });
    }

    if let Some(reason) = protocol_intent_reason(report) {
        reasons.push(reason);
    }
    if policy.can_sign {
        reasons.push(TransactionRiskReason {
            source: "simulation".to_owned(),
            code: "live_simulation_present".to_owned(),
            title: "Live simulation present".to_owned(),
            message: "a live transaction simulation result is attached".to_owned(),
            severity: WarningSeverity::Info,
        });
    }

    let action = if policy.can_sign {
        TransactionRiskAction::OrdinaryApproval
    } else if policy.override_allowed {
        TransactionRiskAction::HighRiskApproval
    } else {
        TransactionRiskAction::Blocked
    };

    let has_non_overrideable_blocker = policy.blockers.iter().any(|blocker| !blocker.overrideable);
    let has_high_risk_override = policy.blockers.iter().any(|blocker| {
        matches!(
            blocker.code.as_str(),
            "unknown_calldata"
                | "high_risk_unlimited_approval"
                | "high_risk_operator_approval"
                | "unknown_approval_authority"
                | "universal_router_semantics_incomplete"
                | "multicall_semantics_incomplete"
                | "uniswap_zero_slippage_floor"
                | "swap_deadline_expired"
                | "swap_deadline_too_far"
                | "swap_deadline_invalid"
                | "swap_third_party_recipient"
                | "aave_third_party_account"
                | "aave_third_party_withdraw_recipient"
                | "aave_borrow_health_factor_unknown"
                | "aave_withdraw_health_factor_unknown"
                | "aave_collateral_disable_risk"
                | "aave_post_transaction_health_factor_unknown"
                | "aave_health_factor_caution"
        )
    });
    let has_warning = report
        .warnings
        .iter()
        .any(|warning| warning.severity == WarningSeverity::Warning);
    let level = if has_non_overrideable_blocker {
        TransactionRiskLevel::Blocked
    } else if has_high_risk_override {
        TransactionRiskLevel::High
    } else if policy.override_allowed || has_warning {
        TransactionRiskLevel::Caution
    } else {
        TransactionRiskLevel::Low
    };

    let (title, message) = match level {
        TransactionRiskLevel::Low => (
            "Ready for ordinary approval",
            "live simulation succeeded and policy found no blockers",
        ),
        TransactionRiskLevel::Caution if policy.override_allowed => (
            "High-risk confirmation required",
            "no live simulation result is attached, so explicit high-risk approval is required",
        ),
        TransactionRiskLevel::Caution => (
            "Review before signing",
            "policy allows signing, but simulation warnings should be reviewed first",
        ),
        TransactionRiskLevel::High => (
            "High-risk transaction",
            "review the policy reasons before using explicit high-risk approval",
        ),
        TransactionRiskLevel::Blocked => (
            "Blocked transaction",
            "policy does not allow this transaction to reach signing",
        ),
    };

    TransactionRiskSummary {
        level,
        action,
        title: title.to_owned(),
        message: message.to_owned(),
        reasons,
    }
}

fn policy_blocker_title(code: &str) -> &'static str {
    match code {
        "live_simulation_required" => "No live simulation",
        "invalid_transaction_request" => "Invalid transaction",
        "simulation_provider_failed" => "Simulation failed",
        "unknown_calldata" => "Unknown calldata",
        "high_risk_unlimited_approval" => "Unlimited token approval",
        "high_risk_operator_approval" => "Approval for all",
        "unknown_approval_authority" => "Unknown approval authority",
        "universal_router_semantics_incomplete" => "Universal Router review incomplete",
        "multicall_semantics_incomplete" => "Multicall review incomplete",
        "uniswap_zero_slippage_floor" => "Zero swap output minimum",
        "swap_deadline_expired" => "Expired swap deadline",
        "swap_deadline_too_far" => "Long swap deadline",
        "swap_deadline_invalid" => "Invalid swap deadline",
        "swap_third_party_recipient" => "Third-party swap recipient",
        "aave_third_party_account" => "Third-party Aave account",
        "aave_third_party_withdraw_recipient" => "Third-party Aave withdraw recipient",
        "aave_borrow_health_factor_unknown" => "Aave borrow risk",
        "aave_withdraw_health_factor_unknown" => "Aave withdraw risk",
        "aave_collateral_disable_risk" => "Aave collateral risk",
        "aave_post_transaction_health_factor_unknown" => "Aave post-transaction risk",
        "aave_health_factor_caution" => "Aave health factor caution",
        "aave_health_factor_liquidation_risk" => "Aave liquidation risk",
        _ => "Policy reason",
    }
}

fn simulation_warning_title(code: &str) -> &'static str {
    match code {
        "native_value_transfer" => "Native value transfer",
        "empty_transaction" => "Empty transaction",
        "malformed_calldata" => "Malformed calldata",
        "unknown_function_selector" => "Unknown function",
        "unlimited_token_approval" => "Unlimited token approval",
        "operator_approval_for_all" => "Approval for all",
        "provider_asset_change_ignored" => "Provider asset change ignored",
        "provider_asset_changes_truncated" => "Provider asset changes truncated",
        "provider_asset_changes_unrecognized" => "Provider asset changes unrecognized",
        _ => "Simulation warning",
    }
}

fn policy_blockers_cover_warning(blockers: &[PolicyBlocker], warning_code: &str) -> bool {
    blockers.iter().any(|blocker| {
        matches!(
            (blocker.code.as_str(), warning_code),
            (
                "invalid_transaction_request",
                "invalid_transaction_params" | "invalid_transaction_field" | "malformed_calldata"
            ) | (
                "simulation_provider_failed",
                "simulation_client_error"
                    | "simulation_provider_unavailable"
                    | "simulation_provider_response_unreadable"
                    | "simulation_provider_response_malformed"
                    | "simulation_provider_http_error"
                    | "simulation_provider_error"
                    | "simulation_provider_result_error"
            ) | ("unknown_calldata", "unknown_function_selector")
                | ("high_risk_unlimited_approval", "unlimited_token_approval")
                | ("high_risk_operator_approval", "operator_approval_for_all")
        )
    })
}

fn protocol_intent_reason(report: &TransactionSimulationReport) -> Option<TransactionRiskReason> {
    let call = report.decoded_call.as_ref()?;
    let protocol = protocol_standard_title(&call.standard)?;
    Some(TransactionRiskReason {
        source: "intent".to_owned(),
        code: "protocol_intent_decoded".to_owned(),
        title: "Protocol intent decoded".to_owned(),
        message: format!("top-level call recognized as {protocol}"),
        severity: WarningSeverity::Info,
    })
}

fn protocol_standard_title(standard: &str) -> Option<&'static str> {
    match standard {
        "uniswap_v2_router" => Some("Uniswap V2 Router"),
        "uniswap_v3_swap_router" => Some("Uniswap V3 Swap Router"),
        "uniswap_universal_router" => Some("Uniswap Universal Router"),
        "aave_v3_pool" => Some("Aave V3 Pool"),
        "multicall" => Some("Multicall"),
        _ => None,
    }
}
