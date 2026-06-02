use crate::{
    decoder::{
        is_max_u256, looks_like_eth_address, overrideable_policy_blocker, policy_blocker,
        same_chain_id,
    },
    model::{
        ApprovalChange, AssetTransfer, PolicyBlocker, SimulationMode, SimulationStatus,
        TokenAmount, TransactionImpactItem, TransactionImpactKind, TransactionImpactSummary,
        TransactionPolicyDecision, TransactionPolicyEvaluation, TransactionRiskAction,
        TransactionRiskLevel, TransactionRiskReason, TransactionRiskSummary,
        TransactionSimulationReport, TransactionTrustItem, TransactionTrustLevel,
        TransactionTrustRole, TransactionTrustStatus, TransactionTrustSummary, WarningSeverity,
    },
};

pub fn evaluate_transaction_policy(
    report: &TransactionSimulationReport,
) -> TransactionPolicyEvaluation {
    let mut blockers = Vec::new();

    let live_simulated = report.mode == SimulationMode::AlchemyRpc
        && report.status == SimulationStatus::ProviderSimulated
        && report.raw_provider_response.is_some();
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

pub fn evaluate_transaction_impact(
    report: &TransactionSimulationReport,
) -> TransactionImpactSummary {
    let native_value = report.native_value.is_some();
    let transfer_count = report.asset_transfers.len();
    let approval_count = report.approvals.len();
    let live_simulated = report.mode == SimulationMode::AlchemyRpc
        && report.status == SimulationStatus::ProviderSimulated
        && report.raw_provider_response.is_some();
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct KnownCounterparty {
    pub(crate) chain_id: &'static str,
    pub(crate) address: &'static str,
    pub(crate) label: &'static str,
    pub(crate) protocol: &'static str,
}

// Source-backed protocol labels for the chains the desktop app can switch to.
// These labels are review/policy context; they are not a replacement for simulation.
const KNOWN_COUNTERPARTIES: &[KnownCounterparty] = &[
    KnownCounterparty {
        chain_id: "0x1",
        address: "0x7a250d5630b4cf539739df2c5dacb4c659f2488d",
        label: "V2 Router02",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x1",
        address: "0xe592427a0aece92de3edee1f18e0157c05861564",
        label: "V3 SwapRouter",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x1",
        address: "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45",
        label: "SwapRouter02",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x1",
        address: "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
        label: "Universal Router",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x1",
        address: "0x4c82d1fbfe28c977cbb58d8c7ff8fcf9f70a2cca",
        label: "Universal Router 2.1.1",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x1",
        address: "0x000000000022d473030f116ddee9f6b43ac78ba3",
        label: "Permit2",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x1",
        address: "0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2",
        label: "V3 Pool",
        protocol: "Aave",
    },
    KnownCounterparty {
        chain_id: "0xaa36a7",
        address: "0xee567fe1712faf6149d80da1e6934e354124cfe3",
        label: "V2 Router02",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xaa36a7",
        address: "0x3bfa4769fb09eefc5a80d6e87c3b9c650f7ae48e",
        label: "SwapRouter02",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xaa36a7",
        address: "0x3a9d48ab9751398bbfa63ad67599bb04e4bdf98b",
        label: "Universal Router",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xaa36a7",
        address: "0x000000000022d473030f116ddee9f6b43ac78ba3",
        label: "Permit2",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xaa36a7",
        address: "0x6ae43d3271ff6888e7fc43fd7321a503ff738951",
        label: "V3 Pool",
        protocol: "Aave",
    },
    KnownCounterparty {
        chain_id: "0x2105",
        address: "0x4752ba5dbc23f44d87826276bf6fd6b1c372ad24",
        label: "V2 Router02",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x2105",
        address: "0x2626664c2603336e57b271c5c0b26f421741e481",
        label: "SwapRouter02",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x2105",
        address: "0x6ff5693b99212da76ad316178a184ab56d299b43",
        label: "Universal Router",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x2105",
        address: "0xfdf682f51fe81aa4898f0ae2163d8a55c127fbc7",
        label: "Universal Router 2.1.1",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x2105",
        address: "0x000000000022d473030f116ddee9f6b43ac78ba3",
        label: "Permit2",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x2105",
        address: "0xa238dd80c259a72e81d7e4664a9801593f98d1c5",
        label: "V3 Pool",
        protocol: "Aave",
    },
    KnownCounterparty {
        chain_id: "0xa",
        address: "0x4a7b5da61326a6379179b40d00f57e5bbdc962c2",
        label: "V2 Router02",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xa",
        address: "0xe592427a0aece92de3edee1f18e0157c05861564",
        label: "V3 SwapRouter",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xa",
        address: "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45",
        label: "SwapRouter02",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xa",
        address: "0x851116d9223fabed8e56c0e6b8ad0c31d98b3507",
        label: "Universal Router",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xa",
        address: "0x8b844f885672f333bc0042cb669255f93a4c1e6b",
        label: "Universal Router 2.1.1",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xa",
        address: "0x000000000022d473030f116ddee9f6b43ac78ba3",
        label: "Permit2",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xa",
        address: "0x794a61358d6845594f94dc1db02a252b5b4814ad",
        label: "V3 Pool",
        protocol: "Aave",
    },
    KnownCounterparty {
        chain_id: "0xa4b1",
        address: "0x4752ba5dbc23f44d87826276bf6fd6b1c372ad24",
        label: "V2 Router02",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xa4b1",
        address: "0xe592427a0aece92de3edee1f18e0157c05861564",
        label: "V3 SwapRouter",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xa4b1",
        address: "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45",
        label: "SwapRouter02",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xa4b1",
        address: "0xa51afafe0263b40edaef0df8781ea9aa03e381a3",
        label: "Universal Router",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xa4b1",
        address: "0x8b844f885672f333bc0042cb669255f93a4c1e6b",
        label: "Universal Router 2.1.1",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xa4b1",
        address: "0x000000000022d473030f116ddee9f6b43ac78ba3",
        label: "Permit2",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0xa4b1",
        address: "0x794a61358d6845594f94dc1db02a252b5b4814ad",
        label: "V3 Pool",
        protocol: "Aave",
    },
    KnownCounterparty {
        chain_id: "0x89",
        address: "0xedf6066a2b290c185783862c7f4776a2c8077ad1",
        label: "V2 Router02",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x89",
        address: "0xe592427a0aece92de3edee1f18e0157c05861564",
        label: "V3 SwapRouter",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x89",
        address: "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45",
        label: "SwapRouter02",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x89",
        address: "0x1095692a6237d83c6a72f3f5efedb9a670c49223",
        label: "Universal Router",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x89",
        address: "0x8b844f885672f333bc0042cb669255f93a4c1e6b",
        label: "Universal Router 2.1.1",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x89",
        address: "0x000000000022d473030f116ddee9f6b43ac78ba3",
        label: "Permit2",
        protocol: "Uniswap",
    },
    KnownCounterparty {
        chain_id: "0x89",
        address: "0x794a61358d6845594f94dc1db02a252b5b4814ad",
        label: "V3 Pool",
        protocol: "Aave",
    },
];

pub(crate) fn known_counterparty(chain_id: &str, address: &str) -> Option<KnownCounterparty> {
    if !looks_like_eth_address(address) {
        return None;
    }
    let address = address.to_ascii_lowercase();
    KNOWN_COUNTERPARTIES.iter().copied().find(|known| {
        same_chain_id(chain_id, known.chain_id) && address.eq_ignore_ascii_case(known.address)
    })
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
