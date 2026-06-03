use super::*;
use crate::registry::known_counterparty;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::{SystemTime, UNIX_EPOCH},
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
    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.can_sign);
    assert!(!review.policy.override_allowed);
    assert!(
        review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "high_risk_unlimited_approval" && !blocker.overrideable)
    );
    assert!(
        review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "unknown_approval_authority" && !blocker.overrideable)
    );
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
    assert_eq!(review.risk.action, TransactionRiskAction::Blocked);
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
fn unknown_active_approval_authority_is_blocked() {
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

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.can_sign);
    assert!(!review.policy.override_allowed);
    assert!(
        review
            .policy
            .blockers
            .iter()
            .any(|blocker| blocker.code == "unknown_approval_authority" && !blocker.overrideable)
    );
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
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
    let recipient = "0x000000000000000000000000000000000000000a";
    let data = format!(
        "0x38ed1739{}{}{}{}{}{}{}{}",
        abi_u256(100),
        abi_u256(90),
        abi_u256(160),
        abi_address(recipient),
        abi_u256(future_swap_deadline()),
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
                "to": "0x7a250d5630b4cf539739df2c5dacb4c659f2488d",
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
    assert_local_allowlist_allowed(&review);
    assert_eq!(review.risk.level, TransactionRiskLevel::Low);
    assert_eq!(review.risk.action, TransactionRiskAction::OrdinaryApproval);
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
                "to": "0xe592427a0aece92de3edee1f18e0157c05861564",
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
    let recipient = "0x000000000000000000000000000000000000000a";
    let data = format!(
        "0x414bf389{}{}{}{}{}{}{}{}",
        abi_address(token_in),
        abi_address(token_out),
        abi_u256(3000),
        abi_address(recipient),
        abi_u256(future_swap_deadline()),
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
                "to": "0xe592427a0aece92de3edee1f18e0157c05861564",
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
    assert_local_allowlist_allowed(&review);
}

#[test]
fn live_uniswap_swap_with_safe_local_semantics_can_use_ordinary_approval() {
    let from = "0x000000000000000000000000000000000000000a";
    let token_in = "0x1111111111111111111111111111111111111111";
    let token_out = "0x2222222222222222222222222222222222222222";
    let data = format!(
        "0x414bf389{}{}{}{}{}{}{}{}",
        abi_address(token_in),
        abi_address(token_out),
        abi_u256(3000),
        abi_address(from),
        abi_u256(future_swap_deadline()),
        abi_u256(1_000_000),
        abi_u256(990_000),
        abi_u256(0),
    );
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": "0xe592427a0aece92de3edee1f18e0157c05861564",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Allowed);
    assert!(review.policy.can_sign);
    assert_eq!(review.risk.level, TransactionRiskLevel::Low);
}

#[test]
fn live_uniswap_zero_slippage_is_blocked() {
    let from = "0x000000000000000000000000000000000000000a";
    let token_in = "0x1111111111111111111111111111111111111111";
    let token_out = "0x2222222222222222222222222222222222222222";
    let data = format!(
        "0x414bf389{}{}{}{}{}{}{}{}",
        abi_address(token_in),
        abi_address(token_out),
        abi_u256(3000),
        abi_address(from),
        abi_u256(future_swap_deadline()),
        abi_u256(1_000_000),
        abi_u256(0),
        abi_u256(0),
    );
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": "0xe592427a0aece92de3edee1f18e0157c05861564",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.can_sign);
    assert!(!review.policy.override_allowed);
    assert!(risk_reason(&review, "uniswap_zero_slippage_floor").is_some());
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
}

#[test]
fn decodes_uniswap_universal_router_execute_without_raw_payload() {
    let from = "0x000000000000000000000000000000000000000a";
    let token_in = "0x1111111111111111111111111111111111111111";
    let token_out = "0x2222222222222222222222222222222222222222";
    let input = universal_router_v3_exact_in_input(
        from,
        1_000_000,
        990_000,
        &uniswap_v3_path_hex(token_in, 3000, token_out),
        true,
    );
    let data = format!(
        "0x3593564c{}",
        universal_router_execute_args("00", &[input.as_str()], Some(future_swap_deadline())),
    );
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
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
    assert_eq!(decoded_arg(call, "commandBytes"), Some("1"));
    assert_eq!(decoded_arg(call, "commandCount"), Some("1"));
    assert_eq!(decoded_arg(call, "inputCount"), Some("1"));
    assert_eq!(decoded_arg(call, "commandTypes"), Some("V3_SWAP_EXACT_IN"));
    assert_eq!(decoded_arg(call, "decodedCommandCount"), Some("1"));
    assert_eq!(decoded_arg(call, "unsupportedCommandCount"), Some("0"));
    assert_eq!(decoded_arg(call, "swapCount"), Some("1"));
    assert_eq!(decoded_arg(call, "recipient"), Some(from));
    assert_eq!(decoded_arg(call, "amountOutMinimum"), Some("990000"));
    assert_eq!(decoded_arg(call, "tokenIn"), Some(token_in));
    assert_eq!(decoded_arg(call, "tokenOut"), Some(token_out));
    assert!(
        call.arguments
            .iter()
            .all(|argument| argument.value != input)
    );
    assert_no_unknown_selector(report);
    assert_local_allowlist_allowed(&review);
}

#[test]
fn live_supported_universal_router_swap_can_use_ordinary_approval() {
    let from = "0x000000000000000000000000000000000000000a";
    let input = universal_router_v3_exact_in_input(
        from,
        1_000_000,
        990_000,
        &uniswap_v3_path_hex(
            "0x1111111111111111111111111111111111111111",
            3000,
            "0x2222222222222222222222222222222222222222",
        ),
        true,
    );
    let data = format!(
        "0x3593564c{}",
        universal_router_execute_args("00", &[input.as_str()], Some(future_swap_deadline())),
    );
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Allowed);
    assert!(review.policy.can_sign);
    assert!(risk_reason(&review, "universal_router_semantics_incomplete").is_none());
    assert_eq!(review.risk.level, TransactionRiskLevel::Low);
}

