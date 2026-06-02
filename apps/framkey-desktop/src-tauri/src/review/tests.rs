use super::*;
use framkey_simulation::{
    SimulationMode, SimulationStatus, TransactionPolicyDecision, TransactionReviewReport,
    evaluate_transaction_impact, evaluate_transaction_policy, evaluate_transaction_risk,
    evaluate_transaction_trust, local_transaction_review,
};
use serde_json::json;

#[test]
fn summarizes_transaction_without_raw_calldata() {
    let params = json!([
        {
            "from": "0x0000000000000000000000000000000000000001",
            "to": "0x0000000000000000000000000000000000000002",
            "value": "0x10",
            "data": "0x12345678"
        }
    ]);

    let summary = summarize_transaction("eth_sendTransaction", &params, "0x1", None, None);
    assert_eq!(summary["intent"], "eth_sendTransaction");
    assert_eq!(summary["dataBytes"], 4);
    assert_eq!(summary["hasData"], true);
    assert_eq!(summary["simulation"]["status"], "local_warnings");
    assert_eq!(
        summary["simulation"]["transaction"]["selector"],
        "0x12345678"
    );
    assert_eq!(summary["policy"]["decision"], "requires_user_override");
    assert_eq!(summary["policy"]["canSign"], false);
    assert_eq!(summary["policy"]["overrideAllowed"], true);
    assert_eq!(summary["risk"]["level"], "high");
    assert_eq!(summary["risk"]["action"], "high_risk_approval");
    assert_eq!(summary["guidance"]["status"], "high_risk");
    assert_eq!(summary["guidance"]["primaryAction"], "Approve High Risk");
    assert_eq!(summary["guidance"]["requiresHighRisk"], true);
    assert_eq!(summary["guidance"]["canApprove"], true);
    assert_eq!(summary["impact"]["title"], "Impact: native value");
    assert_eq!(summary["impact"]["nativeValue"], true);
    assert_eq!(summary["impact"]["approvalCount"], 0);
    assert_eq!(summary["impact"]["transferCount"], 0);
    assert_eq!(summary["trust"]["level"], "unrecognized");
    assert_eq!(summary["trust"]["unknownCount"], 1);
}

#[test]
fn transaction_summary_includes_display_only_asset_context() {
    let params = json!([
        {
            "from": "0x0000000000000000000000000000000000000001",
            "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
            "value": "0x0",
            "data": concat!(
                "0x095ea7b3",
                "0000000000000000000000000000000000000000000000000000000000000002",
                "00000000000000000000000000000000000000000000000000000000000f4240"
            )
        }
    ]);
    let asset_context = json!({
        "status": "ok",
        "tokens": [
            {
                "contractAddress": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                "metadata": {
                    "symbol": "USDC",
                    "decimals": 6
                }
            }
        ]
    });

    let summary = summarize_transaction(
        "eth_sendTransaction",
        &params,
        "0x1",
        None,
        Some(asset_context),
    );
    assert_eq!(summary["assetContext"]["status"], "ok");
    assert_eq!(
        summary["assetContext"]["tokens"][0]["metadata"]["symbol"],
        "USDC"
    );
    assert_eq!(
        summary["policy"]["decision"], "requires_user_override",
        "metadata is display-only and must not alter policy"
    );
    assert_eq!(summary["risk"]["level"], "high");
    assert_eq!(summary["impact"]["approvalCount"], 1);
    assert_eq!(summary["impact"]["items"][0]["title"], "Token approval");
    assert_eq!(summary["trust"]["level"], "unrecognized");
    assert_eq!(summary["trust"]["unknownCount"], 2);
}

#[test]
fn transaction_guidance_marks_live_simulated_request_ready() {
    let summary = summarize_transaction(
        "eth_sendTransaction",
        &json!([
            {
                "from": "0x0000000000000000000000000000000000000001",
                "data": "0x"
            }
        ]),
        "0x1",
        Some(allowed_transaction_review()),
        None,
    );

    assert_eq!(summary["policy"]["decision"], "allowed");
    assert_eq!(summary["guidance"]["status"], "ready");
    assert_eq!(summary["guidance"]["tone"], "good");
    assert_eq!(summary["guidance"]["primaryAction"], "Approve Transaction");
    assert_eq!(summary["guidance"]["canApprove"], true);
    assert_eq!(summary["guidance"]["blocked"], false);
}

