use super::*;
use crate::assessment::known_counterparty;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
};

use serde_json::{Value, json};

#[test]
fn decodes_erc20_approve_with_unlimited_warning() {
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x000000000000000000000000000000000000000b",
                "value": "0x0",
                "data": concat!(
                    "0x095ea7b3",
                    "000000000000000000000000000000000000000000000000000000000000000c",
                    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
            }
        ]),
        "0x1",
    );
    let report = &review.simulation;

    assert_eq!(report.status, SimulationStatus::LocalWarnings);
    assert_eq!(
        report
            .decoded_call
            .as_ref()
            .map(|call| call.function.as_str()),
        Some("approve(address,uint256)")
    );
    assert_eq!(report.approvals.len(), 1);
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.code == "unlimited_token_approval")
    );
    assert_eq!(
        review.policy.decision,
        TransactionPolicyDecision::RequiresUserOverride
    );
    assert!(!review.policy.can_sign);
    assert!(review.policy.override_allowed);
    assert!(
        review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "live_simulation_required" && blocker.overrideable)
    );
    assert!(
        review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "high_risk_unlimited_approval" && blocker.overrideable)
    );
    assert!(
        review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "unknown_approval_authority" && blocker.overrideable)
    );
    assert_eq!(review.risk.level, TransactionRiskLevel::High);
    assert_eq!(review.risk.action, TransactionRiskAction::HighRiskApproval);
    assert!(risk_reason(&review, "high_risk_unlimited_approval").is_some());
    assert!(risk_reason(&review, "unknown_approval_authority").is_some());
    assert_eq!(review.trust.level, TransactionTrustLevel::Unrecognized);
    assert!(
        trust_item(
            &review,
            TransactionTrustRole::ApprovalSpender,
            TransactionTrustStatus::Unknown
        )
        .is_some()
    );
    assert_eq!(review.impact.approval_count, 1);
    assert_eq!(review.impact.transfer_count, 0);
    assert!(
        impact_item(
            &review,
            TransactionImpactKind::Approval,
            "Unlimited token approval"
        )
        .is_some()
    );
}

#[test]
fn impact_summary_marks_empty_local_request() {
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x000000000000000000000000000000000000000b",
                "value": "0x0",
                "data": "0x"
            }
        ]),
        "0x1",
    );

    assert_eq!(review.impact.title, "No decoded asset movement");
    assert!(!review.impact.native_value);
    assert_eq!(review.impact.transfer_count, 0);
    assert_eq!(review.impact.approval_count, 0);
    assert!(
        impact_item(
            &review,
            TransactionImpactKind::NoAssetMovement,
            "No decoded asset movement"
        )
        .is_some()
    );
}

#[test]
fn impact_summary_marks_unlimited_approval() {
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x000000000000000000000000000000000000000b",
                "data": concat!(
                    "0x095ea7b3",
                    "000000000000000000000000000000000000000000000000000000000000000c",
                    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
            }
        ]),
        "0x1",
    );

    assert_eq!(review.impact.approval_count, 1);
    assert_eq!(review.impact.transfer_count, 0);
    let item = impact_item(
        &review,
        TransactionImpactKind::Approval,
        "Unlimited token approval",
    )
    .unwrap();
    assert_eq!(item.severity, WarningSeverity::Warning);
    assert!(item.message.contains("unlimited amount"));
}

#[test]
fn recognizes_uniswap_universal_router_recipient() {
    let data = format!(
        "0x3593564c{}{}{}{}{}{}",
        abi_u256(96),
        abi_u256(160),
        abi_u256(123),
        abi_u256(2),
        abi_bytes_word("0001"),
        abi_u256(0),
    );
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );

    assert_eq!(review.trust.level, TransactionTrustLevel::Recognized);
    assert_eq!(review.trust.known_count, 1);
    let item = trust_item(
        &review,
        TransactionTrustRole::TransactionTo,
        TransactionTrustStatus::Known,
    )
    .unwrap();
    assert_eq!(item.protocol.as_deref(), Some("Uniswap"));
    assert_eq!(item.label.as_deref(), Some("Universal Router"));
}