#[test]
fn live_universal_router_swap_without_deadline_is_blocked() {
    let from = "0x000000000000000000000000000000000000000a";
    let input = universal_router_v3_exact_in_input(
        from,
        1_000_000,
        990_000,
        &uniswap_v3_path_hex(
            "0x1111111111111111111111111111111111111111",
            3000,
            "0x2222222222222222222222222222222222222222",
        ),
        true,
    );
    let data = format!(
        "0x24856bc3{}",
        universal_router_execute_args("00", &[input.as_str()], None),
    );
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.can_sign);
    assert!(risk_reason(&review, "swap_deadline_missing").is_some());
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
}

#[test]
fn live_unsupported_universal_router_command_is_blocked() {
    let data = format!(
        "0x3593564c{}",
        universal_router_execute_args("1f", &[""], Some(future_swap_deadline())),
    );
    let mut review = local_transaction_review(
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
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(risk_reason(&review, "universal_router_semantics_incomplete").is_some());
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
}

#[test]
fn decodes_universal_router_permit2_permit_and_transfer_summary() {
    let from = "0x000000000000000000000000000000000000000a";
    let token = "0x1111111111111111111111111111111111111111";
    let recipient = "0x000000000000000000000000000000000000000b";
    let spender = "0x66a9893cc07d91d95644aedd05d03f95e1dba8af";
    let permit_input = universal_router_permit2_permit_input(
        token,
        1_000_000,
        future_swap_deadline(),
        7,
        spender,
        future_swap_deadline(),
        &"11".repeat(65),
    );
    let transfer_input = format!(
        "{}{}{}",
        abi_address(token),
        abi_address(recipient),
        abi_u256(1_000_000),
    );
    let data = format!(
        "0x3593564c{}",
        universal_router_execute_args(
            "0a02",
            &[permit_input.as_str(), transfer_input.as_str()],
            Some(future_swap_deadline()),
        ),
    );
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );

    let call = review.simulation.decoded_call.as_ref().unwrap();
    assert_eq!(decoded_arg(call, "permit2PermitCount"), Some("1"));
    assert_eq!(decoded_arg(call, "permit2TransferCount"), Some("1"));
    assert_eq!(decoded_arg(call, "permit2Token"), Some(token));
    assert_eq!(decoded_arg(call, "permit2Spender"), Some(spender));
    assert_eq!(decoded_arg(call, "permit2SignatureBytes"), Some("65"));
    assert_eq!(review.impact.approval_count, 1);
    assert_eq!(review.impact.transfer_count, 1);
    assert!(
        review
            .simulation
            .decoded_call
            .as_ref()
            .unwrap()
            .arguments
            .iter()
            .all(|argument| argument.value != permit_input)
    );
}

