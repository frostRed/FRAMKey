use serde_json::Value;

use crate::model::{
    ApprovalChange, AssetTransfer, DecodedArgument, DecodedCall, NormalizedTransaction,
    PolicyBlocker, SimulationMode, SimulationStatus, SimulationWarning, TokenAmount,
    TransactionSimulationReport, WarningSeverity,
};

pub fn local_transaction_report(
    method: &str,
    params: &Value,
    default_chain_id: &str,
) -> TransactionSimulationReport {
    let tx = params
        .as_array()
        .and_then(|items| items.first())
        .and_then(Value::as_object);

    let mut warnings = Vec::new();
    let chain_id = tx
        .and_then(|tx| tx.get("chainId"))
        .and_then(Value::as_str)
        .unwrap_or(default_chain_id)
        .to_owned();

    if tx.is_none() {
        warnings.push(warning(
            WarningSeverity::Error,
            "invalid_transaction_params",
            "eth_sendTransaction params must contain one transaction object",
        ));
    }

    let from = tx.and_then(|tx| string_field(tx, "from"));
    let to = tx.and_then(|tx| string_field(tx, "to"));
    let value = tx
        .and_then(|tx| string_field(tx, "value"))
        .and_then(|value| parse_quantity(value, &mut warnings, "value"));
    let data = tx.and_then(|tx| string_field(tx, "data").or_else(|| string_field(tx, "input")));
    let gas = tx
        .and_then(|tx| string_field(tx, "gas").or_else(|| string_field(tx, "gasLimit")))
        .map(str::to_owned);
    let gas_price = tx
        .and_then(|tx| string_field(tx, "gasPrice"))
        .map(str::to_owned);
    let max_fee_per_gas = tx
        .and_then(|tx| string_field(tx, "maxFeePerGas"))
        .map(str::to_owned);
    let max_priority_fee_per_gas = tx
        .and_then(|tx| string_field(tx, "maxPriorityFeePerGas"))
        .map(str::to_owned);
    let nonce = tx
        .and_then(|tx| string_field(tx, "nonce"))
        .map(str::to_owned);

    warn_if_malformed_address(&mut warnings, "from", from);
    warn_if_malformed_address(&mut warnings, "to", to);
    if !same_chain_id(&chain_id, default_chain_id) {
        warnings.push(warning(
            WarningSeverity::Warning,
            "chain_id_mismatch",
            "transaction chainId differs from the configured FRAMKey chain",
        ));
    }

    let mut data_bytes = None;
    let mut selector = None;
    let mut decoded_call = None;
    let mut approvals = Vec::new();
    let mut asset_transfers = Vec::new();

    if let Some(data) = data {
        match decode_hex_data(data) {
            Ok(bytes) => {
                data_bytes = Some(bytes.len());
                if bytes.is_empty() {
                    if tx.is_some() && value.as_ref().is_none_or(|value| value.decimal == "0") {
                        warnings.push(warning(
                            WarningSeverity::Info,
                            "empty_transaction",
                            "transaction has no native value and no calldata",
                        ));
                    }
                } else if bytes.len() < 4 {
                    warnings.push(warning(
                        WarningSeverity::Error,
                        "malformed_calldata",
                        "calldata is shorter than a 4-byte function selector",
                    ));
                } else {
                    selector = Some(format!("0x{}", hex_lower(&bytes[..4])));
                    decode_known_call(
                        &bytes,
                        from,
                        to,
                        &mut decoded_call,
                        &mut asset_transfers,
                        &mut approvals,
                        &mut warnings,
                    );
                }
            }
            Err(message) => warnings.push(warning(
                WarningSeverity::Error,
                "malformed_calldata",
                message,
            )),
        }
    }

    let native_value = value.as_ref().filter(|value| value.decimal != "0").cloned();
    if native_value.is_some() {
        warnings.push(warning(
            WarningSeverity::Warning,
            "native_value_transfer",
            "transaction moves native chain value",
        ));
    }

    let data_preview = data.map(|value| preview_string(value, 160));
    let transaction = NormalizedTransaction {
        method: method.to_owned(),
        chain_id: chain_id.clone(),
        from: from.map(str::to_owned),
        to: to.map(str::to_owned),
        value,
        data_bytes,
        selector,
        data_preview,
        gas,
        gas_price,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        nonce,
    };

    let status = if warnings
        .iter()
        .any(|warning| warning.severity == WarningSeverity::Error)
    {
        SimulationStatus::InvalidRequest
    } else if warnings.is_empty() {
        SimulationStatus::LocalDecoded
    } else {
        SimulationStatus::LocalWarnings
    };

    TransactionSimulationReport {
        mode: SimulationMode::LocalDecoderOnly,
        status,
        chain_id,
        transaction,
        native_value,
        decoded_call,
        asset_transfers,
        approvals,
        warnings,
        raw_provider_response: None,
    }
}

