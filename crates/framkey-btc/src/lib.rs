use std::{fmt, str::FromStr};

use bitcoin::{
    Address, Amount, CompressedPublicKey, EcdsaSighashType, Network, OutPoint, PrivateKey, Psbt,
    ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness, absolute,
    consensus::encode::{deserialize, serialize},
    ecdsa, psbt,
    secp256k1::{Secp256k1, SecretKey},
    sighash::SighashCache,
    transaction,
};
use framkey_core::{FramkeyError, Result};
use framkey_crypto::{SecretBytes, encode_hex};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const DEFAULT_BTC_TEST_NETWORK: BtcNetwork = BtcNetwork::Testnet4;
pub const USER_VISIBLE_BTC_NETWORKS: [BtcNetwork; 2] = [BtcNetwork::Mainnet, BtcNetwork::Testnet4];
pub const BTC_DEFAULT_FEE_RATE_SAT_VB: u64 = 2;
pub const BTC_MAX_FEE_RATE_SAT_VB: u64 = 1_000;
pub const BTC_MAX_SPEND_INPUTS: usize = 64;
pub const BTC_P2WPKH_DUST_LIMIT_SAT: u64 = 546;
const BTC_P2WPKH_INPUT_VBYTES: u64 = 68;
const BTC_P2WPKH_OUTPUT_VBYTES: u64 = 31;
const BTC_TX_OVERHEAD_VBYTES: u64 = 10;
const BTC_MAX_PSBT_BYTES: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtcNetwork {
    Mainnet,
    Testnet4,
    Signet,
    Regtest,
}

impl BtcNetwork {
    pub fn id(self) -> &'static str {
        match self {
            Self::Mainnet => "bitcoin-mainnet",
            Self::Testnet4 => "bitcoin-testnet4",
            Self::Signet => "bitcoin-signet",
            Self::Regtest => "bitcoin-regtest",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Mainnet => "Bitcoin",
            Self::Testnet4 => "Bitcoin Testnet4",
            Self::Signet => "Bitcoin Signet",
            Self::Regtest => "Bitcoin Regtest",
        }
    }

    pub fn is_test_network(self) -> bool {
        !matches!(self, Self::Mainnet)
    }

    pub fn role(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet4 => "public_testnet",
            Self::Signet => "controlled_testnet",
            Self::Regtest => "local_regtest",
        }
    }

    pub fn is_user_visible_default(self) -> bool {
        USER_VISIBLE_BTC_NETWORKS.contains(&self)
    }

    pub fn bitcoin_network(self) -> Network {
        match self {
            Self::Mainnet => Network::Bitcoin,
            Self::Testnet4 => Network::Testnet4,
            Self::Signet => Network::Signet,
            Self::Regtest => Network::Regtest,
        }
    }
}

impl fmt::Display for BtcNetwork {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.id())
    }
}

impl FromStr for BtcNetwork {
    type Err = FramkeyError;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "bitcoin-mainnet" | "mainnet" => Ok(Self::Mainnet),
            "bitcoin-testnet4" | "testnet4" => Ok(Self::Testnet4),
            "bitcoin-signet" | "signet" => Ok(Self::Signet),
            "bitcoin-regtest" | "regtest" => Ok(Self::Regtest),
            _ => Err(FramkeyError::invalid_data("unsupported BTC network")),
        }
    }
}