#[test]
fn recognizes_aave_v3_pool_recipient() {
    let asset = "0x1111111111111111111111111111111111111111";
    let on_behalf_of = "0x2222222222222222222222222222222222222222";
    let data = format!(
        "0x617ba037{}{}{}{}",
        abi_address(asset),
        abi_u256(50_000_000),
        abi_address(on_behalf_of),
        abi_u256(0),
    );
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );

    assert_eq!(review.trust.level, TransactionTrustLevel::Recognized);
    let item = trust_item(
        &review,
        TransactionTrustRole::TransactionTo,
        TransactionTrustStatus::Known,
    )
    .unwrap();
    assert_eq!(item.protocol.as_deref(), Some("Aave"));
    assert_eq!(item.label.as_deref(), Some("V3 Pool"));
}

#[test]
fn unknown_active_approval_authority_requires_high_risk_override() {
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x000000000000000000000000000000000000000b",
                "value": "0x0",
                "data": concat!(
                    "0x095ea7b3",
                    "000000000000000000000000000000000000000000000000000000000000000c",
                    "0000000000000000000000000000000000000000000000000000000000000064"
                )
            }
        ]),
        "0x1",
    );
    mark_live_simulated(&mut review);

    assert_eq!(
        review.policy.decision,
        TransactionPolicyDecision::RequiresUserOverride
    );
    assert!(!review.policy.can_sign);
    assert!(review.policy.override_allowed);
    assert!(
        review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "unknown_approval_authority" && blocker.overrideable)
    );
    assert_eq!(review.risk.level, TransactionRiskLevel::High);
    assert!(risk_reason(&review, "unknown_approval_authority").is_some());
    assert_eq!(review.trust.level, TransactionTrustLevel::Unrecognized);
}

#[test]
fn known_permit2_approval_can_use_ordinary_approval_with_live_simulation() {
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x000000000000000000000000000000000000000b",
                "value": "0x0",
                "data": concat!(
                    "0x095ea7b3",
                    "000000000000000000000000000000000022d473030f116ddee9f6b43ac78ba3",
                    "0000000000000000000000000000000000000000000000000000000000000064"
                )
            }
        ]),
        "0x1",
    );
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Allowed);
    assert!(review.policy.can_sign);
    assert_eq!(review.risk.level, TransactionRiskLevel::Low);
    assert_eq!(review.trust.level, TransactionTrustLevel::Mixed);
    let item = trust_item(
        &review,
        TransactionTrustRole::ApprovalSpender,
        TransactionTrustStatus::Known,
    )
    .unwrap();
    assert_eq!(item.protocol.as_deref(), Some("Uniswap"));
    assert_eq!(item.label.as_deref(), Some("Permit2"));
}