#[test]
fn live_universal_router_permit2_bounded_permit_can_use_ordinary_approval() {
    let mut review = universal_router_permit2_permit_review(
        &abi_u256(1_000_000),
        future_swap_deadline(),
        future_swap_deadline(),
    );
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Allowed);
    assert!(review.policy.can_sign);
    assert!(risk_reason(&review, "universal_router_permit2_unbounded_amount").is_none());
    assert!(risk_reason(&review, "universal_router_permit2_deadline_too_far").is_none());
}

#[test]
fn live_universal_router_permit2_max_amount_is_blocked() {
    let mut review = universal_router_permit2_permit_review(
        &max_u160_word(),
        future_swap_deadline(),
        future_swap_deadline(),
    );
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.can_sign);
    assert!(risk_reason(&review, "universal_router_permit2_unbounded_amount").is_some());
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
}

#[test]
fn live_universal_router_permit2_expiration_too_far_is_blocked() {
    let mut review = universal_router_permit2_permit_review(
        &abi_u256(1_000_000),
        far_permit_deadline(),
        future_swap_deadline(),
    );
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.can_sign);
    assert!(risk_reason(&review, "universal_router_permit2_deadline_too_far").is_some());
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
}

#[test]
fn live_universal_router_permit2_expired_sig_deadline_is_blocked() {
    let mut review =
        universal_router_permit2_permit_review(&abi_u256(1_000_000), future_swap_deadline(), 0);
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.can_sign);
    assert!(risk_reason(&review, "universal_router_permit2_sig_deadline_invalid").is_some());
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
}

#[test]
fn decodes_aave_supply_intent() {
    let asset = "0x1111111111111111111111111111111111111111";
    let on_behalf_of = "0x000000000000000000000000000000000000000a";
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

    let report = &review.simulation;
    let call = report.decoded_call.as_ref().unwrap();
    assert_eq!(report.status, SimulationStatus::LocalDecoded);
    assert_eq!(call.standard, "aave_v3_pool");
    assert_eq!(call.function, "supply(address,uint256,address,uint16)");
    assert_eq!(decoded_arg(call, "asset"), Some(asset));
    assert_eq!(decoded_arg(call, "amount"), Some("50000000"));
    assert_eq!(decoded_arg(call, "onBehalfOf"), Some(on_behalf_of));
    assert_no_unknown_selector(report);
    assert_local_allowlist_allowed(&review);
}

#[test]
fn live_aave_collateral_enable_can_use_ordinary_approval() {
    let data = format!(
        "0x5a3b74b9{}{}",
        abi_address("0x1111111111111111111111111111111111111111"),
        abi_bool(true),
    );
    let mut review = local_transaction_review(
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
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Allowed);
    assert!(review.policy.can_sign);
    assert!(risk_reason(&review, "aave_collateral_disable_risk").is_none());
}

#[test]
fn live_aave_collateral_disable_with_no_debt_and_dry_run_can_use_ordinary_approval() {
    let data = format!(
        "0x5a3b74b9{}{}",
        abi_address("0x1111111111111111111111111111111111111111"),
        abi_bool(false),
    );
    let mut review = local_transaction_review(
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
    review.simulation.protocol_evidence = Some(json!({
        "aave": {
            "status": "ok",
            "totalDebtBase": "0",
            "healthFactor": "0",
            "transactionDryRun": {
                "status": "ok"
            }
        }
    }));
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Allowed);
    assert!(review.policy.can_sign);
    assert!(risk_reason(&review, "aave_collateral_disable_risk").is_none());
    assert!(risk_reason(&review, "aave_transaction_dry_run_missing").is_none());
}