fn decode_known_call(
    bytes: &[u8],
    from: Option<&str>,
    contract: Option<&str>,
    decoded_call: &mut Option<DecodedCall>,
    asset_transfers: &mut Vec<AssetTransfer>,
    approvals: &mut Vec<ApprovalChange>,
    warnings: &mut Vec<SimulationWarning>,
) {
    let selector = &bytes[..4];
    match selector {
        [0xa9, 0x05, 0x9c, 0xbb] => {
            let Some((recipient, amount)) = decode_address_amount_args(
                bytes,
                warnings,
                "erc20_transfer_calldata_short",
                "erc20_transfer_calldata_malformed",
                "to",
            ) else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "erc20",
                "transfer(address,uint256)",
                selector,
                contract,
                vec![
                    arg("to", "address", recipient.clone()),
                    arg("amount", "uint256", amount.decimal.clone()),
                ],
            ));
            asset_transfers.push(AssetTransfer {
                asset_kind: "erc20".to_owned(),
                contract: contract.map(str::to_owned),
                from: from.map(str::to_owned),
                to: Some(recipient),
                amount: Some(amount),
                token_id: None,
            });
        }
        [0x09, 0x5e, 0xa7, 0xb3] => {
            let Some((spender, amount)) = decode_address_amount_args(
                bytes,
                warnings,
                "erc20_approve_calldata_short",
                "erc20_approve_calldata_malformed",
                "spender",
            ) else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "erc20",
                "approve(address,uint256)",
                selector,
                contract,
                vec![
                    arg("spender", "address", spender.clone()),
                    arg("amount", "uint256", amount.decimal.clone()),
                ],
            ));
            if is_max_u256(&amount.hex) {
                warnings.push(warning(
                    WarningSeverity::Warning,
                    "unlimited_token_approval",
                    "approval amount is the maximum uint256 value",
                ));
            }
            approvals.push(ApprovalChange {
                asset_kind: "erc20".to_owned(),
                contract: contract.map(str::to_owned),
                owner: from.map(str::to_owned),
                spender: Some(spender),
                operator: None,
                amount: Some(amount),
                approved: None,
            });
        }
        [0x23, 0xb8, 0x72, 0xdd] => {
            let Some((owner, recipient, amount)) = decode_transfer_from_args(bytes, warnings)
            else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "erc20",
                "transferFrom(address,address,uint256)",
                selector,
                contract,
                vec![
                    arg("from", "address", owner.clone()),
                    arg("to", "address", recipient.clone()),
                    arg("amount", "uint256", amount.decimal.clone()),
                ],
            ));
            asset_transfers.push(AssetTransfer {
                asset_kind: "erc20".to_owned(),
                contract: contract.map(str::to_owned),
                from: Some(owner),
                to: Some(recipient),
                amount: Some(amount),
                token_id: None,
            });
        }
        [0xa2, 0x2c, 0xb4, 0x65] => {
            let Some((operator, approved)) = decode_operator_approval_args(bytes, warnings) else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "erc721_or_erc1155",
                "setApprovalForAll(address,bool)",
                selector,
                contract,
                vec![
                    arg("operator", "address", operator.clone()),
                    arg("approved", "bool", approved.to_string()),
                ],
            ));
            if approved {
                warnings.push(warning(
                    WarningSeverity::Warning,
                    "operator_approval_for_all",
                    "approval grants an operator transfer authority for all matching tokens",
                ));
            }
            approvals.push(ApprovalChange {
                asset_kind: "erc721_or_erc1155".to_owned(),
                contract: contract.map(str::to_owned),
                owner: from.map(str::to_owned),
                spender: None,
                operator: Some(operator),
                amount: None,
                approved: Some(approved),
            });
        }
        [0x42, 0x84, 0x2e, 0x0e] | [0xb8, 0x8d, 0x4f, 0xde] => {
            let Some((owner, recipient, token_id)) = decode_nft_transfer_args(bytes, warnings)
            else {
                return;
            };
            let function = if selector == [0x42, 0x84, 0x2e, 0x0e] {
                "safeTransferFrom(address,address,uint256)"
            } else {
                "safeTransferFrom(address,address,uint256,bytes)"
            };
            *decoded_call = Some(decoded_call_value(
                "erc721_or_erc1155",
                function,
                selector,
                contract,
                vec![
                    arg("from", "address", owner.clone()),
                    arg("to", "address", recipient.clone()),
                    arg("tokenId", "uint256", token_id.decimal.clone()),
                ],
            ));
            asset_transfers.push(AssetTransfer {
                asset_kind: "erc721_or_erc1155".to_owned(),
                contract: contract.map(str::to_owned),
                from: Some(owner),
                to: Some(recipient),
                amount: None,
                token_id: Some(token_id),
            });
        }
        [0x38, 0xed, 0x17, 0x39] => {
            let Some(arguments) =
                decode_uniswap_v2_swap_exact_tokens_for_tokens_args(bytes, warnings)
            else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "uniswap_v2_router",
                "swapExactTokensForTokens(uint256,uint256,address[],address,uint256)",
                selector,
                contract,
                arguments,
            ));
        }
        [0x7f, 0xf3, 0x6a, 0xb5] => {
            let Some(arguments) = decode_uniswap_v2_swap_exact_eth_for_tokens_args(bytes, warnings)
            else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "uniswap_v2_router",
                "swapExactETHForTokens(uint256,address[],address,uint256)",
                selector,
                contract,
                arguments,
            ));
        }
        [0x18, 0xcb, 0xaf, 0xe5] => {
            let Some(arguments) = decode_uniswap_v2_swap_exact_tokens_for_eth_args(bytes, warnings)
            else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "uniswap_v2_router",
                "swapExactTokensForETH(uint256,uint256,address[],address,uint256)",
                selector,
                contract,
                arguments,
            ));
        }
        [0x41, 0x4b, 0xf3, 0x89] => {
            let Some(arguments) = decode_uniswap_v3_exact_input_single_args(bytes, warnings) else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "uniswap_v3_swap_router",
                "exactInputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))",
                selector,
                contract,
                arguments,
            ));
        }
        [0xc0, 0x4b, 0x8d, 0x59] => {
            let Some(arguments) = decode_uniswap_v3_exact_input_args(bytes, warnings) else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "uniswap_v3_swap_router",
                "exactInput((bytes,address,uint256,uint256,uint256))",
                selector,
                contract,
                arguments,
            ));
        }
        [0x35, 0x93, 0x56, 0x4c] => {
            let Some(arguments) =
                decode_uniswap_universal_router_execute_args(bytes, warnings, true)
            else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "uniswap_universal_router",
                "execute(bytes,bytes[],uint256)",
                selector,
                contract,
                arguments,
            ));
        }
        [0x24, 0x85, 0x6b, 0xc3] => {
            let Some(arguments) =
                decode_uniswap_universal_router_execute_args(bytes, warnings, false)
            else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "uniswap_universal_router",
                "execute(bytes,bytes[])",
                selector,
                contract,
                arguments,
            ));
        }
        [0xac, 0x96, 0x50, 0xd8] => {
            let Some(arguments) = decode_multicall_args(bytes, warnings, false) else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "multicall",
                "multicall(bytes[])",
                selector,
                contract,
                arguments,
            ));
        }
        [0x5a, 0xe4, 0x01, 0xdc] => {
            let Some(arguments) = decode_multicall_args(bytes, warnings, true) else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "multicall",
                "multicall(uint256,bytes[])",
                selector,
                contract,
                arguments,
            ));
        }
        [0x61, 0x7b, 0xa0, 0x37] => {
            let Some(arguments) = decode_aave_supply_args(bytes, warnings) else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "aave_v3_pool",
                "supply(address,uint256,address,uint16)",
                selector,
                contract,
                arguments,
            ));
        }
        [0x69, 0x32, 0x8d, 0xec] => {
            let Some(arguments) = decode_aave_withdraw_args(bytes, warnings) else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "aave_v3_pool",
                "withdraw(address,uint256,address)",
                selector,
                contract,
                arguments,
            ));
        }
        [0xa4, 0x15, 0xbc, 0xad] => {
            let Some(arguments) = decode_aave_borrow_args(bytes, warnings) else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "aave_v3_pool",
                "borrow(address,uint256,uint256,uint16,address)",
                selector,
                contract,
                arguments,
            ));
        }
        [0x57, 0x3a, 0xde, 0x81] => {
            let Some(arguments) = decode_aave_repay_args(bytes, warnings) else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "aave_v3_pool",
                "repay(address,uint256,uint256,address)",
                selector,
                contract,
                arguments,
            ));
        }
        [0x5a, 0x3b, 0x74, 0xb9] => {
            let Some(arguments) = decode_aave_collateral_args(bytes, warnings) else {
                return;
            };
            *decoded_call = Some(decoded_call_value(
                "aave_v3_pool",
                "setUserUseReserveAsCollateral(address,bool)",
                selector,
                contract,
                arguments,
            ));
        }
        _ => {
            let selector_hex = format!("0x{}", hex_lower(selector));
            *decoded_call = Some(DecodedCall {
                standard: "unknown".to_owned(),
                function: "unknown".to_owned(),
                selector: Some(selector_hex),
                contract: contract.map(str::to_owned),
                arguments: Vec::new(),
            });
            warnings.push(warning(
                WarningSeverity::Warning,
                "unknown_function_selector",
                "calldata selector is not covered by the local decoder",
            ));
        }
    }
}