#[test]
fn known_counterparty_registry_covers_switchable_uniswap_chains() {
    let cases = [
        (
            "0x1",
            "0x7a250d5630b4cf539739df2c5dacb4c659f2488d",
            "V2 Router02",
        ),
        (
            "0x1",
            "0xe592427a0aece92de3edee1f18e0157c05861564",
            "V3 SwapRouter",
        ),
        (
            "0x1",
            "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45",
            "SwapRouter02",
        ),
        (
            "0x1",
            "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
            "Universal Router",
        ),
        (
            "0x1",
            "0x4c82d1fbfe28c977cbb58d8c7ff8fcf9f70a2cca",
            "Universal Router 2.1.1",
        ),
        (
            "0xaa36a7",
            "0xee567fe1712faf6149d80da1e6934e354124cfe3",
            "V2 Router02",
        ),
        (
            "0xaa36a7",
            "0x3bfa4769fb09eefc5a80d6e87c3b9c650f7ae48e",
            "SwapRouter02",
        ),
        (
            "0xaa36a7",
            "0x3a9d48ab9751398bbfa63ad67599bb04e4bdf98b",
            "Universal Router",
        ),
        (
            "0x2105",
            "0x4752ba5dbc23f44d87826276bf6fd6b1c372ad24",
            "V2 Router02",
        ),
        (
            "0x2105",
            "0x2626664c2603336e57b271c5c0b26f421741e481",
            "SwapRouter02",
        ),
        (
            "0x2105",
            "0x6ff5693b99212da76ad316178a184ab56d299b43",
            "Universal Router",
        ),
        (
            "0x2105",
            "0xfdf682f51fe81aa4898f0ae2163d8a55c127fbc7",
            "Universal Router 2.1.1",
        ),
        (
            "0xa",
            "0x4a7b5da61326a6379179b40d00f57e5bbdc962c2",
            "V2 Router02",
        ),
        (
            "0xa",
            "0xe592427a0aece92de3edee1f18e0157c05861564",
            "V3 SwapRouter",
        ),
        (
            "0xa",
            "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45",
            "SwapRouter02",
        ),
        (
            "0xa",
            "0x851116d9223fabed8e56c0e6b8ad0c31d98b3507",
            "Universal Router",
        ),
        (
            "0xa",
            "0x8b844f885672f333bc0042cb669255f93a4c1e6b",
            "Universal Router 2.1.1",
        ),
        (
            "0xa4b1",
            "0x4752ba5dbc23f44d87826276bf6fd6b1c372ad24",
            "V2 Router02",
        ),
        (
            "0xa4b1",
            "0xe592427a0aece92de3edee1f18e0157c05861564",
            "V3 SwapRouter",
        ),
        (
            "0xa4b1",
            "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45",
            "SwapRouter02",
        ),
        (
            "0xa4b1",
            "0xa51afafe0263b40edaef0df8781ea9aa03e381a3",
            "Universal Router",
        ),
        (
            "0xa4b1",
            "0x8b844f885672f333bc0042cb669255f93a4c1e6b",
            "Universal Router 2.1.1",
        ),
        (
            "0x89",
            "0xedf6066a2b290c185783862c7f4776a2c8077ad1",
            "V2 Router02",
        ),
        (
            "0x89",
            "0xe592427a0aece92de3edee1f18e0157c05861564",
            "V3 SwapRouter",
        ),
        (
            "0x89",
            "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45",
            "SwapRouter02",
        ),
        (
            "0x89",
            "0x1095692a6237d83c6a72f3f5efedb9a670c49223",
            "Universal Router",
        ),
        (
            "0x89",
            "0x8b844f885672f333bc0042cb669255f93a4c1e6b",
            "Universal Router 2.1.1",
        ),
    ];

    for (chain_id, address, label) in cases {
        let known = known_counterparty(chain_id, address)
            .unwrap_or_else(|| panic!("missing {chain_id} {address}"));
        assert_eq!(known.protocol, "Uniswap");
        assert_eq!(known.label, label);
    }
}

#[test]
fn known_counterparty_registry_covers_switchable_permit2_and_aave_pools() {
    let cases = [
        ("0x1", "0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2"),
        ("0xaa36a7", "0x6ae43d3271ff6888e7fc43fd7321a503ff738951"),
        ("0x2105", "0xa238dd80c259a72e81d7e4664a9801593f98d1c5"),
        ("0xa", "0x794a61358d6845594f94dc1db02a252b5b4814ad"),
        ("0xa4b1", "0x794a61358d6845594f94dc1db02a252b5b4814ad"),
        ("0x89", "0x794a61358d6845594f94dc1db02a252b5b4814ad"),
    ];

    for (chain_id, aave_pool) in cases {
        let permit2 = known_counterparty(chain_id, "0x000000000022d473030f116ddee9f6b43ac78ba3")
            .unwrap_or_else(|| panic!("missing Permit2 for {chain_id}"));
        assert_eq!(permit2.protocol, "Uniswap");
        assert_eq!(permit2.label, "Permit2");

        let aave = known_counterparty(chain_id, aave_pool)
            .unwrap_or_else(|| panic!("missing Aave pool for {chain_id}"));
        assert_eq!(aave.protocol, "Aave");
        assert_eq!(aave.label, "V3 Pool");
    }
}