#[test]
fn live_aave_borrow_without_account_evidence_is_blocked() {
    let from = "0x000000000000000000000000000000000000000a";
    let data = format!(
        "0xa415bcad{}{}{}{}{}",
        abi_address("0x1111111111111111111111111111111111111111"),
        abi_u256(50_000_000),
        abi_u256(2),
        abi_u256(0),
        abi_address(from),
    );
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": "0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(risk_reason(&review, "aave_borrow_health_factor_unknown").is_some());
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
}

#[test]
fn live_aave_borrow_with_safe_account_data_and_dry_run_can_use_ordinary_approval() {
    let from = "0x000000000000000000000000000000000000000a";
    let data = format!(
        "0xa415bcad{}{}{}{}{}",
        abi_address("0x1111111111111111111111111111111111111111"),
        abi_u256(50_000_000),
        abi_u256(2),
        abi_u256(0),
        abi_address(from),
    );
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": "0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );
    review.simulation.protocol_evidence = Some(json!({
        "aave": {
            "status": "ok",
            "totalDebtBase": "1",
            "healthFactor": "2000000000000000000",
            "transactionDryRun": {
                "status": "ok"
            }
        }
    }));
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Allowed);
    assert!(review.policy.can_sign);
    assert!(!review.policy.override_allowed);
    assert!(risk_reason(&review, "aave_borrow_health_factor_unknown").is_none());
    assert!(risk_reason(&review, "aave_transaction_dry_run_missing").is_none());
    assert!(risk_reason(&review, "aave_health_factor_caution").is_none());
    assert_eq!(review.risk.level, TransactionRiskLevel::Low);
}

#[test]
fn live_aave_withdraw_to_third_party_is_blocked() {
    let from = "0x000000000000000000000000000000000000000a";
    let recipient = "0x000000000000000000000000000000000000000b";
    let data = format!(
        "0x69328dec{}{}{}",
        abi_address("0x1111111111111111111111111111111111111111"),
        abi_u256(50_000_000),
        abi_address(recipient),
    );
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": "0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );
    review.simulation.protocol_evidence = Some(json!({
        "aave": {
            "status": "ok",
            "totalDebtBase": "0",
            "healthFactor": "2000000000000000000",
            "transactionDryRun": {
                "status": "ok"
            }
        }
    }));
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(risk_reason(&review, "aave_third_party_withdraw_recipient").is_some());
    assert!(risk_reason(&review, "aave_transaction_dry_run_missing").is_none());
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
}

#[test]
fn live_aave_borrow_with_unsafe_account_data_is_blocked() {
    let from = "0x000000000000000000000000000000000000000a";
    let data = format!(
        "0xa415bcad{}{}{}{}{}",
        abi_address("0x1111111111111111111111111111111111111111"),
        abi_u256(50_000_000),
        abi_u256(2),
        abi_u256(0),
        abi_address(from),
    );
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": "0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );
    review.simulation.protocol_evidence = Some(json!({
        "aave": {
            "status": "ok",
            "healthFactor": "1100000000000000000"
        }
    }));
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.can_sign);
    assert!(risk_reason(&review, "aave_health_factor_liquidation_risk").is_some());
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
}

#[test]
fn live_aave_borrow_with_malformed_health_factor_is_blocked() {
    let from = "0x000000000000000000000000000000000000000a";
    let data = format!(
        "0xa415bcad{}{}{}{}{}",
        abi_address("0x1111111111111111111111111111111111111111"),
        abi_u256(50_000_000),
        abi_u256(2),
        abi_u256(0),
        abi_address(from),
    );
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": "0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    );
    review.simulation.protocol_evidence = Some(json!({
        "aave": {
            "status": "ok",
            "totalDebtBase": "1",
            "healthFactor": "not-a-number",
            "transactionDryRun": {
                "status": "ok"
            }
        }
    }));
    mark_live_simulated(&mut review);

    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.can_sign);
    assert!(risk_reason(&review, "aave_borrow_health_factor_unknown").is_some());
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
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
            .any(|blocker| blocker.code == "unknown_calldata" && !blocker.overrideable)
    );
    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.override_allowed);
    assert_eq!(review.risk.level, TransactionRiskLevel::Blocked);
    assert_eq!(review.risk.action, TransactionRiskAction::Blocked);
    assert!(risk_reason(&review, "unknown_calldata").is_some());
}