#[test]
fn transaction_guidance_explains_live_simulation_failure_block() {
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "from": "0x0000000000000000000000000000000000000001",
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

    let summary = summarize_transaction(
        "eth_sendTransaction",
        &json!([
            {
                "from": "0x0000000000000000000000000000000000000001",
                "data": "0x"
            }
        ]),
        "0x1",
        Some(review),
        None,
    );

    assert_eq!(summary["policy"]["decision"], "blocked");
    assert_eq!(summary["guidance"]["status"], "blocked");
    assert_eq!(summary["guidance"]["tone"], "bad");
    assert_eq!(summary["guidance"]["primaryAction"], "Cannot Sign");
    assert_eq!(
        summary["guidance"]["reasonCode"],
        "simulation_provider_failed"
    );
    assert_eq!(summary["guidance"]["canApprove"], false);
    assert_eq!(summary["guidance"]["blocked"], true);
    assert!(
        summary["guidance"]["nextStep"]
            .as_str()
            .unwrap()
            .contains("Check RPC health")
    );
}

#[test]
fn summarizes_erc20_permit_typed_data() {
    let deadline = future_deadline();
    let summary = summarize_typed_data(
        "eth_signTypedData_v4",
        &json!([
            "0x1111111111111111111111111111111111111111",
            {
                "domain": {
                    "name": "USD Coin",
                    "version": "2",
                    "chainId": 1,
                    "verifyingContract": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                },
                "primaryType": "Permit",
                "types": {
                    "EIP712Domain": [],
                    "Permit": [
                        {"name": "owner", "type": "address"},
                        {"name": "spender", "type": "address"},
                        {"name": "value", "type": "uint256"},
                        {"name": "nonce", "type": "uint256"},
                        {"name": "deadline", "type": "uint256"}
                    ]
                },
                "message": {
                    "owner": "0x1111111111111111111111111111111111111111",
                    "spender": "0x000000000022d473030f116ddee9f6b43ac78ba3",
                    "value": "1000000",
                    "nonce": "7",
                    "deadline": deadline
                }
            }
        ]),
        "0x1",
    );

    assert_eq!(
        summary["account"],
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(summary["typedData"]["intent"], "erc20_permit");
    assert_eq!(summary["typedData"]["primaryType"], "Permit");
    assert_eq!(
        summary["typedData"]["permit"]["token"],
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
    );
    assert_eq!(summary["typedData"]["permit"]["amount"], "1000000");
    assert_eq!(summary["typedData"]["policy"]["canSign"], true);
    assert_eq!(summary["decision"], "blocked_before_approval");
}

#[test]
fn summarizes_permit2_typed_data() {
    let deadline = future_deadline();
    let summary = summarize_typed_data(
        "eth_signTypedData_v4",
        &json!([
            "0x1111111111111111111111111111111111111111",
            {
                "domain": {
                    "name": "Permit2",
                    "chainId": 1,
                    "verifyingContract": "0x000000000022d473030f116ddee9f6b43ac78ba3"
                },
                "primaryType": "PermitSingle",
                "types": {
                    "EIP712Domain": [],
                    "PermitDetails": [
                        {"name": "token", "type": "address"},
                        {"name": "amount", "type": "uint160"},
                        {"name": "expiration", "type": "uint48"},
                        {"name": "nonce", "type": "uint48"}
                    ],
                    "PermitSingle": [
                        {"name": "details", "type": "PermitDetails"},
                        {"name": "spender", "type": "address"},
                        {"name": "sigDeadline", "type": "uint256"}
                    ]
                },
                "message": {
                    "details": {
                        "token": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                        "amount": "1000000",
                        "expiration": deadline,
                        "nonce": "9"
                    },
                    "spender": "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
                    "sigDeadline": deadline
                }
            }
        ]),
        "0x1",
    );

    assert_eq!(summary["typedData"]["intent"], "permit2_permit_single");
    assert_eq!(
        summary["typedData"]["permit"]["kind"],
        "permit2_permit_single"
    );
    assert_eq!(
        summary["typedData"]["permit"]["token"],
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
    );
    assert_eq!(
        summary["typedData"]["permit"]["spender"],
        "0x66a9893cc07d91d95644aedd05d03f95e1dba8af"
    );
    assert_eq!(summary["typedData"]["policy"]["canSign"], true);
    assert_eq!(summary["decision"], "blocked_before_approval");
}

#[test]
fn typed_data_schema_mismatch_blocks_signing() {
    let deadline = future_deadline();
    let summary = summarize_typed_data(
        "eth_signTypedData_v4",
        &json!([
            "0x1111111111111111111111111111111111111111",
            {
                "domain": {
                    "name": "USD Coin",
                    "version": "2",
                    "chainId": 1,
                    "verifyingContract": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                },
                "primaryType": "Permit",
                "types": {
                    "EIP712Domain": [],
                    "Permit": [
                        {"name": "owner", "type": "address"},
                        {"name": "spender", "type": "address"},
                        {"name": "value", "type": "uint160"},
                        {"name": "nonce", "type": "uint256"},
                        {"name": "deadline", "type": "uint256"}
                    ]
                },
                "message": {
                    "owner": "0x1111111111111111111111111111111111111111",
                    "spender": "0x000000000022d473030f116ddee9f6b43ac78ba3",
                    "value": "1000000",
                    "nonce": "7",
                    "deadline": deadline
                }
            }
        ]),
        "0x1",
    );

    assert_eq!(summary["typedData"]["policy"]["canSign"], false);
    assert_eq!(
        summary["typedData"]["policy"]["blockers"][0]["code"],
        "typed_data_schema_mismatch"
    );
}

#[test]
fn review_queue_caps_length() {
    let mut queue = ReviewQueue::new();
    for index in 0..(MAX_REVIEW_QUEUE_ITEMS + 2) {
        queue
            .capture(
                format!("request-{index}"),
                "personal_sign".to_owned(),
                Some("https://example.test".to_owned()),
                &json!(["0x4869", "0x0000000000000000000000000000000000000001"]),
                "0x1",
                None,
            )
            .unwrap();
    }

    let items = queue.snapshot();
    assert_eq!(items.len(), MAX_REVIEW_QUEUE_ITEMS);
    assert_eq!(items[0].provider_request_id, "request-33");
}

#[test]
fn approval_consumes_decision_token() {
    let mut queue = ReviewQueue::new();
    let request = queue
        .capture(
            "request-1".to_owned(),
            "personal_sign".to_owned(),
            Some("https://example.test".to_owned()),
            &json!(["0x4869", "0x0000000000000000000000000000000000000001"]),
            "0x1",
            None,
        )
        .unwrap();

    let outcome = queue
        .decide(
            &request.id,
            &request.decision_token,
            ReviewDecision::Approve,
        )
        .unwrap();
    assert_eq!(outcome.review_request.status, ReviewStatus::Approved);
    assert!(outcome.review_request.decision_token_consumed);
    assert!(outcome.signing_enabled);
    assert_eq!(outcome.broker_mode, "controlled_personal_sign");

    let replay = queue.decide(
        &request.id,
        &request.decision_token,
        ReviewDecision::Approve,
    );
    assert!(replay.unwrap_err().to_string().contains("already consumed"));
}

#[test]
fn recognized_typed_data_approval_enters_controlled_signing_mode() {
    let mut queue = ReviewQueue::new();
    let deadline = future_deadline();
    let request = queue
        .capture(
            "request-1".to_owned(),
            "eth_signTypedData_v4".to_owned(),
            Some("https://example.test".to_owned()),
            &json!([
                "0x1111111111111111111111111111111111111111",
                {
                    "domain": {
                        "name": "USD Coin",
                        "version": "2",
                        "chainId": 1,
                        "verifyingContract": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                    },
                    "primaryType": "Permit",
                    "types": {
                        "EIP712Domain": [],
                        "Permit": [
                            {"name": "owner", "type": "address"},
                            {"name": "spender", "type": "address"},
                            {"name": "value", "type": "uint256"},
                            {"name": "nonce", "type": "uint256"},
                            {"name": "deadline", "type": "uint256"}
                        ]
                    },
                    "message": {
                        "owner": "0x1111111111111111111111111111111111111111",
                        "spender": "0x000000000022d473030f116ddee9f6b43ac78ba3",
                        "value": "1000000",
                        "nonce": "7",
                        "deadline": deadline
                    }
                }
            ]),
            "0x1",
            None,
        )
        .unwrap();

    let outcome = queue
        .decide(
            &request.id,
            &request.decision_token,
            ReviewDecision::Approve,
        )
        .unwrap();
    assert!(outcome.signing_enabled);
    assert_eq!(outcome.broker_mode, "controlled_typed_data_signing");
    assert_eq!(
        typed_data_signing_authorization(&outcome.review_request).unwrap(),
        "controlled_typed_data_signing"
    );
}

#[test]
fn typed_data_owner_mismatch_approval_remains_blocked() {
    let mut queue = ReviewQueue::new();
    let deadline = future_deadline();
    let request = queue
        .capture(
            "request-1".to_owned(),
            "eth_signTypedData_v4".to_owned(),
            Some("https://example.test".to_owned()),
            &json!([
                "0x1111111111111111111111111111111111111111",
                {
                    "domain": {
                        "name": "USD Coin",
                        "version": "2",
                        "chainId": 1,
                        "verifyingContract": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                    },
                    "primaryType": "Permit",
                    "types": {
                        "EIP712Domain": [],
                        "Permit": [
                            {"name": "owner", "type": "address"},
                            {"name": "spender", "type": "address"},
                            {"name": "value", "type": "uint256"},
                            {"name": "nonce", "type": "uint256"},
                            {"name": "deadline", "type": "uint256"}
                        ]
                    },
                    "message": {
                        "owner": "0x9999999999999999999999999999999999999999",
                        "spender": "0x000000000022d473030f116ddee9f6b43ac78ba3",
                        "value": "1000000",
                        "nonce": "7",
                        "deadline": deadline
                    }
                }
            ]),
            "0x1",
            None,
        )
        .unwrap();

    assert_eq!(request.summary["typedData"]["policy"]["canSign"], false);
    assert_eq!(
        request.summary["typedData"]["policy"]["blockers"][0]["code"],
        "permit_owner_mismatch"
    );
    let error = queue
        .decide(
            &request.id,
            &request.decision_token,
            ReviewDecision::Approve,
        )
        .unwrap_err();
    assert!(error.to_string().contains("permit_owner_mismatch"));
}

#[test]
fn typed_data_unknown_spender_approval_remains_blocked() {
    let mut queue = ReviewQueue::new();
    let deadline = future_deadline();
    let request = queue
        .capture(
            "request-1".to_owned(),
            "eth_signTypedData_v4".to_owned(),
            Some("https://example.test".to_owned()),
            &json!([
                "0x1111111111111111111111111111111111111111",
                {
                    "domain": {
                        "name": "Permit2",
                        "chainId": 1,
                        "verifyingContract": "0x000000000022d473030f116ddee9f6b43ac78ba3"
                    },
                    "primaryType": "PermitSingle",
                    "types": {
                        "EIP712Domain": [],
                        "PermitDetails": [
                            {"name": "token", "type": "address"},
                            {"name": "amount", "type": "uint160"},
                            {"name": "expiration", "type": "uint48"},
                            {"name": "nonce", "type": "uint48"}
                        ],
                        "PermitSingle": [
                            {"name": "details", "type": "PermitDetails"},
                            {"name": "spender", "type": "address"},
                            {"name": "sigDeadline", "type": "uint256"}
                        ]
                    },
                    "message": {
                        "details": {
                            "token": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                            "amount": "1000000",
                            "expiration": deadline,
                            "nonce": "9"
                        },
                        "spender": "0x3333333333333333333333333333333333333333",
                        "sigDeadline": deadline
                    }
                }
            ]),
            "0x1",
            None,
        )
        .unwrap();

    assert_eq!(request.summary["typedData"]["policy"]["canSign"], false);
    assert_eq!(
        request.summary["typedData"]["policy"]["blockers"][0]["code"],
        "unknown_permit_spender"
    );
    let error = queue
        .decide(
            &request.id,
            &request.decision_token,
            ReviewDecision::Approve,
        )
        .unwrap_err();
    assert!(error.to_string().contains("unknown_permit_spender"));
}

#[test]
fn unknown_typed_data_approval_remains_blocked() {
    let mut queue = ReviewQueue::new();
    let request = queue
        .capture(
            "request-1".to_owned(),
            "eth_signTypedData_v4".to_owned(),
            Some("https://example.test".to_owned()),
            &json!([
                "0x1111111111111111111111111111111111111111",
                {
                    "domain": {},
                    "primaryType": "Message",
                    "types": {
                        "EIP712Domain": [],
                        "Message": [{"name": "text", "type": "string"}]
                    },
                    "message": {"text": "hello"}
                }
            ]),
            "0x1",
            None,
        )
        .unwrap();

    let error = queue
        .decide(
            &request.id,
            &request.decision_token,
            ReviewDecision::Approve,
        )
        .unwrap_err();
    assert!(error.to_string().contains("unrecognized_typed_data_intent"));
    let pending = queue.get(&request.id).unwrap();
    assert_eq!(pending.status, ReviewStatus::Pending);
    assert!(!pending.decision_token_consumed);
}

#[test]
fn transaction_approval_enters_controlled_signing_mode() {
    let mut queue = ReviewQueue::new();
    let request = queue
        .capture(
            "request-1".to_owned(),
            "eth_sendTransaction".to_owned(),
            Some("https://example.test".to_owned()),
            &json!([{"from": "0x0000000000000000000000000000000000000001", "data": "0x"}]),
            "0x1",
            Some(allowed_transaction_review()),
        )
        .unwrap();

    let outcome = queue
        .decide(
            &request.id,
            &request.decision_token,
            ReviewDecision::Approve,
        )
        .unwrap();
    assert!(outcome.signing_enabled);
    assert_eq!(outcome.broker_mode, "controlled_transaction_signing");
    assert_eq!(
        transaction_signing_authorization(&outcome.review_request).unwrap(),
        "controlled_transaction_signing"
    );
}

#[test]
fn ordinary_transaction_approval_cannot_authorize_override_required_policy() {
    let mut queue = ReviewQueue::new();
    let request = queue
        .capture(
            "request-1".to_owned(),
            "eth_sendTransaction".to_owned(),
            Some("https://example.test".to_owned()),
            &json!([{"from": "0x0000000000000000000000000000000000000001", "data": "0x"}]),
            "0x1",
            None,
        )
        .unwrap();

    let error = queue
        .decide(
            &request.id,
            &request.decision_token,
            ReviewDecision::Approve,
        )
        .unwrap_err();

    assert!(error.to_string().contains("high-risk approval"));
    let pending = queue.get(&request.id).unwrap();
    assert_eq!(pending.status, ReviewStatus::Pending);
    assert!(!pending.decision_token_consumed);
}

#[test]
fn high_risk_transaction_approval_enters_override_mode() {
    let mut queue = ReviewQueue::new();
    let request = queue
        .capture(
            "request-1".to_owned(),
            "eth_sendTransaction".to_owned(),
            Some("https://example.test".to_owned()),
            &json!([{"from": "0x0000000000000000000000000000000000000001", "data": "0x"}]),
            "0x1",
            None,
        )
        .unwrap();

    let outcome = queue
        .decide(
            &request.id,
            &request.decision_token,
            ReviewDecision::ApproveWithRisk,
        )
        .unwrap();

    assert!(outcome.signing_enabled);
    assert_eq!(
        outcome.broker_mode,
        "controlled_transaction_high_risk_override"
    );
    assert_eq!(
        outcome
            .review_request
            .decision
            .as_ref()
            .map(|record| record.decision),
        Some(ReviewDecision::ApproveWithRisk)
    );
    assert_eq!(
        transaction_signing_authorization(&outcome.review_request).unwrap(),
        "controlled_transaction_high_risk_override"
    );
}

#[test]
fn blocked_transaction_policy_rejects_high_risk_override() {
    let mut queue = ReviewQueue::new();
    let request = queue
        .capture(
            "request-1".to_owned(),
            "eth_sendTransaction".to_owned(),
            Some("https://example.test".to_owned()),
            &json!([]),
            "0x1",
            None,
        )
        .unwrap();

    let error = queue
        .decide(
            &request.id,
            &request.decision_token,
            ReviewDecision::ApproveWithRisk,
        )
        .unwrap_err();

    assert!(error.to_string().contains("does not allow"));
    let pending = queue.get(&request.id).unwrap();
    assert_eq!(pending.status, ReviewStatus::Pending);
    assert!(!pending.decision_token_consumed);
}

#[test]
fn high_risk_approval_is_not_valid_for_personal_sign() {
    let mut queue = ReviewQueue::new();
    let request = queue
        .capture(
            "request-1".to_owned(),
            "personal_sign".to_owned(),
            Some("https://example.test".to_owned()),
            &json!(["0x4869", "0x0000000000000000000000000000000000000001"]),
            "0x1",
            None,
        )
        .unwrap();

    let error = queue
        .decide(
            &request.id,
            &request.decision_token,
            ReviewDecision::ApproveWithRisk,
        )
        .unwrap_err();

    assert!(error.to_string().contains("personal_sign"));
}

#[test]
fn signed_request_records_execution_metadata() {
    let mut queue = ReviewQueue::new();
    let request = queue
        .capture(
            "request-1".to_owned(),
            "personal_sign".to_owned(),
            Some("https://example.test".to_owned()),
            &json!(["0x4869", "0x0000000000000000000000000000000000000001"]),
            "0x1",
            None,
        )
        .unwrap();
    queue
        .decide(
            &request.id,
            &request.decision_token,
            ReviewDecision::Approve,
        )
        .unwrap();

    let signed = queue
        .mark_signed(
            &request.id,
            "0x0000000000000000000000000000000000000001".to_owned(),
            "0x1234".to_owned(),
        )
        .unwrap();

    assert_eq!(signed.status, ReviewStatus::Signed);
    assert_eq!(
        signed
            .execution
            .as_ref()
            .and_then(|record| record.address.as_deref()),
        Some("0x0000000000000000000000000000000000000001")
    );
}

#[test]
fn parses_personal_sign_hex_message_and_account() {
    let payload = personal_sign_payload(&json!([
        "0x4652414d4b6579",
        "0x0000000000000000000000000000000000000001"
    ]))
    .unwrap();

    assert_eq!(payload.message, b"FRAMKey");
    assert_eq!(
        payload.expected_address.as_deref(),
        Some("0x0000000000000000000000000000000000000001")
    );
}

#[test]
fn parses_personal_sign_text_message() {
    let payload = personal_sign_payload(&json!(["FRAMKey", null])).unwrap();
    assert_eq!(payload.message, b"FRAMKey");
    assert_eq!(payload.expected_address, None);
}

#[test]
fn rejects_malformed_personal_sign_hex_message() {
    let error = personal_sign_payload(&json!(["0x123", null])).unwrap_err();
    assert!(error.to_string().contains("malformed"));
}

#[test]
fn expired_request_cannot_be_approved() {
    let mut queue = ReviewQueue::new();
    let mut request = queue
        .capture(
            "request-1".to_owned(),
            "personal_sign".to_owned(),
            Some("https://example.test".to_owned()),
            &json!(["0x4869", "0x0000000000000000000000000000000000000001"]),
            "0x1",
            None,
        )
        .unwrap();
    queue.items[0].expires_at_unix_ms = 0;
    request.expires_at_unix_ms = 0;

    let error = queue
        .decide(
            &request.id,
            &request.decision_token,
            ReviewDecision::Approve,
        )
        .unwrap_err();
    assert!(error.to_string().contains("expired"));
}

#[test]
fn provider_view_omits_decision_token() {
    let mut queue = ReviewQueue::new();
    let request = queue
        .capture(
            "request-1".to_owned(),
            "personal_sign".to_owned(),
            Some("https://example.test".to_owned()),
            &json!(["0x4869", "0x0000000000000000000000000000000000000001"]),
            "0x1",
            None,
        )
        .unwrap();

    let view = request.provider_view();
    assert!(view.get("decisionToken").is_none());
    assert!(view.get("brokerSessionId").is_some());
}

fn allowed_transaction_review() -> TransactionReviewReport {
    let mut review = local_transaction_review(
        "eth_sendTransaction",
        &json!([
            {
                "from": "0x0000000000000000000000000000000000000001",
                "data": "0x"
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
    review
}

fn future_deadline() -> String {
    current_unix_seconds().saturating_add(60 * 60).to_string()
}