#[test]
fn decodes_uniswap_v2_swap_path_intent() {
    let token_in = "0x1111111111111111111111111111111111111111";
    let token_out = "0x2222222222222222222222222222222222222222";
    let recipient = "0x3333333333333333333333333333333333333333";
    let data = format!(
        "0x38ed1739{}{}{}{}{}{}{}{}",
        abi_u256(100),
        abi_u256(90),
        abi_u256(160),
        abi_address(recipient),
        abi_u256(999),
        abi_u256(2),
        abi_address(token_in),
        abi_address(token_out),
    );
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x4444444444444444444444444444444444444444",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );

    let report = &review.simulation;
    let call = report.decoded_call.as_ref().unwrap();
    assert_eq!(report.status, SimulationStatus::LocalDecoded);
    assert_eq!(call.standard, "uniswap_v2_router");
    assert_eq!(
        call.function,
        "swapExactTokensForTokens(uint256,uint256,address[],address,uint256)"
    );
    assert_eq!(decoded_arg(call, "pathLength"), Some("2"));
    assert_eq!(decoded_arg(call, "pathFirst"), Some(token_in));
    assert_eq!(decoded_arg(call, "pathLast"), Some(token_out));
    assert_no_unknown_selector(report);
    assert_live_simulation_only_override(&review);
    assert_eq!(review.risk.level, TransactionRiskLevel::Caution);
    assert_eq!(review.risk.action, TransactionRiskAction::HighRiskApproval);
    assert!(risk_reason(&review, "protocol_intent_decoded").is_some());
}

#[test]
fn malformed_uniswap_v2_dynamic_path_fails_closed() {
    let recipient = "0x3333333333333333333333333333333333333333";
    let data = format!(
        "0x38ed1739{}{}{}{}{}",
        abi_u256(100),
        abi_u256(90),
        abi_u256(160),
        abi_address(recipient),
        abi_u256(999),
    );
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x4444444444444444444444444444444444444444",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );

    let report = &review.simulation;
    assert_eq!(report.status, SimulationStatus::InvalidRequest);
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.code == "uniswap_v2_path_malformed")
    );
    assert_no_unknown_selector(report);
    assert!(
        review.policy.blockers.iter().any(|blocker| {
            blocker.code == "invalid_transaction_request" && !blocker.overrideable
        })
    );
    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
    assert_eq!(review.risk.action, TransactionRiskAction::Blocked);
}

#[test]
fn decodes_uniswap_v3_exact_input_single_intent() {
    let token_in = "0x1111111111111111111111111111111111111111";
    let token_out = "0x2222222222222222222222222222222222222222";
    let recipient = "0x3333333333333333333333333333333333333333";
    let data = format!(
        "0x414bf389{}{}{}{}{}{}{}{}",
        abi_address(token_in),
        abi_address(token_out),
        abi_u256(3000),
        abi_address(recipient),
        abi_u256(12345),
        abi_u256(1_000_000),
        abi_u256(990_000),
        abi_u256(0),
    );
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x4444444444444444444444444444444444444444",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );

    let report = &review.simulation;
    let call = report.decoded_call.as_ref().unwrap();
    assert_eq!(report.status, SimulationStatus::LocalDecoded);
    assert_eq!(call.standard, "uniswap_v3_swap_router");
    assert_eq!(
        call.function,
        "exactInputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))"
    );
    assert_eq!(decoded_arg(call, "tokenIn"), Some(token_in));
    assert_eq!(decoded_arg(call, "tokenOut"), Some(token_out));
    assert_eq!(decoded_arg(call, "fee"), Some("3000"));
    assert_eq!(decoded_arg(call, "amountIn"), Some("1000000"));
    assert_no_unknown_selector(report);
    assert_live_simulation_only_override(&review);
}