#[test]
fn malformed_abi_address_padding_fails_closed() {
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "to": "0x000000000000000000000000000000000000000b",
                "data": concat!(
                    "0x095ea7b3",
                    "010000000000000000000000000000000000000000000000000000000000000c",
                    "0000000000000000000000000000000000000000000000000000000000000001"
                )
            }
        ]),
        "0x1",
    );

    assert_eq!(review.simulation.status, SimulationStatus::InvalidRequest);
    assert!(review.simulation.decoded_call.is_none());
    assert!(review.simulation.approvals.is_empty());
    assert!(
        review
            .simulation
            .warnings
            .iter()
            .any(|warning| warning.code == "erc20_approve_calldata_malformed")
    );
    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.override_allowed);
}

#[test]
fn malformed_abi_bool_fails_closed() {
    let review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "from": "0x000000000000000000000000000000000000000a",
                "to": "0x000000000000000000000000000000000000000b",
                "data": concat!(
                    "0xa22cb465",
                    "000000000000000000000000000000000000000000000000000000000000000c",
                    "0000000000000000000000000000000000000000000000000000000000000002"
                )
            }
        ]),
        "0x1",
    );

    assert_eq!(review.simulation.status, SimulationStatus::InvalidRequest);
    assert!(review.simulation.decoded_call.is_none());
    assert!(review.simulation.approvals.is_empty());
    assert!(
        review
            .simulation
            .warnings
            .iter()
            .any(|warning| warning.code == "operator_approval_calldata_malformed")
    );
    assert_eq!(review.policy.decision, TransactionPolicyDecision::Blocked);
    assert!(!review.policy.override_allowed);
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
                "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                "value": "0x0",
                "data": concat!(
                    "0xa9059cbb",
                    "000000000000000000000000000000000000000000000000000000000000000b",
                    "00000000000000000000000000000000000000000000000000000000000f4240"
                )
            }
        ]),
        "0x1",
    );
    review.simulation.mode = SimulationMode::AlchemyRpc;
    review.simulation.status = SimulationStatus::ProviderSimulated;
    review.simulation.provider_evidence = Some(json!({
        "provider": "fixture",
        "httpStatus": 200,
        "jsonRpcError": false,
        "jsonRpcErrorCode": null,
        "resultError": false,
        "changeCount": 0
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
    assert!(review.impact.provider_asset_changes);
    assert_eq!(review.impact.title, "Impact: 1 transfer");
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
            report.provider_evidence = Some(json!({"fixture": true}));
            report
        }
    }

    let review = simulate_transaction_review(
        &FixtureClient,
        TransactionSimulationRequest {
            method: "eth_sendTransaction",
            params: &json!([
                {
                    "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                    "data": concat!(
                        "0xa9059cbb",
                        "000000000000000000000000000000000000000000000000000000000000000b",
                        "00000000000000000000000000000000000000000000000000000000000f4240"
                    )
                }
            ]),
            default_chain_id: "0x1",
        },
    );

    assert_eq!(
        review.simulation.provider_evidence,
        Some(json!({"fixture": true}))
    );
    assert_eq!(review.policy.decision, TransactionPolicyDecision::Allowed);
    assert!(!review.policy.override_allowed);
    assert!(review.policy.blockers.is_empty());
    assert_eq!(review.risk.level, TransactionRiskLevel::Low);
    assert_eq!(review.risk.action, TransactionRiskAction::OrdinaryApproval);
    assert!(risk_reason(&review, "local_allowlist_match").is_some());
}