fn decode_address_amount_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
    short_code: &str,
    malformed_code: &str,
    address_label: &str,
) -> Option<(String, TokenAmount)> {
    if bytes.len() < 4 + (32 * 2) {
        warnings.push(warning(
            WarningSeverity::Error,
            short_code,
            "calldata is too short for address,uint256 arguments",
        ));
        return None;
    }
    Some((
        decode_address_word(&bytes[4..36], warnings, malformed_code, address_label)?,
        decode_u256_word(&bytes[36..68]),
    ))
}

fn decode_transfer_from_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<(String, String, TokenAmount)> {
    if bytes.len() < 4 + (32 * 3) {
        warnings.push(warning(
            WarningSeverity::Error,
            "erc20_transfer_from_calldata_short",
            "calldata is too short for address,address,uint256 arguments",
        ));
        return None;
    }
    Some((
        decode_address_word(
            &bytes[4..36],
            warnings,
            "erc20_transfer_from_calldata_malformed",
            "from",
        )?,
        decode_address_word(
            &bytes[36..68],
            warnings,
            "erc20_transfer_from_calldata_malformed",
            "to",
        )?,
        decode_u256_word(&bytes[68..100]),
    ))
}

fn decode_operator_approval_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<(String, bool)> {
    if bytes.len() < 4 + (32 * 2) {
        warnings.push(warning(
            WarningSeverity::Error,
            "operator_approval_calldata_short",
            "calldata is too short for address,bool arguments",
        ));
        return None;
    }
    Some((
        decode_address_word(
            &bytes[4..36],
            warnings,
            "operator_approval_calldata_malformed",
            "operator",
        )?,
        decode_bool_word(
            &bytes[36..68],
            warnings,
            "operator_approval_calldata_malformed",
            "approved",
        )?,
    ))
}