#[test]
fn decodes_uniswap_universal_router_execute_without_raw_payload() {
    let data = format!(
        "0x3593564c{}{}{}{}{}{}",
        abi_u256(96),
        abi_u256(160),
        abi_u256(123),
        abi_u256(2),
        abi_bytes_word("0001"),
        abi_u256(0),
    );
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x4444444444444444444444444444444444444444",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );

    let report = &review.simulation;
    let call = report.decoded_call.as_ref().unwrap();
    assert_eq!(report.status, SimulationStatus::LocalDecoded);
    assert_eq!(call.standard, "uniswap_universal_router");
    assert_eq!(call.function, "execute(bytes,bytes[],uint256)");
    assert_eq!(decoded_arg(call, "commandBytes"), Some("2"));
    assert_eq!(decoded_arg(call, "inputCount"), Some("0"));
    assert_eq!(decoded_arg(call, "deadline"), Some("123"));
    assert!(
        call.arguments
            .iter()
            .all(|argument| argument.value != "0001")
    );
    assert_no_unknown_selector(report);
    assert_live_simulation_only_override(&review);
}

#[test]
fn decodes_aave_supply_intent() {
    let asset = "0x1111111111111111111111111111111111111111";
    let on_behalf_of = "0x2222222222222222222222222222222222222222";
    let data = format!(
        "0x617ba037{}{}{}{}",
        abi_address(asset),
        abi_u256(50_000_000),
        abi_address(on_behalf_of),
        abi_u256(0),
    );
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x3333333333333333333333333333333333333333",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );

    let report = &review.simulation;
    let call = report.decoded_call.as_ref().unwrap();
    assert_eq!(report.status, SimulationStatus::LocalDecoded);
    assert_eq!(call.standard, "aave_v3_pool");
    assert_eq!(call.function, "supply(address,uint256,address,uint16)");
    assert_eq!(decoded_arg(call, "asset"), Some(asset));
    assert_eq!(decoded_arg(call, "amount"), Some("50000000"));
    assert_eq!(decoded_arg(call, "onBehalfOf"), Some(on_behalf_of));
    assert_no_unknown_selector(report);
    assert_live_simulation_only_override(&review);
}

#[test]
fn decodes_native_transfer() {
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x000000000000000000000000000000000000000b",
                "value": "0xde0b6b3a7640000",
                "data": "0x"
            }
        ]),
        "0x1",
    );
    let report = &review.simulation;

    assert_eq!(
        report.transaction.value.as_ref().unwrap().decimal,
        "1000000000000000000"
    );
    assert!(report.native_value.is_some());
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.code == "native_value_transfer")
    );
    assert!(review.impact.native_value);
    assert_eq!(review.impact.transfer_count, 0);
    assert_eq!(review.impact.approval_count, 0);
    assert!(
        impact_item(
            &review,
            TransactionImpactKind::NativeValue,
            "Native value transfer"
        )
        .is_some()
    );
}

#[test]
fn warns_on_unknown_selector() {
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "to": "0x000000000000000000000000000000000000000b",
                "data": "0x12345678"
            }
        ]),
        "0x1",
    );
    let report = &review.simulation;

    assert_eq!(report.status, SimulationStatus::LocalWarnings);
    assert_eq!(
        report
            .decoded_call
            .as_ref()
            .and_then(|call| call.selector.as_deref()),
        Some("0x12345678")
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.code == "unknown_function_selector")
    );
    assert!(
        review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "unknown_calldata" && blocker.overrideable)
    );
    assert_eq!(
        review.policy.decision,
        TransactionPolicyDecision::RequiresUserOverride
    );
    assert!(review.policy.override_allowed);
    assert_eq!(review.risk.level, TransactionRiskLevel::High);
    assert_eq!(review.risk.action, TransactionRiskAction::HighRiskApproval);
    assert!(risk_reason(&review, "unknown_calldata").is_some());
}

#[test]
fn invalid_params_stay_reportable() {
    let review = local_transaction_review("eth_sendTransaction", &json!([]), "0x1");
    let report = &review.simulation;
    assert_eq!(report.status, SimulationStatus::InvalidRequest);
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.code == "invalid_transaction_params")
    );
    assert!(
        review.policy.blockers.iter().any(|blocker| {
            blocker.code == "invalid_transaction_request" && !blocker.overrideable
        })
    );
    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.override_allowed);
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
    assert_eq!(review.risk.action, TransactionRiskAction::Blocked);
}

