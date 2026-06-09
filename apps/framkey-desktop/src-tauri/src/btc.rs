use std::time::Duration;

use anyhow::{Context, Result};
use framkey_btc::{
    BtcBalance, BtcNetwork, BtcSpendRequest, BtcUtxo, balance_from_utxos, prepare_p2wpkh_spend,
    utxos_from_esplora_value, validate_signed_transaction_for_plan,
};
use reqwest::blocking::Client;
use serde_json::{Value, json};

use crate::*;

pub(crate) fn btc_balance_snapshot(
    state: &AppState,
    config: &DesktopConfig,
    request: BtcBalanceRequest,
) -> Result<Value> {
    let normalized = request.normalized()?;
    let account = connected_btc_account(state)?;
    let address = btc_address_for_account(&account, normalized.network)?;
    let balance = fetch_btc_balance(config, normalized.network, &address)?;
    Ok(btc_balance_value(config, balance))
}

pub(crate) fn send_btc_transfer_from_trusted_ui(
    state: &AppState,
    config: &DesktopConfig,
    request: BtcTransferRequest,
) -> Result<Value> {
    let normalized = request.normalized()?;
    let account = connected_btc_account(state)?;
    let from_address = btc_address_for_account(&account, normalized.network)?;
    let utxos = fetch_btc_utxos(config, normalized.network, &from_address)?;
    let prepared = prepare_p2wpkh_spend(
        &BtcSpendRequest {
            network: normalized.network,
            from_address: from_address.clone(),
            to_address: normalized.to_address,
            amount_sat: normalized.amount_sat,
            fee_rate_sat_vb: normalized.fee_rate_sat_vb,
        },
        &utxos,
    )?;
    let review_request = ProviderRequest {
        id: format!("trusted-btc-send-{}-{}", std::process::id(), now_unix_ms()),
        method: "framkey_btcSendTransaction".to_owned(),
        params: json!(prepared.plan),
        origin: Some(TRUSTED_UI_ORIGIN.to_owned()),
    };
    let review = state.capture_review_request(config, &review_request)?;
    eprintln!("btc_send captured review_id={}", review.id);
    let approved = state.wait_for_review_approval(&review.id)?;
    if approved.kind != review::ReviewMethodKind::BtcTransaction {
        anyhow::bail!("approved review request {} is not BTC send", approved.id);
    }
    let broker_mode = match review::btc_transaction_signing_authorization(&approved) {
        Ok(mode) => mode,
        Err(error) => {
            let message = error.to_string();
            let _ = state.mark_review_sign_failed(&review.id, &message);
            return Err(error);
        }
    };

    let signed = match config.wallet {
        DesktopWalletConfig::MockInMemory => state.sign_btc_psbt_with_mock_wallet(
            normalized.network,
            prepared.psbt_bytes.clone(),
            from_address.clone(),
        ),
        DesktopWalletConfig::KeychainVault => {
            let save_image = read_configured_save_image(config)?;
            sign_btc_psbt_with_helper(
                config,
                save_image,
                normalized.network.id(),
                prepared.psbt_bytes.clone(),
                from_address.clone(),
            )
        }
    };
    let signed = match signed {
        Ok(signed) => signed,
        Err(error) => {
            let message = error.to_string();
            let _ = state.mark_review_sign_failed(&review.id, &message);
            eprintln!(
                "btc_send signing failed review_id={}: {}",
                review.id, message
            );
            return Err(error);
        }
    };
    if let Err(error) =
        validate_signed_transaction_for_plan(&prepared.plan, &signed.raw_transaction)
    {
        let message = error.to_string();
        let _ = state.mark_review_sign_failed(&review.id, &message);
        eprintln!(
            "btc_send signed transaction validation failed review_id={}: {}",
            review.id, message
        );
        return Err(error.into());
    }
    let broadcast_txid =
        match broadcast_btc_transaction(config, normalized.network, &signed.raw_transaction) {
            Ok(txid) => txid,
            Err(error) => {
                let message = error.to_string();
                let _ = state.mark_review_sign_failed(&review.id, &message);
                eprintln!(
                    "btc_send broadcast failed review_id={}: {}",
                    review.id, message
                );
                return Err(error);
            }
        };
    if broadcast_txid != signed.transaction_id {
        let message = "BTC broadcast returned a different txid than the locally signed transaction";
        let _ = state.mark_review_sign_failed(&review.id, message);
        anyhow::bail!("{message}");
    }
    state.mark_review_btc_broadcast(&review.id, &signed.address, &signed.transaction_id)?;
    eprintln!(
        "btc_send broadcast review_id={} broker_mode={} txid={}",
        review.id, broker_mode, signed.transaction_id
    );
    Ok(json!({
        "operation": "send_btc_transfer",
        "status": "broadcast",
        "network": normalized.network.id(),
        "fromAddress": signed.address,
        "toAddress": prepared.plan.to_address,
        "amountSat": prepared.plan.amount_sat,
        "feeSat": prepared.plan.fee_sat,
        "feeRateSatVb": prepared.plan.fee_rate_sat_vb,
        "changeSat": prepared.plan.change_sat,
        "inputCount": prepared.plan.selected_utxos.len(),
        "transactionId": signed.transaction_id,
        "vbytes": signed.vbytes,
        "reviewOrigin": TRUSTED_UI_ORIGIN,
        "backend": config.btc.describe_network(normalized.network),
    }))
}