fn decode_nft_transfer_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<(String, String, TokenAmount)> {
    if bytes.len() < 4 + (32 * 3) {
        warnings.push(warning(
            WarningSeverity::Error,
            "nft_transfer_calldata_short",
            "calldata is too short for address,address,uint256 arguments",
        ));
        return None;
    }
    Some((
        decode_address_word(
            &bytes[4..36],
            warnings,
            "nft_transfer_calldata_malformed",
            "from",
        )?,
        decode_address_word(
            &bytes[36..68],
            warnings,
            "nft_transfer_calldata_malformed",
            "to",
        )?,
        decode_u256_word(&bytes[68..100]),
    ))
}

fn decode_uniswap_v2_swap_exact_tokens_for_tokens_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<Vec<DecodedArgument>> {
    require_abi_words(
        bytes,
        5,
        warnings,
        "uniswap_v2_swap_calldata_short",
        "calldata is too short for swapExactTokensForTokens arguments",
    )?;
    let amount_in = decode_u256_word(argument_word(bytes, 0)?);
    let amount_out_min = decode_u256_word(argument_word(bytes, 1)?);
    let to = decode_address_word(
        argument_word(bytes, 3)?,
        warnings,
        "uniswap_v2_swap_calldata_malformed",
        "to",
    )?;
    let deadline = decode_u256_word(argument_word(bytes, 4)?);
    let mut arguments = vec![
        arg("amountIn", "uint256", amount_in.decimal),
        arg("amountOutMin", "uint256", amount_out_min.decimal),
        arg("to", "address", to),
        arg("deadline", "uint256", deadline.decimal),
    ];
    arguments.extend(decode_address_array_summary(
        bytes,
        argument_word(bytes, 2)?,
        warnings,
        "uniswap_v2_path_malformed",
        "path",
    )?);
    Some(arguments)
}

fn decode_uniswap_v2_swap_exact_eth_for_tokens_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<Vec<DecodedArgument>> {
    require_abi_words(
        bytes,
        4,
        warnings,
        "uniswap_v2_swap_calldata_short",
        "calldata is too short for swapExactETHForTokens arguments",
    )?;
    let amount_out_min = decode_u256_word(argument_word(bytes, 0)?);
    let to = decode_address_word(
        argument_word(bytes, 2)?,
        warnings,
        "uniswap_v2_swap_calldata_malformed",
        "to",
    )?;
    let deadline = decode_u256_word(argument_word(bytes, 3)?);
    let mut arguments = vec![
        arg("amountOutMin", "uint256", amount_out_min.decimal),
        arg("to", "address", to),
        arg("deadline", "uint256", deadline.decimal),
    ];
    arguments.extend(decode_address_array_summary(
        bytes,
        argument_word(bytes, 1)?,
        warnings,
        "uniswap_v2_path_malformed",
        "path",
    )?);
    Some(arguments)
}

fn decode_uniswap_v2_swap_exact_tokens_for_eth_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<Vec<DecodedArgument>> {
    require_abi_words(
        bytes,
        5,
        warnings,
        "uniswap_v2_swap_calldata_short",
        "calldata is too short for swapExactTokensForETH arguments",
    )?;
    let amount_in = decode_u256_word(argument_word(bytes, 0)?);
    let amount_out_min = decode_u256_word(argument_word(bytes, 1)?);
    let to = decode_address_word(
        argument_word(bytes, 3)?,
        warnings,
        "uniswap_v2_swap_calldata_malformed",
        "to",
    )?;
    let deadline = decode_u256_word(argument_word(bytes, 4)?);
    let mut arguments = vec![
        arg("amountIn", "uint256", amount_in.decimal),
        arg("amountOutMin", "uint256", amount_out_min.decimal),
        arg("to", "address", to),
        arg("deadline", "uint256", deadline.decimal),
    ];
    arguments.extend(decode_address_array_summary(
        bytes,
        argument_word(bytes, 2)?,
        warnings,
        "uniswap_v2_path_malformed",
        "path",
    )?);
    Some(arguments)
}

fn decode_uniswap_v3_exact_input_single_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<Vec<DecodedArgument>> {
    require_abi_words(
        bytes,
        8,
        warnings,
        "uniswap_v3_exact_input_single_calldata_short",
        "calldata is too short for exactInputSingle arguments",
    )?;
    let token_in = decode_address_word(
        argument_word(bytes, 0)?,
        warnings,
        "uniswap_v3_exact_input_single_calldata_malformed",
        "tokenIn",
    )?;
    let token_out = decode_address_word(
        argument_word(bytes, 1)?,
        warnings,
        "uniswap_v3_exact_input_single_calldata_malformed",
        "tokenOut",
    )?;
    let fee = decode_u256_word(argument_word(bytes, 2)?);
    let recipient = decode_address_word(
        argument_word(bytes, 3)?,
        warnings,
        "uniswap_v3_exact_input_single_calldata_malformed",
        "recipient",
    )?;
    let deadline = decode_u256_word(argument_word(bytes, 4)?);
    let amount_in = decode_u256_word(argument_word(bytes, 5)?);
    let amount_out_minimum = decode_u256_word(argument_word(bytes, 6)?);
    let sqrt_price_limit_x96 = decode_u256_word(argument_word(bytes, 7)?);
    Some(vec![
        arg("tokenIn", "address", token_in),
        arg("tokenOut", "address", token_out),
        arg("fee", "uint24", fee.decimal),
        arg("recipient", "address", recipient),
        arg("deadline", "uint256", deadline.decimal),
        arg("amountIn", "uint256", amount_in.decimal),
        arg("amountOutMinimum", "uint256", amount_out_minimum.decimal),
        arg("sqrtPriceLimitX96", "uint160", sqrt_price_limit_x96.decimal),
    ])
}