impl Serialize for BtcNetwork {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for BtcNetwork {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BtcAccount {
    pub network: BtcNetwork,
    pub address: String,
    pub address_type: String,
    pub script_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BtcUtxo {
    pub txid: String,
    pub vout: u32,
    pub value_sat: u64,
    pub confirmed: bool,
    pub block_height: Option<u64>,
    pub block_hash: Option<String>,
    pub block_time: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BtcBalance {
    pub network: BtcNetwork,
    pub address: String,
    pub confirmed_sat: u64,
    pub unconfirmed_sat: u64,
    pub spendable_sat: u64,
    pub utxo_count: usize,
    pub spendable_utxo_count: usize,
    pub utxos: Vec<BtcUtxo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BtcSpendRequest {
    pub network: BtcNetwork,
    pub from_address: String,
    pub to_address: String,
    pub amount_sat: u64,
    pub fee_rate_sat_vb: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BtcOutputPlan {
    pub kind: String,
    pub address: String,
    pub value_sat: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BtcSpendPlan {
    pub network: BtcNetwork,
    pub from_address: String,
    pub to_address: String,
    pub amount_sat: u64,
    pub fee_sat: u64,
    pub fee_rate_sat_vb: u64,
    pub input_value_sat: u64,
    pub change_sat: u64,
    pub dust_limit_sat: u64,
    pub estimated_vbytes: u64,
    pub selected_utxos: Vec<BtcUtxo>,
    pub outputs: Vec<BtcOutputPlan>,
    pub policy: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BtcPreparedSpend {
    pub plan: BtcSpendPlan,
    pub psbt_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BtcSignedTransaction {
    pub network: BtcNetwork,
    pub address: String,
    pub transaction_id: String,
    pub raw_transaction: String,
    pub vbytes: usize,
}

pub fn p2wpkh_account_from_secret(
    secret: &SecretBytes<32>,
    network: BtcNetwork,
) -> Result<BtcAccount> {
    let private_key = private_key_from_secret(secret, network)?;
    let secp = Secp256k1::new();
    let public_key = CompressedPublicKey::from_private_key(&secp, &private_key)
        .map_err(|_| FramkeyError::invalid_data("BTC public key must be compressed"))?;
    let address = Address::p2wpkh(&public_key, network.bitcoin_network());
    Ok(BtcAccount {
        network,
        address: address.to_string(),
        address_type: "p2wpkh".to_owned(),
        script_policy: "single_key_p2wpkh".to_owned(),
    })
}

pub fn validate_private_key_bytes(bytes: &[u8; 32]) -> Result<()> {
    SecretKey::from_slice(bytes)
        .map(|_| ())
        .map_err(|_| FramkeyError::invalid_data("invalid secp256k1 private key"))
}

pub fn validate_address(address: &str, network: BtcNetwork) -> Result<Address> {
    let address = Address::from_str(address.trim())
        .map_err(|_| FramkeyError::invalid_data("BTC address is malformed"))?
        .require_network(network.bitcoin_network())
        .map_err(|_| FramkeyError::invalid_data("BTC address is not valid for selected network"))?;
    Ok(address)
}

pub fn validate_p2wpkh_address(address: &str, network: BtcNetwork) -> Result<Address> {
    let address = validate_address(address, network)?;
    if !address.script_pubkey().is_p2wpkh() {
        return Err(FramkeyError::unsupported(
            "BTC send is currently limited to native SegWit P2WPKH addresses",
        ));
    }
    Ok(address)
}

pub fn balance_from_utxos(network: BtcNetwork, address: &str, utxos: Vec<BtcUtxo>) -> BtcBalance {
    let mut confirmed_sat = 0_u64;
    let mut unconfirmed_sat = 0_u64;
    let mut spendable_sat = 0_u64;
    let mut spendable_utxo_count = 0_usize;
    for utxo in &utxos {
        if utxo.confirmed {
            confirmed_sat = confirmed_sat.saturating_add(utxo.value_sat);
            spendable_sat = spendable_sat.saturating_add(utxo.value_sat);
            spendable_utxo_count = spendable_utxo_count.saturating_add(1);
        } else {
            unconfirmed_sat = unconfirmed_sat.saturating_add(utxo.value_sat);
        }
    }
    BtcBalance {
        network,
        address: address.to_owned(),
        confirmed_sat,
        unconfirmed_sat,
        spendable_sat,
        utxo_count: utxos.len(),
        spendable_utxo_count,
        utxos,
    }
}

pub fn utxos_from_esplora_value(value: &Value) -> Result<Vec<BtcUtxo>> {
    let items = value
        .as_array()
        .ok_or_else(|| FramkeyError::invalid_data("Esplora UTXO response must be an array"))?;
    items
        .iter()
        .map(utxo_from_esplora_item)
        .collect::<Result<Vec<_>>>()
}

pub fn prepare_p2wpkh_spend(
    request: &BtcSpendRequest,
    utxos: &[BtcUtxo],
) -> Result<BtcPreparedSpend> {
    if !request.network.is_user_visible_default() {
        return Err(FramkeyError::unsupported(
            "BTC send is enabled only for the user-visible BTC account networks",
        ));
    }
    let from_address = validate_p2wpkh_address(&request.from_address, request.network)?;
    let to_address = validate_address(&request.to_address, request.network)?;
    if request.amount_sat < BTC_P2WPKH_DUST_LIMIT_SAT {
        return Err(FramkeyError::invalid_data(format!(
            "BTC amount must be at least {BTC_P2WPKH_DUST_LIMIT_SAT} sat"
        )));
    }
    if request.fee_rate_sat_vb == 0 || request.fee_rate_sat_vb > BTC_MAX_FEE_RATE_SAT_VB {
        return Err(FramkeyError::invalid_data(format!(
            "BTC fee rate must be between 1 and {BTC_MAX_FEE_RATE_SAT_VB} sat/vB"
        )));
    }

    let selected =
        select_confirmed_utxos(utxos, request.amount_sat, request.fee_rate_sat_vb, true)?;
    let input_value_sat = selected
        .iter()
        .try_fold(0_u64, |sum, utxo| sum.checked_add(utxo.value_sat))
        .ok_or_else(|| FramkeyError::invalid_data("BTC selected input value overflowed"))?;
    let mut estimated_vbytes = estimate_p2wpkh_vbytes(selected.len(), 2)?;
    let mut required_fee_sat = fee_for_vbytes(estimated_vbytes, request.fee_rate_sat_vb)?;
    let mut change_sat = input_value_sat
        .checked_sub(request.amount_sat)
        .and_then(|remaining| remaining.checked_sub(required_fee_sat))
        .ok_or_else(|| {
            FramkeyError::invalid_data("selected BTC UTXOs do not cover amount and fee")
        })?;

    if change_sat > 0 && change_sat < BTC_P2WPKH_DUST_LIMIT_SAT {
        estimated_vbytes = estimate_p2wpkh_vbytes(selected.len(), 1)?;
        required_fee_sat = fee_for_vbytes(estimated_vbytes, request.fee_rate_sat_vb)?;
        let no_change_fee = input_value_sat
            .checked_sub(request.amount_sat)
            .ok_or_else(|| FramkeyError::invalid_data("selected BTC UTXOs do not cover amount"))?;
        if no_change_fee < required_fee_sat {
            return Err(FramkeyError::invalid_data(
                "selected BTC UTXOs do not cover no-change fee",
            ));
        }
        change_sat = 0;
        required_fee_sat = no_change_fee;
    }

    if change_sat >= BTC_P2WPKH_DUST_LIMIT_SAT {
        let min_fee = fee_for_vbytes(estimated_vbytes, request.fee_rate_sat_vb)?;
        if required_fee_sat < min_fee {
            return Err(FramkeyError::invalid_data(
                "selected BTC UTXOs do not cover required fee",
            ));
        }
    }

    let mut outputs = vec![TxOut {
        value: Amount::from_sat(request.amount_sat),
        script_pubkey: to_address.script_pubkey(),
    }];
    let mut output_plan = vec![BtcOutputPlan {
        kind: "recipient".to_owned(),
        address: to_address.to_string(),
        value_sat: request.amount_sat,
    }];
    if change_sat >= BTC_P2WPKH_DUST_LIMIT_SAT {
        outputs.push(TxOut {
            value: Amount::from_sat(change_sat),
            script_pubkey: from_address.script_pubkey(),
        });
        output_plan.push(BtcOutputPlan {
            kind: "change".to_owned(),
            address: from_address.to_string(),
            value_sat: change_sat,
        });
    }

    let unsigned_tx = unsigned_transaction_from_plan(&selected, outputs)?;
    let owned_script = from_address.script_pubkey();
    let inputs = selected
        .iter()
        .map(|utxo| {
            let mut input = psbt::Input {
                witness_utxo: Some(TxOut {
                    value: Amount::from_sat(utxo.value_sat),
                    script_pubkey: owned_script.clone(),
                }),
                sighash_type: Some(EcdsaSighashType::All.into()),
                ..Default::default()
            };
            input.final_script_sig = None;
            input.final_script_witness = None;
            input
        })
        .collect::<Vec<_>>();
    let psbt = Psbt {
        unsigned_tx,
        version: 0,
        xpub: Default::default(),
        proprietary: Default::default(),
        unknown: Default::default(),
        inputs,
        outputs: vec![psbt::Output::default(); output_plan.len()],
    };

    let psbt_bytes = psbt.serialize();
    if psbt_bytes.len() > BTC_MAX_PSBT_BYTES {
        return Err(FramkeyError::invalid_data(
            "BTC PSBT exceeds signer helper limit",
        ));
    }

    let plan = BtcSpendPlan {
        network: request.network,
        from_address: from_address.to_string(),
        to_address: to_address.to_string(),
        amount_sat: request.amount_sat,
        fee_sat: required_fee_sat,
        fee_rate_sat_vb: request.fee_rate_sat_vb,
        input_value_sat,
        change_sat,
        dust_limit_sat: BTC_P2WPKH_DUST_LIMIT_SAT,
        estimated_vbytes,
        selected_utxos: selected,
        outputs: output_plan,
        policy: json!({
            "canSign": true,
            "scriptPolicy": "single_key_p2wpkh",
            "sighash": "SIGHASH_ALL",
            "rbf": true,
            "inputConfirmationPolicy": "confirmed_only",
            "changePolicy": "return_to_source_address",
            "dustLimitSat": BTC_P2WPKH_DUST_LIMIT_SAT,
        }),
    };
    Ok(BtcPreparedSpend { plan, psbt_bytes })
}

pub fn sign_p2wpkh_psbt(
    secret: &SecretBytes<32>,
    network: BtcNetwork,
    expected_address: &str,
    psbt_bytes: &[u8],
) -> Result<BtcSignedTransaction> {
    if psbt_bytes.len() > BTC_MAX_PSBT_BYTES {
        return Err(FramkeyError::invalid_data(
            "BTC PSBT exceeds signer helper limit",
        ));
    }
    let account = p2wpkh_account_from_secret(secret, network)?;
    if account.address != expected_address.trim() {
        return Err(FramkeyError::invalid_data(format!(
            "BTC signing account mismatch: requested {}, vault {}",
            expected_address, account.address
        )));
    }
    let address = validate_p2wpkh_address(expected_address, network)?;
    let private_key = private_key_from_secret(secret, network)?;
    let secp = Secp256k1::new();
    let public_key = private_key.public_key(&secp);
    if !address.is_related_to_pubkey(&public_key) {
        return Err(FramkeyError::invalid_data(
            "BTC signing key is not related to expected address",
        ));
    }
    let psbt = Psbt::deserialize(psbt_bytes)
        .map_err(|_| FramkeyError::invalid_data("BTC PSBT is malformed"))?;
    validate_owned_p2wpkh_psbt(&psbt, &address)?;

    let mut tx = psbt.unsigned_tx.clone();
    {
        let mut cache = SighashCache::new(&mut tx);
        for (index, input) in psbt.inputs.iter().enumerate() {
            let utxo = input
                .witness_utxo
                .as_ref()
                .ok_or_else(|| FramkeyError::invalid_data("BTC PSBT input missing witness UTXO"))?;
            let sighash_type = input.ecdsa_hash_ty().map_err(|_| {
                FramkeyError::invalid_data("BTC PSBT contains non-standard sighash")
            })?;
            if sighash_type != EcdsaSighashType::All {
                return Err(FramkeyError::unsupported(
                    "BTC PSBT signing is currently limited to SIGHASH_ALL",
                ));
            }
            let sighash = cache
                .p2wpkh_signature_hash(index, &utxo.script_pubkey, utxo.value, sighash_type)
                .map_err(|_| FramkeyError::invalid_data("BTC PSBT sighash failed"))?;
            let signature = ecdsa::Signature {
                signature: secp.sign_ecdsa(&sighash.into(), &private_key.inner),
                sighash_type,
            };
            let witness = cache
                .witness_mut(index)
                .ok_or_else(|| FramkeyError::invalid_data("BTC PSBT input index is invalid"))?;
            *witness = Witness::p2wpkh(&signature, &public_key.inner);
        }
    }

    let input_value = psbt
        .inputs
        .iter()
        .try_fold(0_u64, |sum, input| {
            let value = input.witness_utxo.as_ref()?.value.to_sat();
            sum.checked_add(value)
        })
        .ok_or_else(|| FramkeyError::invalid_data("BTC PSBT input value overflowed"))?;
    let output_value = tx
        .output
        .iter()
        .try_fold(0_u64, |sum, output| sum.checked_add(output.value.to_sat()))
        .ok_or_else(|| FramkeyError::invalid_data("BTC PSBT output value overflowed"))?;
    if input_value <= output_value {
        return Err(FramkeyError::invalid_data("BTC PSBT fee must be positive"));
    }
    let transaction_id = tx.compute_txid().to_string();
    let raw_transaction = encode_hex(&serialize(&tx));
    Ok(BtcSignedTransaction {
        network,
        address: address.to_string(),
        transaction_id,
        raw_transaction,
        vbytes: tx.vsize(),
    })
}

pub fn validate_signed_transaction_for_plan(
    plan: &BtcSpendPlan,
    raw_transaction_hex: &str,
) -> Result<()> {
    let bytes = decode_hex(raw_transaction_hex)?;
    let tx: Transaction = deserialize(&bytes)
        .map_err(|_| FramkeyError::invalid_data("signed BTC transaction is malformed"))?;
    if tx.version != transaction::Version::TWO {
        return Err(FramkeyError::invalid_data(
            "signed BTC transaction version mismatch",
        ));
    }
    if tx.lock_time != absolute::LockTime::ZERO {
        return Err(FramkeyError::invalid_data(
            "signed BTC transaction locktime mismatch",
        ));
    }
    if tx.input.len() != plan.selected_utxos.len() {
        return Err(FramkeyError::invalid_data(
            "signed BTC transaction input count mismatch",
        ));
    }
    if tx.output.len() != plan.outputs.len() {
        return Err(FramkeyError::invalid_data(
            "signed BTC transaction output count mismatch",
        ));
    }
    let selected_input_value = plan
        .selected_utxos
        .iter()
        .try_fold(0_u64, |sum, utxo| sum.checked_add(utxo.value_sat))
        .ok_or_else(|| FramkeyError::invalid_data("BTC plan input value overflowed"))?;
    if selected_input_value != plan.input_value_sat {
        return Err(FramkeyError::invalid_data(
            "BTC spend plan input value mismatch",
        ));
    }
    for (index, (tx_input, utxo)) in tx.input.iter().zip(&plan.selected_utxos).enumerate() {
        let txid = Txid::from_str(&utxo.txid)
            .map_err(|_| FramkeyError::invalid_data("BTC plan UTXO txid is malformed"))?;
        if tx_input.previous_output
            != (OutPoint {
                txid,
                vout: utxo.vout,
            })
        {
            return Err(FramkeyError::invalid_data(format!(
                "signed BTC transaction input {index} outpoint mismatch"
            )));
        }
        if !tx_input.script_sig.is_empty() {
            return Err(FramkeyError::invalid_data(format!(
                "signed BTC transaction input {index} has unexpected scriptSig"
            )));
        }
        if tx_input.sequence != Sequence::ENABLE_RBF_NO_LOCKTIME {
            return Err(FramkeyError::invalid_data(format!(
                "signed BTC transaction input {index} sequence mismatch"
            )));
        }
        if tx_input.witness.is_empty() {
            return Err(FramkeyError::invalid_data(format!(
                "signed BTC transaction input {index} is missing witness"
            )));
        }
    }
    for (index, (tx_output, planned)) in tx.output.iter().zip(&plan.outputs).enumerate() {
        let address = validate_address(&planned.address, plan.network)?;
        if tx_output.script_pubkey != address.script_pubkey() {
            return Err(FramkeyError::invalid_data(format!(
                "signed BTC transaction output {index} script mismatch"
            )));
        }
        if tx_output.value.to_sat() != planned.value_sat {
            return Err(FramkeyError::invalid_data(format!(
                "signed BTC transaction output {index} value mismatch"
            )));
        }
    }
    let fee = plan
        .input_value_sat
        .checked_sub(
            tx.output
                .iter()
                .try_fold(0_u64, |sum, output| sum.checked_add(output.value.to_sat()))
                .ok_or_else(|| {
                    FramkeyError::invalid_data("signed BTC transaction output overflowed")
                })?,
        )
        .ok_or_else(|| {
            FramkeyError::invalid_data("signed BTC transaction outputs exceed inputs")
        })?;
    if fee != plan.fee_sat {
        return Err(FramkeyError::invalid_data(
            "signed BTC transaction fee mismatch",
        ));
    }
    if tx.compute_txid().to_string().is_empty() {
        return Err(FramkeyError::invalid_data(
            "signed BTC transaction txid missing",
        ));
    }
    Ok(())
}

pub fn transaction_id_from_raw_hex(raw_transaction_hex: &str) -> Result<String> {
    let bytes = decode_hex(raw_transaction_hex)?;
    let tx: Transaction = deserialize(&bytes)
        .map_err(|_| FramkeyError::invalid_data("signed BTC transaction is malformed"))?;
    Ok(tx.compute_txid().to_string())
}

fn private_key_from_secret(secret: &SecretBytes<32>, network: BtcNetwork) -> Result<PrivateKey> {
    let secret_key = SecretKey::from_slice(secret.expose())
        .map_err(|_| FramkeyError::invalid_data("invalid secp256k1 private key"))?;
    Ok(PrivateKey::new(secret_key, network.bitcoin_network()))
}

fn utxo_from_esplora_item(value: &Value) -> Result<BtcUtxo> {
    let object = value
        .as_object()
        .ok_or_else(|| FramkeyError::invalid_data("Esplora UTXO item must be an object"))?;
    let txid = object
        .get("txid")
        .and_then(Value::as_str)
        .ok_or_else(|| FramkeyError::invalid_data("Esplora UTXO missing txid"))?
        .to_owned();
    let vout_u64 = object
        .get("vout")
        .and_then(Value::as_u64)
        .ok_or_else(|| FramkeyError::invalid_data("Esplora UTXO missing vout"))?;
    let vout = u32::try_from(vout_u64)
        .map_err(|_| FramkeyError::invalid_data("Esplora UTXO vout is too large"))?;
    let value_sat = object
        .get("value")
        .and_then(Value::as_u64)
        .ok_or_else(|| FramkeyError::invalid_data("Esplora UTXO missing value"))?;
    let status = object.get("status").and_then(Value::as_object);
    let confirmed = status
        .and_then(|status| status.get("confirmed"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let block_height = status
        .and_then(|status| status.get("block_height"))
        .and_then(Value::as_u64);
    let block_hash = status
        .and_then(|status| status.get("block_hash"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let block_time = status
        .and_then(|status| status.get("block_time"))
        .and_then(Value::as_u64);
    Ok(BtcUtxo {
        txid,
        vout,
        value_sat,
        confirmed,
        block_height,
        block_hash,
        block_time,
    })
}

fn select_confirmed_utxos(
    utxos: &[BtcUtxo],
    amount_sat: u64,
    fee_rate_sat_vb: u64,
    assume_change: bool,
) -> Result<Vec<BtcUtxo>> {
    let mut candidates = utxos
        .iter()
        .filter(|utxo| utxo.confirmed)
        .cloned()
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .value_sat
            .cmp(&left.value_sat)
            .then_with(|| left.txid.cmp(&right.txid))
            .then_with(|| left.vout.cmp(&right.vout))
    });

    let mut selected = Vec::new();
    let mut total = 0_u64;
    for utxo in candidates {
        if selected.len() >= BTC_MAX_SPEND_INPUTS {
            return Err(FramkeyError::invalid_data(
                "BTC spend requires too many UTXOs",
            ));
        }
        total = total
            .checked_add(utxo.value_sat)
            .ok_or_else(|| FramkeyError::invalid_data("BTC selected input value overflowed"))?;
        selected.push(utxo);
        let outputs = if assume_change { 2 } else { 1 };
        let vbytes = estimate_p2wpkh_vbytes(selected.len(), outputs)?;
        let fee = fee_for_vbytes(vbytes, fee_rate_sat_vb)?;
        if total >= amount_sat.saturating_add(fee) {
            return Ok(selected);
        }
    }
    Err(FramkeyError::invalid_data(
        "confirmed BTC balance is insufficient for amount and fee",
    ))
}

fn unsigned_transaction_from_plan(
    selected: &[BtcUtxo],
    outputs: Vec<TxOut>,
) -> Result<Transaction> {
    let inputs = selected
        .iter()
        .map(|utxo| {
            let txid = Txid::from_str(&utxo.txid)
                .map_err(|_| FramkeyError::invalid_data("BTC UTXO txid is malformed"))?;
            Ok(TxIn {
                previous_output: OutPoint {
                    txid,
                    vout: utxo.vout,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Transaction {
        version: transaction::Version::TWO,
        lock_time: absolute::LockTime::ZERO,
        input: inputs,
        output: outputs,
    })
}

fn validate_owned_p2wpkh_psbt(psbt: &Psbt, address: &Address) -> Result<()> {
    if psbt.unsigned_tx.input.is_empty() {
        return Err(FramkeyError::invalid_data(
            "BTC PSBT must have at least one input",
        ));
    }
    if psbt.unsigned_tx.input.len() != psbt.inputs.len() {
        return Err(FramkeyError::invalid_data(
            "BTC PSBT input count does not match unsigned transaction",
        ));
    }
    if psbt.unsigned_tx.output.is_empty() {
        return Err(FramkeyError::invalid_data(
            "BTC PSBT must have at least one output",
        ));
    }
    if psbt.unsigned_tx.input.len() > BTC_MAX_SPEND_INPUTS {
        return Err(FramkeyError::invalid_data("BTC PSBT has too many inputs"));
    }
    let owned_script = address.script_pubkey();
    let mut input_value = 0_u64;
    for (index, input) in psbt.inputs.iter().enumerate() {
        let tx_input = psbt
            .unsigned_tx
            .input
            .get(index)
            .ok_or_else(|| FramkeyError::invalid_data("BTC PSBT input index mismatch"))?;
        if !tx_input.script_sig.is_empty() || !tx_input.witness.is_empty() {
            return Err(FramkeyError::invalid_data(
                "BTC PSBT unsigned transaction must not include signatures",
            ));
        }
        if tx_input.sequence != Sequence::ENABLE_RBF_NO_LOCKTIME {
            return Err(FramkeyError::invalid_data(
                "BTC PSBT must use FRAMKey RBF sequence policy",
            ));
        }
        let utxo = input
            .witness_utxo
            .as_ref()
            .ok_or_else(|| FramkeyError::invalid_data("BTC PSBT input missing witness UTXO"))?;
        if utxo.script_pubkey != owned_script {
            return Err(FramkeyError::invalid_data(
                "BTC PSBT includes an input that is not owned by the selected account",
            ));
        }
        if !utxo.script_pubkey.is_p2wpkh() {
            return Err(FramkeyError::unsupported(
                "BTC PSBT signing is currently limited to P2WPKH inputs",
            ));
        }
        let sighash_type = input
            .ecdsa_hash_ty()
            .map_err(|_| FramkeyError::invalid_data("BTC PSBT contains non-standard sighash"))?;
        if sighash_type != EcdsaSighashType::All {
            return Err(FramkeyError::unsupported(
                "BTC PSBT signing is currently limited to SIGHASH_ALL",
            ));
        }
        input_value = input_value
            .checked_add(utxo.value.to_sat())
            .ok_or_else(|| FramkeyError::invalid_data("BTC PSBT input value overflowed"))?;
    }
    let output_value = psbt
        .unsigned_tx
        .output
        .iter()
        .try_fold(0_u64, |sum, output| sum.checked_add(output.value.to_sat()))
        .ok_or_else(|| FramkeyError::invalid_data("BTC PSBT output value overflowed"))?;
    if input_value <= output_value {
        return Err(FramkeyError::invalid_data("BTC PSBT fee must be positive"));
    }
    Ok(())
}

fn estimate_p2wpkh_vbytes(inputs: usize, outputs: usize) -> Result<u64> {
    let inputs = u64::try_from(inputs)
        .map_err(|_| FramkeyError::invalid_data("BTC input count is too large"))?;
    let outputs = u64::try_from(outputs)
        .map_err(|_| FramkeyError::invalid_data("BTC output count is too large"))?;
    BTC_TX_OVERHEAD_VBYTES
        .checked_add(inputs.saturating_mul(BTC_P2WPKH_INPUT_VBYTES))
        .and_then(|value| value.checked_add(outputs.saturating_mul(BTC_P2WPKH_OUTPUT_VBYTES)))
        .ok_or_else(|| FramkeyError::invalid_data("BTC vbytes estimate overflowed"))
}

fn fee_for_vbytes(vbytes: u64, fee_rate_sat_vb: u64) -> Result<u64> {
    vbytes
        .checked_mul(fee_rate_sat_vb)
        .ok_or_else(|| FramkeyError::invalid_data("BTC fee estimate overflowed"))
}

fn decode_hex(input: &str) -> Result<Vec<u8>> {
    let input = input.trim();
    let input = input.strip_prefix("0x").unwrap_or(input);
    if input.len() % 2 != 0 {
        return Err(FramkeyError::invalid_data("hex string length must be even"));
    }
    let mut bytes = Vec::with_capacity(input.len() / 2);
    let chars = input.as_bytes();
    for index in (0..chars.len()).step_by(2) {
        let high = hex_nibble(chars[index])?;
        let low = hex_nibble(chars[index + 1])?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn hex_nibble(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(FramkeyError::invalid_data(
            "hex string contains invalid digit",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_mainnet_native_segwit_address_from_secret() {
        let secret = SecretBytes::new([1; 32]);
        let account = p2wpkh_account_from_secret(&secret, BtcNetwork::Mainnet).unwrap();

        assert_eq!(account.network, BtcNetwork::Mainnet);
        assert_eq!(account.address_type, "p2wpkh");
        assert_eq!(account.script_policy, "single_key_p2wpkh");
        assert!(account.address.starts_with("bc1q"));
    }

    #[test]
    fn test_networks_use_testnet_hrp() {
        let secret = SecretBytes::new([2; 32]);
        let account = p2wpkh_account_from_secret(&secret, BtcNetwork::Testnet4).unwrap();

        assert_eq!(account.network.id(), "bitcoin-testnet4");
        assert!(account.address.starts_with("tb1q"));
        assert!(account.network.is_test_network());
    }

    #[test]
    fn chooses_testnet4_as_default_user_test_network() {
        assert_eq!(DEFAULT_BTC_TEST_NETWORK, BtcNetwork::Testnet4);
        assert!(BtcNetwork::Testnet4.is_user_visible_default());
        assert!(!BtcNetwork::Signet.is_user_visible_default());
        assert_eq!(BtcNetwork::Testnet4.role(), "public_testnet");
        assert_eq!(BtcNetwork::Signet.role(), "controlled_testnet");
        assert_eq!(
            serde_json::to_string(&BtcNetwork::Testnet4).unwrap(),
            r#""bitcoin-testnet4""#
        );
        assert_eq!(
            serde_json::from_str::<BtcNetwork>(r#""testnet4""#).unwrap(),
            BtcNetwork::Testnet4
        );
    }

    #[test]
    fn rejects_invalid_secret_bytes() {
        assert!(validate_private_key_bytes(&[0; 32]).is_err());
    }

    #[test]
    fn parses_esplora_utxos_and_separates_confirmed_balance() {
        let value = json!([
            {
                "txid": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "vout": 0,
                "value": 10_000,
                "status": {
                    "confirmed": true,
                    "block_height": 1,
                    "block_hash": "bbbb",
                    "block_time": 100,
                },
            },
            {
                "txid": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "vout": 1,
                "value": 2_000,
                "status": {
                    "confirmed": false,
                },
            },
        ]);
        let utxos = utxos_from_esplora_value(&value).unwrap();
        let balance = balance_from_utxos(BtcNetwork::Testnet4, "tb1qexample", utxos);

        assert_eq!(balance.confirmed_sat, 10_000);
        assert_eq!(balance.unconfirmed_sat, 2_000);
        assert_eq!(balance.spendable_sat, 10_000);
        assert_eq!(balance.spendable_utxo_count, 1);
    }

    #[test]
    fn prepares_and_signs_owned_p2wpkh_psbt() {
        let secret = SecretBytes::new([3; 32]);
        let account = p2wpkh_account_from_secret(&secret, BtcNetwork::Testnet4).unwrap();
        let recipient =
            p2wpkh_account_from_secret(&SecretBytes::new([6; 32]), BtcNetwork::Testnet4).unwrap();
        let request = BtcSpendRequest {
            network: BtcNetwork::Testnet4,
            from_address: account.address.clone(),
            to_address: recipient.address.clone(),
            amount_sat: 1_000,
            fee_rate_sat_vb: 2,
        };
        let prepared = prepare_p2wpkh_spend(
            &request,
            &[BtcUtxo {
                txid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
                vout: 0,
                value_sat: 10_000,
                confirmed: true,
                block_height: Some(1),
                block_hash: None,
                block_time: None,
            }],
        )
        .unwrap();

        assert_eq!(prepared.plan.amount_sat, 1_000);
        assert!(prepared.plan.fee_sat > 0);
        assert_eq!(prepared.plan.selected_utxos.len(), 1);

        let signed = sign_p2wpkh_psbt(
            &secret,
            BtcNetwork::Testnet4,
            &account.address,
            &prepared.psbt_bytes,
        )
        .unwrap();

        assert_eq!(signed.network, BtcNetwork::Testnet4);
        assert_eq!(signed.address, account.address);
        assert_eq!(signed.transaction_id.len(), 64);
        assert!(signed.raw_transaction.len() > 100);
        assert!(signed.vbytes > 0);
        validate_signed_transaction_for_plan(&prepared.plan, &signed.raw_transaction).unwrap();

        let mut tampered_plan = prepared.plan.clone();
        tampered_plan.outputs[0].address = account.address;
        let error = validate_signed_transaction_for_plan(&tampered_plan, &signed.raw_transaction)
            .unwrap_err();
        assert!(error.to_string().contains("output 0 script mismatch"));
    }

    #[test]
    fn coin_selection_ignores_many_dust_utxos_when_large_utxo_can_pay() {
        let secret = SecretBytes::new([7; 32]);
        let account = p2wpkh_account_from_secret(&secret, BtcNetwork::Testnet4).unwrap();
        let mut utxos = (1_u64..=80)
            .map(|index| BtcUtxo {
                txid: format!("{index:064x}"),
                vout: 0,
                value_sat: 1,
                confirmed: true,
                block_height: Some(index),
                block_hash: None,
                block_time: None,
            })
            .collect::<Vec<_>>();
        utxos.push(BtcUtxo {
            txid: format!("{:064x}", 999_u64),
            vout: 0,
            value_sat: 20_000,
            confirmed: true,
            block_height: Some(999),
            block_hash: None,
            block_time: None,
        });

        let prepared = prepare_p2wpkh_spend(
            &BtcSpendRequest {
                network: BtcNetwork::Testnet4,
                from_address: account.address.clone(),
                to_address: account.address,
                amount_sat: 1_000,
                fee_rate_sat_vb: 2,
            },
            &utxos,
        )
        .unwrap();

        assert_eq!(prepared.plan.selected_utxos.len(), 1);
        assert_eq!(prepared.plan.selected_utxos[0].value_sat, 20_000);
    }

    #[test]
    fn rejects_psbt_not_owned_by_expected_address() {
        let secret = SecretBytes::new([4; 32]);
        let other_secret = SecretBytes::new([5; 32]);
        let account = p2wpkh_account_from_secret(&secret, BtcNetwork::Testnet4).unwrap();
        let other = p2wpkh_account_from_secret(&other_secret, BtcNetwork::Testnet4).unwrap();
        let prepared = prepare_p2wpkh_spend(
            &BtcSpendRequest {
                network: BtcNetwork::Testnet4,
                from_address: other.address,
                to_address: account.address.clone(),
                amount_sat: 1_000,
                fee_rate_sat_vb: 2,
            },
            &[BtcUtxo {
                txid: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
                vout: 1,
                value_sat: 10_000,
                confirmed: true,
                block_height: Some(1),
                block_hash: None,
                block_time: None,
            }],
        )
        .unwrap();

        let error = sign_p2wpkh_psbt(
            &secret,
            BtcNetwork::Testnet4,
            &account.address,
            &prepared.psbt_bytes,
        )
        .unwrap_err();
        assert!(error.to_string().contains("not owned"));
    }
}