#[test]
fn provider_evidence_serializes_without_raw_response_name_and_accepts_legacy_alias() {
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([{"to": "0x000000000000000000000000000000000000000b", "data": "0x"}]),
        "0x1",
    );
    review.simulation.provider_evidence = Some(json!({
        "provider": "fixture",
        "changeCount": 0,
    }));

    let encoded = serde_json::to_value(&review.simulation).unwrap();
    assert!(encoded.get("providerEvidence").is_some());
    assert!(encoded.get("rawProviderResponse").is_none());

    let mut legacy = encoded.as_object().unwrap().clone();
    let evidence = legacy.remove("providerEvidence").unwrap();
    legacy.insert("rawProviderResponse".to_owned(), evidence);
    let decoded: TransactionSimulationReport =
        serde_json::from_value(Value::Object(legacy)).unwrap();

    assert_eq!(
        decoded.provider_evidence,
        Some(json!({
            "provider": "fixture",
            "changeCount": 0,
        }))
    );
}

#[test]
fn alchemy_rpc_debug_redacts_endpoint_url() {
    let config = AlchemyRpcSimulationConfig {
        endpoint_url: "https://eth-mainnet.g.alchemy.com/v2/secret-alchemy-token".to_owned(),
        timeout_ms: 1_000,
        default_gas: "0x7a1200".to_owned(),
    };
    let client = AlchemyRpcSimulationClient::new(config.clone());

    let config_debug = format!("{config:?}");
    let client_debug = format!("{client:?}");

    assert!(config_debug.contains("<redacted>"));
    assert!(!config_debug.contains("secret-alchemy-token"));
    assert!(!client_debug.contains("secret-alchemy-token"));
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
            "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
            "value": "0x0",
            "data": concat!(
                "0xa9059cbb",
                "000000000000000000000000000000000000000000000000000000000000000b",
                "00000000000000000000000000000000000000000000000000000000000f4240"
            ),
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
    assert_eq!(
        review.simulation.provider_evidence,
        Some(json!({
            "provider": "alchemy_simulateAssetChanges",
            "httpStatus": 200,
            "jsonRpcError": false,
            "jsonRpcErrorCode": null,
            "resultError": false,
            "changeCount": 1,
        }))
    );
    let provider_evidence = serde_json::to_string(&review.simulation.provider_evidence).unwrap();
    assert!(!provider_evidence.contains("USDC"));
    assert!(!provider_evidence.contains("USD Coin"));
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
    assert_eq!(
        review.simulation.provider_evidence,
        Some(json!({
            "provider": "alchemy_simulateAssetChanges",
            "httpStatus": 200,
            "jsonRpcError": true,
            "jsonRpcErrorCode": -32000,
            "resultError": false,
            "changeCount": null,
        }))
    );
    let provider_evidence = serde_json::to_string(&review.simulation.provider_evidence).unwrap();
    assert!(!provider_evidence.contains("simulation failed"));
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

fn future_swap_deadline() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as u128)
        .unwrap_or(0)
        .saturating_add(60 * 60)
}

fn far_permit_deadline() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as u128)
        .unwrap_or(0)
        .saturating_add((90 * 24 * 60 * 60) + 1)
}

fn abi_address(value: &str) -> String {
    let address = value.strip_prefix("0x").unwrap_or(value);
    format!("{address:0>64}")
}

fn abi_word_hex(value: &str) -> String {
    let value = value.strip_prefix("0x").unwrap_or(value);
    format!("{value:0>64}")
}

fn max_u160_word() -> String {
    abi_word_hex("ffffffffffffffffffffffffffffffffffffffff")
}

fn abi_bytes_word(hex_bytes: &str) -> String {
    format!("{hex_bytes:0<64}")
}

fn abi_bool(value: bool) -> String {
    abi_u256(u128::from(value))
}

fn abi_dynamic_bytes(hex_bytes: &str) -> String {
    format!(
        "{}{}",
        abi_u256((hex_bytes.len() / 2) as u128),
        abi_padded_hex(hex_bytes)
    )
}

fn abi_padded_hex(hex_bytes: &str) -> String {
    let mut padded = hex_bytes.to_owned();
    let remainder = padded.len() % 64;
    if remainder != 0 {
        padded.push_str(&"0".repeat(64 - remainder));
    }
    padded
}

fn abi_bytes_array(items: &[&str]) -> String {
    let tails = items
        .iter()
        .map(|item| abi_dynamic_bytes(item))
        .collect::<Vec<_>>();
    let mut offset = 32_usize.saturating_mul(items.len());
    let mut offsets = String::new();
    for tail in &tails {
        offsets.push_str(&abi_u256(offset as u128));
        offset = offset.saturating_add(tail.len() / 2);
    }
    format!(
        "{}{}{}",
        abi_u256(items.len() as u128),
        offsets,
        tails.join("")
    )
}