fn decode_uniswap_v3_exact_input_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<Vec<DecodedArgument>> {
    require_abi_words(
        bytes,
        5,
        warnings,
        "uniswap_v3_exact_input_calldata_short",
        "calldata is too short for exactInput arguments",
    )?;
    let path = decode_dynamic_bytes_slice(
        bytes,
        argument_word(bytes, 0)?,
        warnings,
        "uniswap_v3_path_malformed",
        "path",
    )?;
    let recipient = decode_address_word(
        argument_word(bytes, 1)?,
        warnings,
        "uniswap_v3_exact_input_calldata_malformed",
        "recipient",
    )?;
    let deadline = decode_u256_word(argument_word(bytes, 2)?);
    let amount_in = decode_u256_word(argument_word(bytes, 3)?);
    let amount_out_minimum = decode_u256_word(argument_word(bytes, 4)?);
    let mut arguments = uniswap_v3_path_arguments(path);
    arguments.extend([
        arg("recipient", "address", recipient),
        arg("deadline", "uint256", deadline.decimal),
        arg("amountIn", "uint256", amount_in.decimal),
        arg("amountOutMinimum", "uint256", amount_out_minimum.decimal),
    ]);
    Some(arguments)
}

fn decode_uniswap_universal_router_execute_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
    has_deadline: bool,
) -> Option<Vec<DecodedArgument>> {
    let required_words = if has_deadline { 3 } else { 2 };
    require_abi_words(
        bytes,
        required_words,
        warnings,
        "uniswap_universal_router_execute_calldata_short",
        "calldata is too short for Universal Router execute arguments",
    )?;
    let commands = decode_dynamic_bytes_slice(
        bytes,
        argument_word(bytes, 0)?,
        warnings,
        "uniswap_universal_router_commands_malformed",
        "commands",
    )?;
    let input_count = decode_dynamic_head_len(
        bytes,
        argument_word(bytes, 1)?,
        warnings,
        "uniswap_universal_router_inputs_malformed",
        "inputs",
    )?;
    let mut arguments = vec![
        arg("commandBytes", "bytes", commands.len().to_string()),
        arg("inputCount", "bytes[]", input_count.to_string()),
    ];
    if has_deadline {
        let deadline = decode_u256_word(argument_word(bytes, 2)?);
        arguments.push(arg("deadline", "uint256", deadline.decimal));
    }
    Some(arguments)
}

fn decode_multicall_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
    has_deadline: bool,
) -> Option<Vec<DecodedArgument>> {
    let required_words = if has_deadline { 2 } else { 1 };
    require_abi_words(
        bytes,
        required_words,
        warnings,
        "multicall_calldata_short",
        "calldata is too short for multicall arguments",
    )?;
    let mut arguments = Vec::new();
    let data_offset_word = if has_deadline {
        let deadline = decode_u256_word(argument_word(bytes, 0)?);
        arguments.push(arg("deadline", "uint256", deadline.decimal));
        argument_word(bytes, 1)?
    } else {
        argument_word(bytes, 0)?
    };
    let call_count = decode_dynamic_head_len(
        bytes,
        data_offset_word,
        warnings,
        "multicall_payload_malformed",
        "multicall data",
    )?;
    arguments.push(arg("callCount", "bytes[]", call_count.to_string()));
    Some(arguments)
}

fn decode_aave_supply_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<Vec<DecodedArgument>> {
    require_abi_words(
        bytes,
        4,
        warnings,
        "aave_supply_calldata_short",
        "calldata is too short for Aave supply arguments",
    )?;
    let amount = decode_u256_word(argument_word(bytes, 1)?);
    let referral_code = decode_u256_word(argument_word(bytes, 3)?);
    Some(vec![
        arg(
            "asset",
            "address",
            decode_address_word(
                argument_word(bytes, 0)?,
                warnings,
                "aave_supply_calldata_malformed",
                "asset",
            )?,
        ),
        arg("amount", "uint256", amount.decimal),
        arg(
            "onBehalfOf",
            "address",
            decode_address_word(
                argument_word(bytes, 2)?,
                warnings,
                "aave_supply_calldata_malformed",
                "onBehalfOf",
            )?,
        ),
        arg("referralCode", "uint16", referral_code.decimal),
    ])
}

fn decode_aave_withdraw_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<Vec<DecodedArgument>> {
    require_abi_words(
        bytes,
        3,
        warnings,
        "aave_withdraw_calldata_short",
        "calldata is too short for Aave withdraw arguments",
    )?;
    let amount = decode_u256_word(argument_word(bytes, 1)?);
    Some(vec![
        arg(
            "asset",
            "address",
            decode_address_word(
                argument_word(bytes, 0)?,
                warnings,
                "aave_withdraw_calldata_malformed",
                "asset",
            )?,
        ),
        arg("amount", "uint256", amount.decimal),
        arg(
            "to",
            "address",
            decode_address_word(
                argument_word(bytes, 2)?,
                warnings,
                "aave_withdraw_calldata_malformed",
                "to",
            )?,
        ),
    ])
}