#[test]
fn risk_summary_marks_live_simulated_request_low() {
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x000000000000000000000000000000000000000b",
                "value": "0x0",
                "data": "0x"
            }
        ]),
        "0x1",
    );
    review.simulation.mode = SimulationMode::AlchemyRpc;
    review.simulation.status = SimulationStatus::ProviderSimulated;
    review.simulation.raw_provider_response = Some(json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": {
            "changes": [],
            "error": null
        }
    }));
    review.policy = evaluate_transaction_policy(&review.simulation);
    review.risk = evaluate_transaction_risk(&review.simulation, &review.policy);
    review.impact = evaluate_transaction_impact(&review.simulation);
    review.trust = evaluate_transaction_trust(&review.simulation);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Allowed);
    assert_eq!(review.risk.level, TransactionRiskLevel::Low);
    assert_eq!(review.risk.action, TransactionRiskAction::OrdinaryApproval);
    assert!(risk_reason(&review, "live_simulation_present").is_some());
    assert!(review.impact.live_simulated);
    assert!(!review.impact.provider_asset_changes);
    assert_eq!(review.impact.title, "No asset movement reported");
}

#[test]
fn risk_summary_marks_provider_failure_blocked() {
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "to": "0x000000000000000000000000000000000000000b",
                "data": "0x"
            }
        ]),
        "0x1",
    );
    review.simulation.mode = SimulationMode::AlchemyRpc;
    review.simulation.status = SimulationStatus::ProviderFailed;
    review.policy = evaluate_transaction_policy(&review.simulation);
    review.risk = evaluate_transaction_risk(&review.simulation, &review.policy);
    review.impact = evaluate_transaction_impact(&review.simulation);
    review.trust = evaluate_transaction_trust(&review.simulation);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
    assert_eq!(review.risk.action, TransactionRiskAction::Blocked);
    assert!(risk_reason(&review, "simulation_provider_failed").is_some());
}

#[test]
fn simulation_client_trait_is_swappable() {
    #[derive(Debug)]
    struct FixtureClient;

    impl TransactionSimulationClient for FixtureClient {
        fn simulate_transaction(
            &self,
            request: TransactionSimulationRequest<'_>,
        ) -> TransactionSimulationReport {
            let mut report =
                local_transaction_report(request.method, request.params, request.default_chain_id);
            report.raw_provider_response = Some(json!({"fixture": true}));
            report
        }
    }

    let review = simulate_transaction_review(
        &FixtureClient,
        TransactionSimulationRequest {
            method: "eth_sendTransaction",
            params: &json!([{"data": "0x"}]),
            default_chain_id: "0x1",
        },
    );

    assert_eq!(
        review.simulation.raw_provider_response,
        Some(json!({"fixture": true}))
    );
    assert_eq!(
        review.policy.decision,
        TransactionPolicyDecision::RequiresUserOverride
    );
    assert!(review.policy.override_allowed);
    assert!(
        review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "live_simulation_required" && blocker.overrideable)
    );
    assert_eq!(review.risk.level, TransactionRiskLevel::Caution);
    assert_eq!(review.risk.action, TransactionRiskAction::HighRiskApproval);
}