pub(crate) fn fetch_btc_balance(
    config: &DesktopConfig,
    network: BtcNetwork,
    address: &str,
) -> Result<BtcBalance> {
    let utxos = fetch_btc_utxos(config, network, address)?;
    Ok(balance_from_utxos(network, address, utxos))
}

pub(crate) fn fetch_btc_utxos(
    config: &DesktopConfig,
    network: BtcNetwork,
    address: &str,
) -> Result<Vec<BtcUtxo>> {
    let client = EsploraClient::from_config(config, network)?;
    let value = client
        .get_json(&format!("/address/{address}/utxo"))
        .with_context(|| format!("failed to read BTC UTXOs for {}", network.id()))?;
    utxos_from_esplora_value(&value).map_err(Into::into)
}

pub(crate) fn broadcast_btc_transaction(
    config: &DesktopConfig,
    network: BtcNetwork,
    raw_transaction: &str,
) -> Result<String> {
    let client = EsploraClient::from_config(config, network)?;
    client.post_tx(raw_transaction)
}

fn btc_balance_value(config: &DesktopConfig, balance: BtcBalance) -> Value {
    json!({
        "operation": "btc_balance",
        "status": "ok",
        "network": balance.network.id(),
        "address": balance.address,
        "confirmedSat": balance.confirmed_sat,
        "unconfirmedSat": balance.unconfirmed_sat,
        "spendableSat": balance.spendable_sat,
        "utxoCount": balance.utxo_count,
        "spendableUtxoCount": balance.spendable_utxo_count,
        "utxos": balance.utxos,
        "backend": config.btc.describe_network(balance.network),
    })
}

fn connected_btc_account(state: &AppState) -> Result<DesktopAccount> {
    state.connected_account()?.ok_or_else(|| {
        anyhow::anyhow!("wallet account is not connected; connect the vault before using BTC")
    })
}

fn btc_address_for_account(account: &DesktopAccount, network: BtcNetwork) -> Result<String> {
    let accounts = account
        .accounts
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("wallet account list is malformed"))?;
    accounts
        .iter()
        .find(|account| {
            account.get("family").and_then(Value::as_str) == Some("btc")
                && account.get("network").and_then(Value::as_str) == Some(network.id())
        })
        .and_then(|account| account.get("address"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("wallet has no BTC account for {}", network.id()))
}

struct EsploraClient {
    base_url: String,
    client: Client,
}

impl EsploraClient {
    fn from_config(config: &DesktopConfig, network: BtcNetwork) -> Result<Self> {
        let endpoint = config.btc.endpoint_for_network(network).ok_or_else(|| {
            anyhow::anyhow!("BTC Esplora backend is not configured for {}", network.id())
        })?;
        let client = Client::builder()
            .timeout(Duration::from_millis(config.btc.timeout_ms))
            .build()
            .context("failed to create BTC Esplora HTTP client")?;
        Ok(Self {
            base_url: endpoint.trim_end_matches('/').to_owned(),
            client,
        })
    }

    fn get_json(&self, path: &str) -> Result<Value> {
        let response = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .send()
            .context("BTC Esplora request failed")?;
        let status = response.status();
        let text = response
            .text()
            .context("failed to read BTC Esplora response body")?;
        if !status.is_success() {
            anyhow::bail!(
                "BTC Esplora request failed with HTTP {}: {}",
                status.as_u16(),
                truncate_for_event(&text, 240)
            );
        }
        serde_json::from_str(&text).context("BTC Esplora returned malformed JSON")
    }

    fn post_tx(&self, raw_transaction: &str) -> Result<String> {
        let response = self
            .client
            .post(format!("{}/tx", self.base_url))
            .body(raw_transaction.to_owned())
            .send()
            .context("BTC transaction broadcast request failed")?;
        let status = response.status();
        let text = response
            .text()
            .context("failed to read BTC transaction broadcast response")?;
        if !status.is_success() {
            anyhow::bail!(
                "BTC transaction broadcast failed with HTTP {}",
                status.as_u16()
            );
        }
        let txid = text.trim();
        if txid.len() != 64 || !txid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            anyhow::bail!("BTC transaction broadcast returned malformed txid");
        }
        Ok(txid.to_owned())
    }
}