fn decode_aave_borrow_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<Vec<DecodedArgument>> {
    require_abi_words(
        bytes,
        5,
        warnings,
        "aave_borrow_calldata_short",
        "calldata is too short for Aave borrow arguments",
    )?;
    let amount = decode_u256_word(argument_word(bytes, 1)?);
    let rate_mode = decode_u256_word(argument_word(bytes, 2)?);
    let referral_code = decode_u256_word(argument_word(bytes, 3)?);
    Some(vec![
        arg(
            "asset",
            "address",
            decode_address_word(
                argument_word(bytes, 0)?,
                warnings,
                "aave_borrow_calldata_malformed",
                "asset",
            )?,
        ),
        arg("amount", "uint256", amount.decimal),
        arg("interestRateMode", "uint256", rate_mode.decimal),
        arg("referralCode", "uint16", referral_code.decimal),
        arg(
            "onBehalfOf",
            "address",
            decode_address_word(
                argument_word(bytes, 4)?,
                warnings,
                "aave_borrow_calldata_malformed",
                "onBehalfOf",
            )?,
        ),
    ])
}

fn decode_aave_repay_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<Vec<DecodedArgument>> {
    require_abi_words(
        bytes,
        4,
        warnings,
        "aave_repay_calldata_short",
        "calldata is too short for Aave repay arguments",
    )?;
    let amount = decode_u256_word(argument_word(bytes, 1)?);
    let rate_mode = decode_u256_word(argument_word(bytes, 2)?);
    Some(vec![
        arg(
            "asset",
            "address",
            decode_address_word(
                argument_word(bytes, 0)?,
                warnings,
                "aave_repay_calldata_malformed",
                "asset",
            )?,
        ),
        arg("amount", "uint256", amount.decimal),
        arg("interestRateMode", "uint256", rate_mode.decimal),
        arg(
            "onBehalfOf",
            "address",
            decode_address_word(
                argument_word(bytes, 3)?,
                warnings,
                "aave_repay_calldata_malformed",
                "onBehalfOf",
            )?,
        ),
    ])
}

fn decode_aave_collateral_args(
    bytes: &[u8],
    warnings: &mut Vec<SimulationWarning>,
) -> Option<Vec<DecodedArgument>> {
    require_abi_words(
        bytes,
        2,
        warnings,
        "aave_collateral_calldata_short",
        "calldata is too short for Aave collateral arguments",
    )?;
    Some(vec![
        arg(
            "asset",
            "address",
            decode_address_word(
                argument_word(bytes, 0)?,
                warnings,
                "aave_collateral_calldata_malformed",
                "asset",
            )?,
        ),
        arg(
            "useAsCollateral",
            "bool",
            decode_bool_word(
                argument_word(bytes, 1)?,
                warnings,
                "aave_collateral_calldata_malformed",
                "useAsCollateral",
            )?
            .to_string(),
        ),
    ])
}

fn decoded_call_value(
    standard: &str,
    function: &str,
    selector: &[u8],
    contract: Option<&str>,
    arguments: Vec<DecodedArgument>,
) -> DecodedCall {
    DecodedCall {
        standard: standard.to_owned(),
        function: function.to_owned(),
        selector: Some(format!("0x{}", hex_lower(selector))),
        contract: contract.map(str::to_owned),
        arguments,
    }
}

fn arg(name: &str, kind: &str, value: String) -> DecodedArgument {
    DecodedArgument {
        name: name.to_owned(),
        kind: kind.to_owned(),
        value,
    }
}

fn require_abi_words(
    bytes: &[u8],
    word_count: usize,
    warnings: &mut Vec<SimulationWarning>,
    code: &str,
    message: &str,
) -> Option<()> {
    let Some(required) = word_count
        .checked_mul(32)
        .and_then(|bytes| 4_usize.checked_add(bytes))
    else {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            "calldata word count overflows supported decoder bounds",
        ));
        return None;
    };
    if bytes.len() < required {
        warnings.push(warning(WarningSeverity::Error, code, message));
        return None;
    }
    Some(())
}

fn argument_word(bytes: &[u8], index: usize) -> Option<&[u8]> {
    let start = 4_usize.checked_add(index.checked_mul(32)?)?;
    bytes.get(start..start + 32)
}

fn decode_address_array_summary(
    bytes: &[u8],
    offset_word: &[u8],
    warnings: &mut Vec<SimulationWarning>,
    code: &str,
    label: &str,
) -> Option<Vec<DecodedArgument>> {
    let count = decode_dynamic_head_len(bytes, offset_word, warnings, code, label)?;
    let offset = decode_usize_word(offset_word)?;
    let Some(values_start) = 4_usize
        .checked_add(offset)
        .and_then(|value| value.checked_add(32))
    else {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("dynamic {label} offset overflows calldata length"),
        ));
        return None;
    };
    let Some(values_len) = count.checked_mul(32) else {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("dynamic {label} address array length is too large"),
        ));
        return None;
    };
    let Some(values_end) = values_start.checked_add(values_len) else {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("dynamic {label} address array overflows calldata length"),
        ));
        return None;
    };
    if bytes.len() < values_end {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("calldata is too short for dynamic {label} address array"),
        ));
        return None;
    }

    let mut arguments = vec![arg("pathLength", "address[]", count.to_string())];
    if count > 0 {
        arguments.push(arg(
            "pathFirst",
            "address",
            decode_address_word(
                &bytes[values_start..values_start + 32],
                warnings,
                code,
                "path first address",
            )?,
        ));
    }
    if count > 1 {
        let last_start = values_start + ((count - 1) * 32);
        arguments.push(arg(
            "pathLast",
            "address",
            decode_address_word(
                &bytes[last_start..last_start + 32],
                warnings,
                code,
                "path last address",
            )?,
        ));
    }
    Some(arguments)
}