fn universal_router_execute_args(
    commands_hex: &str,
    inputs: &[&str],
    deadline: Option<u128>,
) -> String {
    let commands = abi_dynamic_bytes(commands_hex);
    let encoded_inputs = abi_bytes_array(inputs);
    let head_words = if deadline.is_some() { 3 } else { 2 };
    let commands_offset = 32_usize.saturating_mul(head_words);
    let inputs_offset = commands_offset.saturating_add(commands.len() / 2);
    let mut encoded = format!(
        "{}{}",
        abi_u256(commands_offset as u128),
        abi_u256(inputs_offset as u128)
    );
    if let Some(deadline) = deadline {
        encoded.push_str(&abi_u256(deadline));
    }
    encoded.push_str(&commands);
    encoded.push_str(&encoded_inputs);
    encoded
}

fn universal_router_v3_exact_in_input(
    recipient: &str,
    amount_in: u128,
    amount_out_minimum: u128,
    path_hex: &str,
    payer_is_user: bool,
) -> String {
    format!(
        "{}{}{}{}{}{}",
        abi_address(recipient),
        abi_u256(amount_in),
        abi_u256(amount_out_minimum),
        abi_u256(160),
        abi_bool(payer_is_user),
        abi_dynamic_bytes(path_hex),
    )
}

fn universal_router_permit2_permit_input(
    token: &str,
    amount: u128,
    expiration: u128,
    nonce: u128,
    spender: &str,
    sig_deadline: u128,
    signature_hex: &str,
) -> String {
    universal_router_permit2_permit_input_with_amount_word(
        token,
        &abi_u256(amount),
        expiration,
        nonce,
        spender,
        sig_deadline,
        signature_hex,
    )
}

fn universal_router_permit2_permit_input_with_amount_word(
    token: &str,
    amount_word: &str,
    expiration: u128,
    nonce: u128,
    spender: &str,
    sig_deadline: u128,
    signature_hex: &str,
) -> String {
    format!(
        "{}{}{}{}{}{}{}{}",
        abi_address(token),
        amount_word,
        abi_u256(expiration),
        abi_u256(nonce),
        abi_address(spender),
        abi_u256(sig_deadline),
        abi_u256(224),
        abi_dynamic_bytes(signature_hex),
    )
}

fn universal_router_permit2_permit_review(
    amount_word: &str,
    expiration: u128,
    sig_deadline: u128,
) -> TransactionReviewReport {
    let from = "0x000000000000000000000000000000000000000a";
    let token = "0x1111111111111111111111111111111111111111";
    let spender = "0x66a9893cc07d91d95644aedd05d03f95e1dba8af";
    let permit_input = universal_router_permit2_permit_input_with_amount_word(
        token,
        amount_word,
        expiration,
        7,
        spender,
        sig_deadline,
        &"11".repeat(65),
    );
    let data = format!(
        "0x3593564c{}",
        universal_router_execute_args("0a", &[permit_input.as_str()], Some(future_swap_deadline())),
    );
    local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
                "value": "0x0",
                "data": data
            }
        ]),
        "0x1",
    )
}

fn uniswap_v3_path_hex(token_in: &str, fee: u32, token_out: &str) -> String {
    format!(
        "{}{fee:06x}{}",
        token_in.strip_prefix("0x").unwrap_or(token_in),
        token_out.strip_prefix("0x").unwrap_or(token_out),
    )
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
    review.simulation.provider_evidence = Some(json!({
        "provider": "fixture",
        "httpStatus": 200,
        "jsonRpcError": false,
        "jsonRpcErrorCode": null,
        "resultError": false,
        "changeCount": 0
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

fn assert_local_allowlist_allowed(review: &TransactionReviewReport) {
    assert_eq!(review.policy.decision, TransactionPolicyDecision::Allowed);
    assert!(review.policy.can_sign);
    assert!(!review.policy.override_allowed);
    assert!(review.policy.blockers.is_empty());
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