#[test]
fn alchemy_rpc_adapter_posts_json_rpc_payload() {
    let (endpoint, request_body_rx) = spawn_fixture_http_server(
        200,
        json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": {
                "changes": [
                    {
                        "assetType": "ERC20",
                        "changeType": "TRANSFER",
                        "from": "0x000000000000000000000000000000000000000a",
                        "to": "0x000000000000000000000000000000000000000b",
                        "rawAmount": "1000000",
                        "contractAddress": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                        "tokenId": null,
                        "decimals": 6,
                        "symbol": "USDC",
                        "name": "USD Coin",
                        "amount": "1"
                    }
                ],
                "gasUsed": "0x5208",
                "error": null
            }
        }),
    );
    let client = AlchemyRpcSimulationClient::new(AlchemyRpcSimulationConfig {
        endpoint_url: endpoint,
        timeout_ms: 1_000,
        default_gas: "0x7a1200".to_owned(),
    });
    let request_params = json!([
        {
            "from": "0x000000000000000000000000000000000000000a",
            "to": "0x000000000000000000000000000000000000000b",
            "value": "0x0",
            "data": "0x",
        }
    ]);

    let review = simulate_transaction_review(
        &client,
        TransactionSimulationRequest {
            method: "eth_sendTransaction",
            params: &request_params,
            default_chain_id: "0x1",
        },
    );
    let request_body: Value = serde_json::from_str(&request_body_rx.recv().unwrap()).unwrap();

    assert_eq!(request_body["method"], "alchemy_simulateAssetChanges");
    assert_eq!(request_body["params"][0]["from"], request_params[0]["from"]);
    assert_eq!(request_body["params"][0]["gas"], "0x7a1200");
    assert_eq!(request_body["params"].as_array().unwrap().len(), 1);
    assert_eq!(review.simulation.mode, SimulationMode::AlchemyRpc);
    assert_eq!(
        review.simulation.status,
        SimulationStatus::ProviderSimulated
    );
    assert_eq!(review.simulation.asset_transfers.len(), 1);
    assert_eq!(
        review.simulation.asset_transfers[0],
        AssetTransfer {
            asset_kind: "erc20".to_owned(),
            contract: Some("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_owned()),
            from: Some("0x000000000000000000000000000000000000000a".to_owned()),
            to: Some("0x000000000000000000000000000000000000000b".to_owned()),
            amount: Some(TokenAmount {
                hex: "0xf4240".to_owned(),
                decimal: "1000000".to_owned(),
            }),
            token_id: None,
        }
    );
    assert_eq!(review.policy.decision, TransactionPolicyDecision::Allowed);
    assert!(review.policy.can_sign);
    assert!(review.policy.can_broadcast);
    assert!(!review.policy.override_allowed);
    assert!(review.policy.blockers.is_empty());
    assert!(
        !review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "live_simulation_required")
    );
    assert_eq!(review.risk.level, TransactionRiskLevel::Low);
    assert_eq!(review.risk.action, TransactionRiskAction::OrdinaryApproval);
    assert!(risk_reason(&review, "live_simulation_present").is_some());
    assert!(review.impact.live_simulated);
    assert!(review.impact.provider_asset_changes);
    assert_eq!(review.impact.transfer_count, 1);
    assert_eq!(review.impact.approval_count, 0);
    assert!(
        impact_item(
            &review,
            TransactionImpactKind::AssetTransfer,
            "Asset transfer"
        )
        .is_some()
    );
}

#[test]
fn alchemy_rpc_adapter_fails_closed_on_rpc_error() {
    let (endpoint, _request_body_rx) = spawn_fixture_http_server(
        200,
        json!({
            "id": 1,
            "jsonrpc": "2.0",
            "error": {
                "code": -32000,
                "message": "simulation failed"
            }
        }),
    );
    let client = AlchemyRpcSimulationClient::new(AlchemyRpcSimulationConfig {
        endpoint_url: endpoint,
        timeout_ms: 1_000,
        default_gas: "0x7a1200".to_owned(),
    });

    let review = simulate_transaction_review(
        &client,
        TransactionSimulationRequest {
            method: "eth_sendTransaction",
            params: &json!([{"to": "0x000000000000000000000000000000000000000b", "data": "0x"}]),
            default_chain_id: "0x1",
        },
    );

    assert_eq!(review.simulation.status, SimulationStatus::ProviderFailed);
    assert!(
        review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "simulation_provider_failed")
    );
    assert!(!review.policy.can_sign);
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
    assert_eq!(review.risk.action, TransactionRiskAction::Blocked);
}

#[test]
fn alchemy_rpc_adapter_fails_closed_when_changes_are_missing() {
    let (endpoint, _request_body_rx) = spawn_fixture_http_server(
        200,
        json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": {
                "gasUsed": "0x5208",
                "error": null
            }
        }),
    );
    let client = AlchemyRpcSimulationClient::new(AlchemyRpcSimulationConfig {
        endpoint_url: endpoint,
        timeout_ms: 1_000,
        default_gas: "0x7a1200".to_owned(),
    });

    let review = simulate_transaction_review(
        &client,
        TransactionSimulationRequest {
            method: "eth_sendTransaction",
            params: &json!([{"to": "0x000000000000000000000000000000000000000b", "data": "0x"}]),
            default_chain_id: "0x1",
        },
    );

    assert_eq!(review.simulation.status, SimulationStatus::ProviderFailed);
    assert!(
        review
            .simulation
            .warnings
            .iter()
            .any(|warning| warning.code == "simulation_provider_response_malformed")
    );
    assert!(
        review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "simulation_provider_failed")
    );
    assert!(!review.policy.can_sign);
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
    assert_eq!(review.risk.action, TransactionRiskAction::Blocked);
}