fn decode_dynamic_head_len(
    bytes: &[u8],
    offset_word: &[u8],
    warnings: &mut Vec<SimulationWarning>,
    code: &str,
    label: &str,
) -> Option<usize> {
    let Some(offset) = decode_usize_word(offset_word) else {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("dynamic {label} offset is too large"),
        ));
        return None;
    };
    let Some(len_start) = 4_usize.checked_add(offset) else {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("dynamic {label} offset overflows calldata length"),
        ));
        return None;
    };
    let Some(len_word) = bytes.get(len_start..len_start + 32) else {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("calldata is too short for dynamic {label} length"),
        ));
        return None;
    };
    let Some(len) = decode_usize_word(len_word) else {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("dynamic {label} length is too large"),
        ));
        return None;
    };
    Some(len)
}

fn decode_dynamic_bytes_slice<'a>(
    bytes: &'a [u8],
    offset_word: &[u8],
    warnings: &mut Vec<SimulationWarning>,
    code: &str,
    label: &str,
) -> Option<&'a [u8]> {
    let len = decode_dynamic_head_len(bytes, offset_word, warnings, code, label)?;
    let offset = decode_usize_word(offset_word)?;
    let Some(data_start) = 4_usize
        .checked_add(offset)
        .and_then(|value| value.checked_add(32))
    else {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("dynamic {label} offset overflows calldata length"),
        ));
        return None;
    };
    let Some(data_end) = data_start.checked_add(len) else {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("dynamic {label} bytes length overflows calldata length"),
        ));
        return None;
    };
    let Some(data) = bytes.get(data_start..data_end) else {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("calldata is too short for dynamic {label} bytes"),
        ));
        return None;
    };
    Some(data)
}

fn uniswap_v3_path_arguments(path: &[u8]) -> Vec<DecodedArgument> {
    let mut arguments = vec![arg("pathBytes", "bytes", path.len().to_string())];
    if path.len() >= 43 && (path.len() - 20).is_multiple_of(23) {
        arguments.push(arg(
            "pathHops",
            "uint256",
            ((path.len() - 20) / 23).to_string(),
        ));
        arguments.push(arg(
            "tokenIn",
            "address",
            format!("0x{}", hex_lower(&path[..20])),
        ));
        arguments.push(arg(
            "tokenOut",
            "address",
            format!("0x{}", hex_lower(&path[path.len() - 20..])),
        ));
    }
    arguments
}

fn decode_usize_word(word: &[u8]) -> Option<usize> {
    let mut value = 0_usize;
    for byte in word {
        value = value.checked_mul(256)?.checked_add(*byte as usize)?;
    }
    Some(value)
}

fn decode_address_word(
    word: &[u8],
    warnings: &mut Vec<SimulationWarning>,
    code: &str,
    label: &str,
) -> Option<String> {
    if word.len() != 32 {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("ABI address field {label} is not 32 bytes"),
        ));
        return None;
    }
    if word[..12].iter().any(|byte| *byte != 0) {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("ABI address field {label} has non-zero left padding"),
        ));
        return None;
    }
    Some(format!("0x{}", hex_lower(&word[12..32])))
}

fn decode_bool_word(
    word: &[u8],
    warnings: &mut Vec<SimulationWarning>,
    code: &str,
    label: &str,
) -> Option<bool> {
    if word.len() != 32 {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("ABI bool field {label} is not 32 bytes"),
        ));
        return None;
    }
    if word[..31].iter().any(|byte| *byte != 0) {
        warnings.push(warning(
            WarningSeverity::Error,
            code,
            format!("ABI bool field {label} has non-zero left padding"),
        ));
        return None;
    }
    match word[31] {
        0 => Some(false),
        1 => Some(true),
        _ => {
            warnings.push(warning(
                WarningSeverity::Error,
                code,
                format!("ABI bool field {label} is not encoded as 0 or 1"),
            ));
            None
        }
    }
}

fn decode_u256_word(word: &[u8]) -> TokenAmount {
    TokenAmount {
        hex: format!("0x{}", hex_lower(word)),
        decimal: hex_bytes_to_decimal(word),
    }
}

fn parse_quantity(
    value: &str,
    warnings: &mut Vec<SimulationWarning>,
    field: &str,
) -> Option<TokenAmount> {
    match decode_hex_quantity(value) {
        Ok(bytes) => Some(TokenAmount {
            hex: normalize_hex_quantity(&bytes),
            decimal: hex_bytes_to_decimal(&bytes),
        }),
        Err(message) => {
            warnings.push(warning(
                WarningSeverity::Error,
                format!("malformed_{field}"),
                message,
            ));
            None
        }
    }
}

pub(crate) fn string_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Option<&'a str> {
    object.get(key).and_then(Value::as_str)
}