fn abi_u256(value: u128) -> String {
    format!("{value:064x}")
}

fn abi_address(value: &str) -> String {
    let address = value.strip_prefix("0x").unwrap_or(value);
    format!("{address:0>64}")
}

fn abi_bytes_word(hex_bytes: &str) -> String {
    format!("{hex_bytes:0<64}")
}

fn decoded_arg<'a>(call: &'a DecodedCall, name: &str) -> Option<&'a str> {
    call.arguments
        .iter()
        .find(|argument| argument.name == name)
        .map(|argument| argument.value.as_str())
}

fn risk_reason<'a>(
    review: &'a TransactionReviewReport,
    code: &str,
) -> Option<&'a TransactionRiskReason> {
    review
        .risk
        .reasons
        .iter()
        .find(|reason| reason.code == code)
}

fn impact_item<'a>(
    review: &'a TransactionReviewReport,
    kind: TransactionImpactKind,
    title: &str,
) -> Option<&'a TransactionImpactItem> {
    review
        .impact
        .items
        .iter()
        .find(|item| item.kind == kind && item.title == title)
}

fn trust_item(
    review: &TransactionReviewReport,
    role: TransactionTrustRole,
    status: TransactionTrustStatus,
) -> Option<&TransactionTrustItem> {
    review
        .trust
        .items
        .iter()
        .find(|item| item.role == role && item.status == status)
}

fn mark_live_simulated(review: &mut TransactionReviewReport) {
    review.simulation.mode = SimulationMode::AlchemyRpc;
    review.simulation.status = SimulationStatus::ProviderSimulated;
    review.simulation.raw_provider_response = Some(json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": {
            "changes": [],
            "error": null
        }
    }));
    review.policy = evaluate_transaction_policy(&review.simulation);
    review.risk = evaluate_transaction_risk(&review.simulation, &review.policy);
    review.impact = evaluate_transaction_impact(&review.simulation);
    review.trust = evaluate_transaction_trust(&review.simulation);
}

fn assert_no_unknown_selector(report: &TransactionSimulationReport) {
    assert!(
        !report
            .warnings
            .iter()
            .any(|warning| warning.code == "unknown_function_selector")
    );
}

fn assert_live_simulation_only_override(review: &TransactionReviewReport) {
    assert_eq!(
        review.policy.decision,
        TransactionPolicyDecision::RequiresUserOverride
    );
    assert!(
        review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "live_simulation_required" && blocker.overrideable)
    );
    assert!(
        !review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "unknown_calldata")
    );
}

fn spawn_fixture_http_server(status: u16, body: Value) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (request_tx, request_rx) = mpsc::channel();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = Vec::new();
        let mut chunk = [0_u8; 4096];
        loop {
            let read = stream.read(&mut chunk).unwrap();
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..read]);
            if request_body_from_http(&buffer).is_some() {
                break;
            }
        }
        let request_body = request_body_from_http(&buffer).unwrap_or_default();
        request_tx.send(request_body).unwrap();

        let body = serde_json::to_string(&body).unwrap();
        let response = format!(
            "HTTP/1.1 {status} OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (format!("http://{address}"), request_rx)
}

fn request_body_from_http(buffer: &[u8]) -> Option<String> {
    let marker = b"\r\n\r\n";
    let header_end = buffer
        .windows(marker.len())
        .position(|window| window == marker)?
        + marker.len();
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let content_length = headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse::<usize>().ok())?
    })?;
    if buffer.len() < header_end + content_length {
        return None;
    }
    String::from_utf8(buffer[header_end..header_end + content_length].to_vec()).ok()
}