fn warn_if_malformed_address(
    warnings: &mut Vec<SimulationWarning>,
    field: &str,
    value: Option<&str>,
) {
    if let Some(value) = value
        && !looks_like_eth_address(value)
    {
        warnings.push(warning(
            WarningSeverity::Error,
            format!("malformed_{field}_address"),
            format!("transaction {field} is not a 0x-prefixed 20-byte address"),
        ));
    }
}

pub(crate) fn looks_like_eth_address(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value.as_bytes().iter().skip(2).all(u8::is_ascii_hexdigit)
}

pub(crate) fn same_chain_id(left: &str, right: &str) -> bool {
    let left = left.trim_start_matches("0x");
    let right = right.trim_start_matches("0x");
    let left = left.trim_start_matches('0');
    let right = right.trim_start_matches('0');
    left.eq_ignore_ascii_case(right) || (left.is_empty() && right.is_empty())
}

fn decode_hex_data(value: &str) -> Result<Vec<u8>, &'static str> {
    let Some(hex) = value.strip_prefix("0x") else {
        return Err("calldata must be 0x-prefixed hex");
    };
    if hex.len() % 2 != 0 {
        return Err("calldata hex must have an even number of digits");
    }
    if !hex.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return Err("calldata contains non-hex characters");
    }
    Ok(hex_to_bytes(hex))
}

pub(crate) fn decode_hex_quantity(value: &str) -> Result<Vec<u8>, &'static str> {
    let Some(hex) = value.strip_prefix("0x") else {
        return Err("quantity must be 0x-prefixed hex");
    };
    if hex.is_empty() {
        return Err("quantity hex must not be empty");
    }
    if !hex.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return Err("quantity contains non-hex characters");
    }
    let padded = if hex.len() % 2 == 0 {
        hex.to_owned()
    } else {
        format!("0{hex}")
    };
    Ok(hex_to_bytes(&padded))
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|pair| (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]))
        .collect()
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => 0,
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

pub(crate) fn normalize_hex_quantity(bytes: &[u8]) -> String {
    let trimmed = trim_leading_zero_bytes(bytes);
    if trimmed.is_empty() {
        "0x0".to_owned()
    } else {
        let hex = hex_lower(trimmed);
        format!("0x{}", hex.trim_start_matches('0'))
    }
}

pub(crate) fn decimal_digits_to_hex_quantity(value: &str) -> Option<String> {
    let mut digits = value
        .as_bytes()
        .iter()
        .copied()
        .skip_while(|byte| *byte == b'0')
        .map(|byte| byte - b'0')
        .collect::<Vec<_>>();
    if digits.is_empty() {
        return Some("0x0".to_owned());
    }

    let mut hex = Vec::new();
    while !digits.is_empty() {
        let mut next = Vec::new();
        let mut remainder = 0_u8;
        for digit in digits {
            let value = u16::from(remainder) * 10 + u16::from(digit);
            let quotient = (value / 16) as u8;
            remainder = (value % 16) as u8;
            if !next.is_empty() || quotient != 0 {
                next.push(quotient);
            }
        }
        hex.push(std::char::from_digit(u32::from(remainder), 16)?);
        digits = next;
    }
    hex.reverse();
    Some(format!("0x{}", hex.into_iter().collect::<String>()))
}

pub(crate) fn normalize_decimal_digits(value: &str) -> String {
    let trimmed = value.trim_start_matches('0');
    if trimmed.is_empty() {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

pub(crate) fn hex_bytes_to_decimal(bytes: &[u8]) -> String {
    let mut digits = vec![0_u8];
    for byte in trim_leading_zero_bytes(bytes) {
        let mut carry = u16::from(*byte);
        for digit in digits.iter_mut().rev() {
            let value = u16::from(*digit) * 256 + carry;
            *digit = (value % 10) as u8;
            carry = value / 10;
        }
        while carry > 0 {
            digits.insert(0, (carry % 10) as u8);
            carry /= 10;
        }
    }
    digits
        .into_iter()
        .map(|digit| char::from(b'0' + digit))
        .collect()
}

fn trim_leading_zero_bytes(bytes: &[u8]) -> &[u8] {
    let index = bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(bytes.len());
    &bytes[index..]
}

pub(crate) fn is_max_u256(hex: &str) -> bool {
    hex == "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
}

pub(crate) fn preview_string(value: &str, max_chars: usize) -> String {
    let char_count = value.chars().count();
    if char_count <= max_chars {
        return value.to_owned();
    }
    let prefix = value.chars().take(max_chars).collect::<String>();
    format!("{prefix}... ({char_count} chars)")
}

pub(crate) fn warning(
    severity: WarningSeverity,
    code: impl Into<String>,
    message: impl Into<String>,
) -> SimulationWarning {
    SimulationWarning {
        severity,
        code: code.into(),
        message: message.into(),
    }
}

pub(crate) fn policy_blocker(code: impl Into<String>, message: impl Into<String>) -> PolicyBlocker {
    PolicyBlocker {
        code: code.into(),
        message: message.into(),
        overrideable: false,
    }
}

pub(crate) fn overrideable_policy_blocker(
    code: impl Into<String>,
    message: impl Into<String>,
) -> PolicyBlocker {
    PolicyBlocker {
        code: code.into(),
        message: message.into(),
        overrideable: true,
    }
}
