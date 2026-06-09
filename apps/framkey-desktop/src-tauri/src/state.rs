use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs::{self},
    path::{Path, PathBuf},
    sync::{Condvar, Mutex},
    time::Duration,
};

use crate::review;
use crate::*;
use anyhow::{Context, Result};
use framkey_btc::{BtcNetwork, p2wpkh_account_from_secret, sign_p2wpkh_psbt};
use framkey_crypto::{SecretBytes, encode_hex, random_array};
use framkey_evm::{
    EvmAddress, EvmTransaction, address_from_secret, personal_sign, sign_transaction,
    sign_typed_data_v4,
};
use framkey_ipc::{
    SignerPersonalSignResponse, SignerSignBtcPsbtResponse, SignerSignTypedDataResponse,
    SignerVaultMetadata,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

pub(crate) struct AppState {
    pub(crate) config: Mutex<Option<DesktopConfig>>,
    pub(crate) review_queue: Mutex<ReviewQueue>,
    pub(crate) review_condvar: Condvar,
    pub(crate) mock_wallet: Mutex<Option<MockWallet>>,
    pub(crate) account_connect_lock: Mutex<()>,
    pub(crate) account_session_sequence: Mutex<u64>,
    pub(crate) connected_account: Mutex<Option<ConnectedAccountSession>>,
    pub(crate) account_permissions: Mutex<AccountPermissionStore>,
    pub(crate) provider_events: Mutex<ProviderEventLog>,
    pub(crate) dapp_session: Mutex<DappSessionState>,
    pub(crate) watched_assets: Mutex<WatchedAssetStore>,
    pub(crate) wallet_ui_state_path: Option<PathBuf>,
    pub(crate) wallet_ui_state_persistence: Mutex<WalletUiStatePersistenceStatus>,
    pub(crate) transaction_activity: Mutex<TransactionActivityLog>,
    pub(crate) pending_nonces: Mutex<BTreeMap<(String, String), BTreeSet<u128>>>,
    pub(crate) transaction_activity_persistence_path: Option<PathBuf>,
    pub(crate) transaction_activity_persistence: Mutex<TransactionActivityPersistenceStatus>,
    pub(crate) recovery_ui_state: Mutex<RecoveryUiState>,
    pub(crate) recovery_ui_state_path: Option<PathBuf>,
    pub(crate) recovery_ui_state_persistence: Mutex<RecoveryUiStatePersistenceStatus>,
}

#[derive(Debug, Clone)]
pub(crate) struct ConnectedAccountSession {
    pub(crate) address: String,
    pub(crate) accounts: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingNonceReservation {
    pub(crate) chain_id: String,
    pub(crate) address: String,
    pub(crate) nonce: String,
    pub(crate) nonce_value: u128,
}

impl ConnectedAccountSession {
    pub(crate) fn new(address: String, accounts: Value) -> Result<Self> {
        let _address = address
            .parse::<EvmAddress>()
            .map_err(|_| anyhow::anyhow!("connected account address is not a valid EVM address"))?;
        Ok(Self { address, accounts })
    }

    pub(crate) fn from_account(account: &DesktopAccount) -> Result<Self> {
        Self::new(account.address.clone(), account.accounts.clone())
    }

    pub(crate) fn to_minimal_account(&self) -> DesktopAccount {
        DesktopAccount {
            address: self.address.clone(),
            accounts: self.accounts.clone(),
            wallet: json!({
                "kind": "connected_session",
                "scope": "address_only",
            }),
            metadata: json!({}),
            keychain: None,
            helper_report: None,
        }
    }
}

impl AppState {
    pub(crate) fn new() -> Self {
        let (recovery_ui_state_path, recovery_ui_state, recovery_ui_state_persistence) =
            load_recovery_ui_state_persistence();
        Self {
            config: Mutex::new(None),
            review_queue: Mutex::new(ReviewQueue::new()),
            review_condvar: Condvar::new(),
            mock_wallet: Mutex::new(None),
            account_connect_lock: Mutex::new(()),
            account_session_sequence: Mutex::new(0),
            connected_account: Mutex::new(None),
            account_permissions: Mutex::new(AccountPermissionStore::new()),
            provider_events: Mutex::new(ProviderEventLog::new()),
            dapp_session: Mutex::new(DappSessionState::new()),
            watched_assets: Mutex::new(WatchedAssetStore::new()),
            wallet_ui_state_path: None,
            wallet_ui_state_persistence: Mutex::new(WalletUiStatePersistenceStatus::disabled()),
            transaction_activity: Mutex::new(TransactionActivityLog::new()),
            pending_nonces: Mutex::new(BTreeMap::new()),
            transaction_activity_persistence_path: None,
            transaction_activity_persistence: Mutex::new(
                TransactionActivityPersistenceStatus::disabled(),
            ),
            recovery_ui_state: Mutex::new(recovery_ui_state),
            recovery_ui_state_path,
            recovery_ui_state_persistence: Mutex::new(recovery_ui_state_persistence),
        }
    }

    pub(crate) fn load() -> Self {
        let state = match transaction_activity_path() {
            Ok(path) => Self::new_with_transaction_activity_persistence(path),
            Err(error) => Self::new_with_transaction_activity_persistence_warning(error),
        };
        state.with_wallet_ui_state_persistence_from_default_path()
    }

    pub(crate) fn new_with_transaction_activity_persistence(path: PathBuf) -> Self {
        let mut persistence = TransactionActivityPersistenceStatus::enabled();
        let activity = match TransactionActivityLog::read_from_path(&path) {
            Ok(activity) => {
                persistence.restored = activity.len() > 0;
                persistence.items_restored = activity.len();
                activity
            }
            Err(error) => {
                persistence.warning = Some(truncate_for_event(&error.to_string(), 240));
                TransactionActivityLog::new()
            }
        };
        let (recovery_ui_state_path, recovery_ui_state, recovery_ui_state_persistence) =
            load_recovery_ui_state_persistence();
        Self {
            config: Mutex::new(None),
            review_queue: Mutex::new(ReviewQueue::new()),
            review_condvar: Condvar::new(),
            mock_wallet: Mutex::new(None),
            account_connect_lock: Mutex::new(()),
            account_session_sequence: Mutex::new(0),
            connected_account: Mutex::new(None),
            account_permissions: Mutex::new(AccountPermissionStore::new()),
            provider_events: Mutex::new(ProviderEventLog::new()),
            dapp_session: Mutex::new(DappSessionState::new()),
            watched_assets: Mutex::new(WatchedAssetStore::new()),
            wallet_ui_state_path: None,
            wallet_ui_state_persistence: Mutex::new(WalletUiStatePersistenceStatus::disabled()),
            transaction_activity: Mutex::new(activity),
            pending_nonces: Mutex::new(BTreeMap::new()),
            transaction_activity_persistence_path: Some(path),
            transaction_activity_persistence: Mutex::new(persistence),
            recovery_ui_state: Mutex::new(recovery_ui_state),
            recovery_ui_state_path,
            recovery_ui_state_persistence: Mutex::new(recovery_ui_state_persistence),
        }
    }

    pub(crate) fn new_with_transaction_activity_persistence_warning(error: anyhow::Error) -> Self {
        let mut state = Self::new();
        state.transaction_activity_persistence =
            Mutex::new(TransactionActivityPersistenceStatus::unavailable(
                truncate_for_event(&error.to_string(), 240),
            ));
        state
    }

    pub(crate) fn with_wallet_ui_state_persistence_from_default_path(mut self) -> Self {
        match wallet_ui_state_path() {
            Ok(path) => self.load_wallet_ui_state_from_path(path),
            Err(error) => {
                self.wallet_ui_state_persistence =
                    Mutex::new(WalletUiStatePersistenceStatus::unavailable(
                        truncate_for_event(&error.to_string(), 240),
                    ));
            }
        }
        self
    }

    #[cfg(test)]
    pub(crate) fn new_with_wallet_ui_state_persistence(path: PathBuf) -> Self {
        Self::new().with_wallet_ui_state_persistence_path(path)
    }

    #[cfg(test)]
    pub(crate) fn with_wallet_ui_state_persistence_path(mut self, path: PathBuf) -> Self {
        self.load_wallet_ui_state_from_path(path);
        self
    }

    pub(crate) fn load_wallet_ui_state_from_path(&mut self, path: PathBuf) {
        let mut persistence = WalletUiStatePersistenceStatus::enabled();
        let watched_assets = match WatchedAssetStore::read_from_path(&path) {
            Ok(store) => {
                persistence.restored = store.len() > 0;
                persistence.watched_assets_restored = store.len();
                store
            }
            Err(error) => {
                persistence.warning = Some(truncate_for_event(&error.to_string(), 240));
                WatchedAssetStore::new()
            }
        };
        self.watched_assets = Mutex::new(watched_assets);
        self.wallet_ui_state_path = Some(path);
        self.wallet_ui_state_persistence = Mutex::new(persistence);
    }

    pub(crate) fn with_config<R>(
        &self,
        use_config: impl FnOnce(&DesktopConfig) -> Result<R>,
    ) -> Result<R> {
        let mut guard = self
            .config
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey desktop config lock poisoned"))?;
        if guard.is_none() {
            *guard = Some(DesktopConfig::load()?);
        }
        use_config(guard.as_ref().expect("config initialized above"))
    }

    pub(crate) fn config_snapshot(&self) -> Result<DesktopConfig> {
        self.with_config(|config| Ok(config.clone()))
    }

    pub(crate) fn switch_session_chain(
        &self,
        chain: SupportedChain,
        alchemy_token: Option<&str>,
    ) -> Result<DesktopConfig> {
        let mut guard = self
            .config
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey desktop config lock poisoned"))?;
        if guard.is_none() {
            *guard = Some(DesktopConfig::load()?);
        }
        let config = guard.as_mut().expect("config initialized above");
        config.switch_to_supported_chain(chain, alchemy_token)?;
        Ok(config.clone())
    }

    pub(crate) fn capture_review_request(
        &self,
        config: &DesktopConfig,
        request: &ProviderRequest,
    ) -> Result<ReviewRequest> {
        let transaction_review = match dangerous_method_kind(&request.method) {
            Some(review::ReviewMethodKind::Transaction) => {
                let mut review = config.simulation.transaction_review(
                    &request.method,
                    &request.params,
                    &config.chain_id,
                );
                enrich_aave_account_evidence(config, request, &mut review);
                Some(review)
            }
            _ => None,
        };
        let transaction_asset_context = transaction_review
            .as_ref()
            .and_then(|review| transaction_asset_context(config, &review.simulation));

        let review = {
            let mut guard = self
                .review_queue
                .lock()
                .map_err(|_| anyhow::anyhow!("FRAMKey review queue lock poisoned"))?;
            guard.capture_with_asset_context(
                request.id.clone(),
                request.method.clone(),
                request.origin.clone(),
                &request.params,
                &config.chain_id,
                transaction_review,
                transaction_asset_context,
            )?
        };
        if matches!(
            review.kind,
            review::ReviewMethodKind::Transaction | review::ReviewMethodKind::BtcTransaction
        ) {
            self.record_transaction_review(&review)?;
        }
        Ok(review)
    }

    pub(crate) fn load_account(&self, config: &DesktopConfig) -> Result<DesktopAccount> {
        match config.wallet {
            DesktopWalletConfig::KeychainVault => load_keychain_account(config),
            DesktopWalletConfig::MockInMemory => {
                let wallet = self.mock_wallet()?;
                let accounts = desktop_accounts_value(
                    config,
                    &wallet.address,
                    Some(&wallet.btc_mainnet_address),
                    Some(&wallet.btc_testnet4_address),
                );
                Ok(DesktopAccount {
                    address: wallet.address,
                    accounts,
                    wallet: json!({
                        "kind": "mock_in_memory",
                        "mock": true,
                        "lifetime": "process",
                    }),
                    metadata: json!({
                        "walletType": "secp256k1_single_key",
                        "walletSecretHash": wallet.secret_hash,
                    }),
                    keychain: None,
                    helper_report: None,
                })
            }
        }
    }

    pub(crate) fn load_and_connect_account(
        &self,
        config: &DesktopConfig,
    ) -> Result<DesktopAccount> {
        let connect_sequence = self.begin_account_connect_intent()?;
        let _connect_guard = self
            .account_connect_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey account connect lock poisoned"))?;
        self.ensure_account_connect_intent_current(connect_sequence)?;
        let account = self.load_account(config)?;
        self.remember_connected_account_for_intent(&account, connect_sequence)?;
        Ok(account)
    }

    pub(crate) fn connected_or_load_account(
        &self,
        config: &DesktopConfig,
    ) -> Result<DesktopAccount> {
        if let Some(account) = self.connected_account()? {
            return Ok(account);
        }
        self.load_and_connect_account(config)
    }

    #[cfg(test)]
    pub(crate) fn remember_connected_account(&self, account: DesktopAccount) -> Result<()> {
        let session = ConnectedAccountSession::from_account(&account)?;
        let mut guard = self
            .connected_account
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey connected account lock poisoned"))?;
        *guard = Some(session);
        Ok(())
    }

    pub(crate) fn begin_account_connect_intent(&self) -> Result<u64> {
        let mut guard = self
            .account_session_sequence
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey account session sequence lock poisoned"))?;
        *guard = guard
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("FRAMKey account session sequence overflowed"))?;
        Ok(*guard)
    }

    pub(crate) fn ensure_account_connect_intent_current(
        &self,
        connect_sequence: u64,
    ) -> Result<()> {
        let guard = self
            .account_session_sequence
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey account session sequence lock poisoned"))?;
        if *guard != connect_sequence {
            anyhow::bail!("wallet connect was superseded by a newer connect or disconnect request");
        }
        Ok(())
    }

    pub(crate) fn remember_connected_account_for_intent(
        &self,
        account: &DesktopAccount,
        connect_sequence: u64,
    ) -> Result<()> {
        let session = ConnectedAccountSession::from_account(account)?;
        let sequence_guard = self
            .account_session_sequence
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey account session sequence lock poisoned"))?;
        if *sequence_guard != connect_sequence {
            anyhow::bail!("wallet connect was superseded by a newer connect or disconnect request");
        }
        let mut account_guard = self
            .connected_account
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey connected account lock poisoned"))?;
        *account_guard = Some(session);
        Ok(())
    }

    pub(crate) fn connected_account(&self) -> Result<Option<DesktopAccount>> {
        let guard = self
            .connected_account
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey connected account lock poisoned"))?;
        Ok(guard
            .as_ref()
            .map(ConnectedAccountSession::to_minimal_account))
    }

    pub(crate) fn connected_account_address(&self) -> Result<Option<String>> {
        let guard = self
            .connected_account
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey connected account lock poisoned"))?;
        Ok(guard.as_ref().map(|session| session.address.clone()))
    }

    pub(crate) fn require_connected_account_address(&self) -> Result<String> {
        self.connected_account_address()?.ok_or_else(|| {
            anyhow::anyhow!(
                "wallet account is not connected; connect the vault before using assets"
            )
        })
    }

    pub(crate) fn disconnect_account_session(&self) -> Result<Value> {
        let account_cleared = {
            let mut sequence_guard = self
                .account_session_sequence
                .lock()
                .map_err(|_| anyhow::anyhow!("FRAMKey account session sequence lock poisoned"))?;
            *sequence_guard = sequence_guard
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("FRAMKey account session sequence overflowed"))?;
            let mut guard = self
                .connected_account
                .lock()
                .map_err(|_| anyhow::anyhow!("FRAMKey connected account lock poisoned"))?;
            guard.take().is_some()
        };
        let permissions_cleared = self.clear_account_permissions()?;
        let reviews_cleared = self.clear_review_queue()?;
        Ok(json!({
            "accountCleared": account_cleared,
            "accountPermissionsCleared": permissions_cleared,
            "reviewRequestsCleared": reviews_cleared,
        }))
    }

    pub(crate) fn transaction_wallet_address(
        &self,
        config: &DesktopConfig,
        request: &ProviderRequest,
    ) -> Result<String> {
        match config.wallet {
            DesktopWalletConfig::MockInMemory => Ok(self.mock_wallet()?.address),
            DesktopWalletConfig::KeychainVault => {
                let tx = transaction_params_object(&request.params)?;
                if let Some(from) = optional_string_field(tx, "from")? {
                    if let Some(connected) = self.connected_account_address()? {
                        validate_address_matches(&from, &connected, "transaction from")?;
                    }
                    return Ok(from);
                }
                self.require_connected_account_address()
            }
        }
    }

    pub(crate) fn reserve_transaction_nonce(
        &self,
        chain_id: &str,
        address: &str,
        rpc_pending_nonce: &str,
    ) -> Result<PendingNonceReservation> {
        let rpc_nonce = hex_quantity_to_u128(rpc_pending_nonce)
            .with_context(|| "pending transaction nonce from RPC is malformed")?;
        let address = address
            .parse::<EvmAddress>()
            .map_err(|_| anyhow::anyhow!("transaction wallet address is not a valid EVM address"))?
            .to_string();
        let key = (chain_id.to_ascii_lowercase(), address.to_ascii_lowercase());
        let mut guard = self
            .pending_nonces
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey pending nonce lock poisoned"))?;
        let reserved = guard.entry(key).or_default();
        let mut next_nonce = rpc_nonce;
        while reserved.contains(&next_nonce) {
            next_nonce = next_nonce
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("pending transaction nonce overflowed"))?;
        }
        reserved.insert(next_nonce);
        Ok(PendingNonceReservation {
            chain_id: chain_id.to_owned(),
            address,
            nonce: format!("0x{next_nonce:x}"),
            nonce_value: next_nonce,
        })
    }

    pub(crate) fn release_transaction_nonce(&self, reservation: &PendingNonceReservation) {
        let key = (
            reservation.chain_id.to_ascii_lowercase(),
            reservation.address.to_ascii_lowercase(),
        );
        let Ok(mut guard) = self.pending_nonces.lock() else {
            return;
        };
        if let Some(reserved) = guard.get_mut(&key) {
            reserved.remove(&reservation.nonce_value);
            if reserved.is_empty() {
                guard.remove(&key);
            }
        }
    }

    pub(crate) fn personal_sign_with_mock_wallet(
        &self,
        message: Vec<u8>,
        expected_address: Option<String>,
    ) -> Result<SignerPersonalSignResponse> {
        let wallet = self.mock_wallet()?;
        if let Some(expected) = expected_address
            && !wallet.address.eq_ignore_ascii_case(&expected)
        {
            anyhow::bail!(
                "personal_sign account mismatch: requested {}, mock wallet {}",
                expected,
                wallet.address
            );
        }
        let signed = personal_sign(&wallet.secret, &message)?;
        Ok(SignerPersonalSignResponse {
            keychain_service: "mock".to_owned(),
            keychain_account: "mock_in_memory".to_owned(),
            keychain_item_id: "mock_in_memory".to_owned(),
            keychain_access_policy: "mock_in_memory".to_owned(),
            device_id: "mock".to_owned(),
            kek_id: "mock".to_owned(),
            metadata: SignerVaultMetadata {
                image_size: 0,
                shard_size: 0,
                data_shards: 0,
                parity_shards: 0,
                wallet_id: "mock_in_memory".to_owned(),
                generation: 0,
                wallet_type: "secp256k1_single_key".to_owned(),
                payload_hash_valid: true,
                recovered_shard_count: 0,
                wallet_secret_hash: Some(wallet.secret_hash),
            },
            address: signed.address.to_string(),
            message_hash: signed.message_hash_hex(),
            signature: signed.signature_hex(),
        })
    }

    pub(crate) fn sign_typed_data_with_mock_wallet(
        &self,
        typed_data: Value,
        expected_address: Option<String>,
    ) -> Result<SignerSignTypedDataResponse> {
        let wallet = self.mock_wallet()?;
        if let Some(expected) = expected_address
            && !wallet.address.eq_ignore_ascii_case(&expected)
        {
            anyhow::bail!(
                "typed-data account mismatch: requested {}, mock wallet {}",
                expected,
                wallet.address
            );
        }
        let signed = sign_typed_data_v4(&wallet.secret, &typed_data)?;
        Ok(SignerSignTypedDataResponse {
            keychain_service: "mock".to_owned(),
            keychain_account: "mock_in_memory".to_owned(),
            keychain_item_id: "mock_in_memory".to_owned(),
            keychain_access_policy: "mock_in_memory".to_owned(),
            device_id: "mock".to_owned(),
            kek_id: "mock".to_owned(),
            metadata: SignerVaultMetadata {
                image_size: 0,
                shard_size: 0,
                data_shards: 0,
                parity_shards: 0,
                wallet_id: "mock_in_memory".to_owned(),
                generation: 0,
                wallet_type: "secp256k1_single_key".to_owned(),
                payload_hash_valid: true,
                recovered_shard_count: 0,
                wallet_secret_hash: Some(wallet.secret_hash),
            },
            address: signed.address.to_string(),
            typed_data_hash: signed.typed_data_hash_hex(),
            signature: signed.signature_hex(),
        })
    }

    pub(crate) fn sign_transaction_with_mock_wallet(
        &self,
        transaction: &EvmTransaction,
        expected_address: &str,
    ) -> Result<DesktopSignedTransaction> {
        let wallet = self.mock_wallet()?;
        validate_address_matches(&wallet.address, expected_address, "mock wallet address")?;
        let signed = sign_transaction(&wallet.secret, transaction)?;
        Ok(DesktopSignedTransaction::from(signed))
    }

    pub(crate) fn sign_btc_psbt_with_mock_wallet(
        &self,
        network: BtcNetwork,
        psbt_bytes: Vec<u8>,
        expected_address: String,
    ) -> Result<SignerSignBtcPsbtResponse> {
        let wallet = self.mock_wallet()?;
        let expected = match network {
            BtcNetwork::Mainnet => &wallet.btc_mainnet_address,
            BtcNetwork::Testnet4 => &wallet.btc_testnet4_address,
            BtcNetwork::Signet | BtcNetwork::Regtest => {
                anyhow::bail!("mock BTC signing supports only mainnet and Testnet4")
            }
        };
        if expected != &expected_address {
            anyhow::bail!(
                "BTC signing account mismatch: requested {}, mock wallet {}",
                expected_address,
                expected
            );
        }
        let signed = sign_p2wpkh_psbt(&wallet.secret, network, &expected_address, &psbt_bytes)?;
        Ok(SignerSignBtcPsbtResponse {
            keychain_service: "mock".to_owned(),
            keychain_account: "mock_in_memory".to_owned(),
            keychain_item_id: "mock_in_memory".to_owned(),
            keychain_access_policy: "mock_in_memory".to_owned(),
            device_id: "mock".to_owned(),
            kek_id: "mock".to_owned(),
            metadata: SignerVaultMetadata {
                image_size: 0,
                shard_size: 0,
                data_shards: 0,
                parity_shards: 0,
                wallet_id: "mock_in_memory".to_owned(),
                generation: 0,
                wallet_type: "secp256k1_single_key".to_owned(),
                payload_hash_valid: true,
                recovered_shard_count: 0,
                wallet_secret_hash: Some(wallet.secret_hash),
            },
            network: signed.network.id().to_owned(),
            address: signed.address,
            transaction_id: signed.transaction_id,
            raw_transaction: signed.raw_transaction,
            vbytes: signed.vbytes,
        })
    }

    pub(crate) fn mock_wallet(&self) -> Result<MockWalletSnapshot> {
        let mut guard = self
            .mock_wallet
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey mock wallet lock poisoned"))?;
        if guard.is_none() {
            *guard = Some(MockWallet::generate()?);
        }
        let wallet = guard.as_ref().expect("mock wallet initialized above");
        Ok(MockWalletSnapshot {
            address: wallet.address.clone(),
            btc_mainnet_address: wallet.btc_mainnet_address.clone(),
            btc_testnet4_address: wallet.btc_testnet4_address.clone(),
            secret_hash: wallet.secret_hash.clone(),
            secret: SecretBytes::new(*wallet.secret.expose()),
        })
    }

    pub(crate) fn review_queue_snapshot(&self) -> Result<Vec<ReviewRequest>> {
        let mut guard = self
            .review_queue
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey review queue lock poisoned"))?;
        Ok(guard.snapshot())
    }

    pub(crate) fn account_permission_allowed(&self, origin: &str) -> Result<bool> {
        if is_trusted_ui_origin(origin) {
            return Ok(true);
        }
        let guard = self
            .account_permissions
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey account permission lock poisoned"))?;
        Ok(guard.has(origin))
    }

    pub(crate) fn grant_account_permission(&self, origin: String) -> Result<()> {
        if is_trusted_ui_origin(&origin) {
            return Ok(());
        }
        let mut guard = self
            .account_permissions
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey account permission lock poisoned"))?;
        guard.grant(origin);
        Ok(())
    }

    pub(crate) fn revoke_account_permission(&self, origin: &str) -> Result<bool> {
        if is_trusted_ui_origin(origin) {
            return Ok(false);
        }
        let mut guard = self
            .account_permissions
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey account permission lock poisoned"))?;
        Ok(guard.revoke(origin))
    }

    pub(crate) fn clear_account_permissions(&self) -> Result<usize> {
        let mut guard = self
            .account_permissions
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey account permission lock poisoned"))?;
        Ok(guard.clear())
    }

    pub(crate) fn account_permission_snapshot(&self) -> Result<Vec<String>> {
        let guard = self
            .account_permissions
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey account permission lock poisoned"))?;
        Ok(guard.snapshot())
    }

    pub(crate) fn record_provider_request_event(
        &self,
        request: &ProviderRequest,
        envelope: &ProviderEnvelope,
        duration: Duration,
    ) -> Result<ProviderEvent> {
        let mut guard = self
            .provider_events
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey provider event log lock poisoned"))?;
        Ok(guard.push(ProviderEvent::from_provider_request(
            request, envelope, duration,
        )))
    }

    pub(crate) fn record_provider_telemetry_event(
        &self,
        window_label: &str,
        event: ProviderTelemetryEvent,
    ) -> Result<ProviderEvent> {
        let mut guard = self
            .provider_events
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey provider event log lock poisoned"))?;
        Ok(guard.push(ProviderEvent::from_telemetry(window_label, event)?))
    }

    pub(crate) fn provider_events_snapshot(&self) -> Result<Vec<ProviderEvent>> {
        let guard = self
            .provider_events
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey provider event log lock poisoned"))?;
        Ok(guard.snapshot())
    }

    pub(crate) fn clear_provider_events(&self) -> Result<usize> {
        let mut guard = self
            .provider_events
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey provider event log lock poisoned"))?;
        Ok(guard.clear())
    }

    pub(crate) fn remember_dapp_open_request(&self, target: DappSessionTarget) -> Result<()> {
        let mut guard = self
            .dapp_session
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey dApp session lock poisoned"))?;
        guard.remember_open_request(target);
        Ok(())
    }

    pub(crate) fn remember_dapp_navigation_url(&self, url: &str) -> Result<()> {
        let mut guard = self
            .dapp_session
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey dApp session lock poisoned"))?;
        guard.remember_navigation_url(dapp_session_location(url)?);
        Ok(())
    }

    pub(crate) fn remember_dapp_page_load(&self, event: &str, url: &str) -> Result<()> {
        let mut guard = self
            .dapp_session
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey dApp session lock poisoned"))?;
        guard.remember_page_load(event, dapp_session_location(url)?);
        Ok(())
    }

    pub(crate) fn remember_dapp_navigation_action(
        &self,
        action: DappNavigationAction,
    ) -> Result<()> {
        let mut guard = self
            .dapp_session
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey dApp session lock poisoned"))?;
        guard.remember_navigation_action(action);
        Ok(())
    }

    pub(crate) fn dapp_session_snapshot(&self) -> Result<Value> {
        let guard = self
            .dapp_session
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey dApp session lock poisoned"))?;
        Ok(json!(guard.clone()))
    }

    pub(crate) fn remember_watched_asset(&self, asset: WatchedAsset) -> Result<()> {
        let mut guard = self
            .watched_assets
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey watched asset lock poisoned"))?;
        guard.remember(asset);
        self.persist_wallet_ui_state_locked(&guard);
        Ok(())
    }

    pub(crate) fn watched_assets_for_chain(&self, chain_id: &str) -> Result<Vec<WatchedAsset>> {
        let guard = self
            .watched_assets
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey watched asset lock poisoned"))?;
        Ok(guard.for_chain(chain_id))
    }

    pub(crate) fn wallet_ui_state_persistence_snapshot(
        &self,
    ) -> Result<WalletUiStatePersistenceStatus> {
        let guard = self
            .wallet_ui_state_persistence
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey wallet UI persistence lock poisoned"))?;
        Ok(guard.clone())
    }

    pub(crate) fn persist_wallet_ui_state_locked(&self, watched_assets: &WatchedAssetStore) {
        let Some(path) = &self.wallet_ui_state_path else {
            return;
        };
        let result = watched_assets.write_to_path(path);
        if let Ok(mut guard) = self.wallet_ui_state_persistence.lock() {
            match result {
                Ok(()) => guard.mark_saved(),
                Err(error) => guard.warning = Some(truncate_for_event(&error.to_string(), 240)),
            }
        }
    }

    pub(crate) fn decide_review_request(
        &self,
        review_id: &str,
        decision_token: &str,
        decision: ReviewDecision,
    ) -> Result<review::ReviewDecisionOutcome> {
        let mut guard = self
            .review_queue
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey review queue lock poisoned"))?;
        let outcome = guard.decide(review_id, decision_token, decision)?;
        if outcome.review_request.kind == review::ReviewMethodKind::Transaction {
            self.record_transaction_review(&outcome.review_request)?;
        }
        self.review_condvar.notify_all();
        Ok(outcome)
    }

    pub(crate) fn wait_for_review_approval(&self, review_id: &str) -> Result<ReviewRequest> {
        let mut guard = self
            .review_queue
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey review queue lock poisoned"))?;

        loop {
            let request = guard.get(review_id)?;
            match request.status {
                ReviewStatus::Pending => {
                    let (next_guard, _) = self
                        .review_condvar
                        .wait_timeout(guard, Duration::from_millis(250))
                        .map_err(|_| anyhow::anyhow!("FRAMKey review queue lock poisoned"))?;
                    guard = next_guard;
                }
                ReviewStatus::Approved => return Ok(request),
                ReviewStatus::Rejected => anyhow::bail!("review request {review_id} was rejected"),
                ReviewStatus::Expired => {
                    anyhow::bail!("review request {review_id} expired before approval")
                }
                ReviewStatus::Signed => {
                    anyhow::bail!("review request {review_id} was already signed")
                }
                ReviewStatus::Completed => {
                    anyhow::bail!("review request {review_id} was already completed")
                }
                ReviewStatus::SignFailed => {
                    anyhow::bail!("review request {review_id} already failed")
                }
            }
        }
    }

    pub(crate) fn mark_review_signed(
        &self,
        review_id: &str,
        signed: &SignerPersonalSignResponse,
    ) -> Result<ReviewRequest> {
        self.mark_review_signature(review_id, &signed.address, &signed.message_hash)
    }

    pub(crate) fn mark_review_signature(
        &self,
        review_id: &str,
        address: &str,
        message_hash: &str,
    ) -> Result<ReviewRequest> {
        let mut guard = self
            .review_queue
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey review queue lock poisoned"))?;
        let request = guard.mark_signed(review_id, address.to_owned(), message_hash.to_owned())?;
        self.review_condvar.notify_all();
        Ok(request)
    }

    pub(crate) fn mark_review_completed(
        &self,
        review_id: &str,
        address: Option<String>,
    ) -> Result<ReviewRequest> {
        let mut guard = self
            .review_queue
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey review queue lock poisoned"))?;
        let request = guard.mark_completed(review_id, address)?;
        self.review_condvar.notify_all();
        Ok(request)
    }

    pub(crate) fn mark_review_transaction_broadcast(
        &self,
        review_id: &str,
        address: &str,
        tx_hash: &str,
        local_tx_hash: &str,
    ) -> Result<ReviewRequest> {
        let mut guard = self
            .review_queue
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey review queue lock poisoned"))?;
        let request = guard.mark_signed(review_id, address.to_owned(), tx_hash.to_owned())?;
        self.record_transaction_broadcast(&request, address, tx_hash, local_tx_hash)?;
        self.review_condvar.notify_all();
        Ok(request)
    }

    pub(crate) fn mark_review_btc_broadcast(
        &self,
        review_id: &str,
        address: &str,
        txid: &str,
    ) -> Result<ReviewRequest> {
        let mut guard = self
            .review_queue
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey review queue lock poisoned"))?;
        let request = guard.mark_signed(review_id, address.to_owned(), txid.to_owned())?;
        self.record_transaction_broadcast(&request, address, txid, txid)?;
        self.review_condvar.notify_all();
        Ok(request)
    }

    pub(crate) fn mark_review_sign_failed(
        &self,
        review_id: &str,
        error: &str,
    ) -> Result<ReviewRequest> {
        let mut guard = self
            .review_queue
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey review queue lock poisoned"))?;
        let request = guard.mark_sign_failed(review_id, error.to_owned())?;
        if matches!(
            request.kind,
            review::ReviewMethodKind::Transaction | review::ReviewMethodKind::BtcTransaction
        ) {
            self.record_transaction_failure(&request, error)?;
        }
        self.review_condvar.notify_all();
        Ok(request)
    }

    pub(crate) fn dismiss_review_request(&self, review_id: &str) -> Result<bool> {
        let mut guard = self
            .review_queue
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey review queue lock poisoned"))?;
        let dismissed = guard.dismiss(review_id);
        self.review_condvar.notify_all();
        Ok(dismissed)
    }

    pub(crate) fn clear_review_queue(&self) -> Result<usize> {
        let mut guard = self
            .review_queue
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey review queue lock poisoned"))?;
        let cleared = guard.clear();
        self.review_condvar.notify_all();
        Ok(cleared)
    }

    pub(crate) fn transaction_activity_snapshot(&self) -> Result<Vec<TransactionActivityEntry>> {
        let guard = self
            .transaction_activity
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey transaction activity lock poisoned"))?;
        Ok(guard.snapshot())
    }

    pub(crate) fn transaction_activity_persistence_snapshot(
        &self,
    ) -> Result<TransactionActivityPersistenceStatus> {
        let guard = self.transaction_activity_persistence.lock().map_err(|_| {
            anyhow::anyhow!("FRAMKey transaction activity persistence lock poisoned")
        })?;
        Ok(guard.clone())
    }

    pub(crate) fn refresh_transaction_receipts(&self, config: &DesktopConfig) -> Result<Value> {
        let hashes = {
            let guard = self
                .transaction_activity
                .lock()
                .map_err(|_| anyhow::anyhow!("FRAMKey transaction activity lock poisoned"))?;
            guard.receipt_refresh_hashes(TRANSACTION_RECEIPT_REFRESH_LIMIT)
        };

        let mut refreshed = 0_usize;
        let mut pending = 0_usize;
        let mut errors = Vec::new();
        for hash in hashes {
            match rpc_result(config, "eth_getTransactionReceipt", json!([hash])) {
                Ok(receipt) => match transaction_receipt_summary(&receipt) {
                    Ok(summary) => {
                        if summary.is_some() {
                            refreshed += 1;
                        } else {
                            pending += 1;
                        }
                        let mut guard = self.transaction_activity.lock().map_err(|_| {
                            anyhow::anyhow!("FRAMKey transaction activity lock poisoned")
                        })?;
                        guard.record_receipt(&hash, summary);
                        self.persist_transaction_activity_locked(&guard);
                    }
                    Err(error) => errors.push(json!({
                        "transactionHash": hash,
                        "message": error.to_string(),
                    })),
                },
                Err(error) => errors.push(json!({
                    "transactionHash": hash,
                    "message": error.to_string(),
                })),
            }
        }

        Ok(json!({
            "queried": refreshed + pending + errors.len(),
            "included": refreshed,
            "pending": pending,
            "errors": errors,
            "limit": TRANSACTION_RECEIPT_REFRESH_LIMIT,
        }))
    }

    pub(crate) fn record_transaction_review(&self, request: &ReviewRequest) -> Result<()> {
        let mut guard = self
            .transaction_activity
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey transaction activity lock poisoned"))?;
        guard.record_review(request);
        self.persist_transaction_activity_locked(&guard);
        Ok(())
    }

    pub(crate) fn record_transaction_broadcast(
        &self,
        request: &ReviewRequest,
        address: &str,
        tx_hash: &str,
        local_tx_hash: &str,
    ) -> Result<()> {
        let mut guard = self
            .transaction_activity
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey transaction activity lock poisoned"))?;
        guard.record_broadcast(request, address, tx_hash, local_tx_hash);
        self.persist_transaction_activity_locked(&guard);
        Ok(())
    }

    pub(crate) fn record_transaction_failure(
        &self,
        request: &ReviewRequest,
        error: &str,
    ) -> Result<()> {
        let mut guard = self
            .transaction_activity
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey transaction activity lock poisoned"))?;
        guard.record_failure(request, error);
        self.persist_transaction_activity_locked(&guard);
        Ok(())
    }

    pub(crate) fn persist_transaction_activity_locked(&self, activity: &TransactionActivityLog) {
        let Some(path) = &self.transaction_activity_persistence_path else {
            return;
        };
        let result = activity.write_to_path(path);
        if let Ok(mut guard) = self.transaction_activity_persistence.lock() {
            match result {
                Ok(()) => guard.mark_saved(),
                Err(error) => guard.warning = Some(truncate_for_event(&error.to_string(), 240)),
            }
        }
    }

    pub(crate) fn recovery_ui_state_snapshot(&self) -> Result<Value> {
        let state = self
            .recovery_ui_state
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey recovery UI state lock poisoned"))?
            .clone();
        let persistence = self
            .recovery_ui_state_persistence
            .lock()
            .map_err(|_| anyhow::anyhow!("FRAMKey recovery UI persistence lock poisoned"))?
            .clone();
        Ok(recovery_ui_state_payload(&state, &persistence))
    }

    pub(crate) fn clear_recovery_ui_state(&self) -> Result<Value> {
        {
            let mut state = self
                .recovery_ui_state
                .lock()
                .map_err(|_| anyhow::anyhow!("FRAMKey recovery UI state lock poisoned"))?;
            state.clear();
            self.persist_recovery_ui_state_locked(&state);
        }
        self.recovery_ui_state_snapshot()
    }

    pub(crate) fn remember_recovery_outcome(&self, outcome: &Value) {
        let Ok(mut state) = self.recovery_ui_state.lock() else {
            return;
        };
        if state.remember(outcome) {
            self.persist_recovery_ui_state_locked(&state);
        }
    }

    pub(crate) fn persist_recovery_ui_state_locked(&self, state: &RecoveryUiState) {
        let Some(path) = &self.recovery_ui_state_path else {
            return;
        };
        let result = state.write_to_path(path);
        if let Ok(mut guard) = self.recovery_ui_state_persistence.lock() {
            match result {
                Ok(()) => guard.mark_saved(),
                Err(error) => guard.warning = Some(truncate_for_event(&error.to_string(), 240)),
            }
        }
    }
}

pub(crate) struct MockWallet {
    pub(crate) secret: SecretBytes<32>,
    pub(crate) address: String,
    pub(crate) btc_mainnet_address: String,
    pub(crate) btc_testnet4_address: String,
    pub(crate) secret_hash: String,
}

impl MockWallet {
    pub(crate) fn generate() -> Result<Self> {
        for _ in 0..16 {
            let secret = SecretBytes::new(random_array::<32>()?);
            let Ok(address) = address_from_secret(&secret) else {
                continue;
            };
            let Ok(btc_account) = p2wpkh_account_from_secret(&secret, BtcNetwork::Mainnet) else {
                continue;
            };
            let Ok(btc_testnet_account) = p2wpkh_account_from_secret(&secret, BtcNetwork::Testnet4)
            else {
                continue;
            };
            let secret_hash = encode_hex(blake3::hash(secret.expose()).as_bytes());
            return Ok(Self {
                secret,
                address: address.to_string(),
                btc_mainnet_address: btc_account.address,
                btc_testnet4_address: btc_testnet_account.address,
                secret_hash,
            });
        }
        anyhow::bail!("failed to generate valid mock secp256k1 wallet")
    }
}

pub(crate) struct AccountPermissionStore {
    pub(crate) origins: BTreeSet<String>,
}

impl AccountPermissionStore {
    pub(crate) fn new() -> Self {
        Self {
            origins: BTreeSet::new(),
        }
    }

    pub(crate) fn has(&self, origin: &str) -> bool {
        self.origins.contains(origin)
    }

    pub(crate) fn grant(&mut self, origin: String) {
        self.origins.insert(origin);
    }

    pub(crate) fn revoke(&mut self, origin: &str) -> bool {
        self.origins.remove(origin)
    }

    pub(crate) fn clear(&mut self) -> usize {
        let cleared = self.origins.len();
        self.origins.clear();
        cleared
    }

    pub(crate) fn snapshot(&self) -> Vec<String> {
        self.origins.iter().cloned().collect()
    }
}

#[derive(Debug)]
pub(crate) struct ProviderEventLog {
    pub(crate) next_sequence: u64,
    pub(crate) events: VecDeque<ProviderEvent>,
}

impl ProviderEventLog {
    pub(crate) fn new() -> Self {
        Self {
            next_sequence: 1,
            events: VecDeque::new(),
        }
    }

    pub(crate) fn push(&mut self, mut event: ProviderEvent) -> ProviderEvent {
        event.sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        let recorded = event.clone();
        self.events.push_back(event);
        while self.events.len() > PROVIDER_EVENT_LOG_LIMIT {
            self.events.pop_front();
        }
        recorded
    }

    pub(crate) fn snapshot(&self) -> Vec<ProviderEvent> {
        self.events.iter().cloned().collect()
    }

    pub(crate) fn clear(&mut self) -> usize {
        let cleared = self.events.len();
        self.events.clear();
        cleared
    }
}

#[derive(Debug)]
pub(crate) struct TransactionActivityLog {
    pub(crate) items: VecDeque<TransactionActivityEntry>,
}

impl TransactionActivityLog {
    pub(crate) fn new() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    pub(crate) fn read_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read transaction activity {}", path.display()))?;
        let persisted: PersistedTransactionActivityLog = serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse transaction activity {}", path.display()))?;
        if persisted.version != TRANSACTION_ACTIVITY_PERSISTENCE_VERSION {
            anyhow::bail!(
                "unsupported transaction activity version {}",
                persisted.version
            );
        }
        Ok(Self::from_entries(persisted.items))
    }

    pub(crate) fn from_entries(items: Vec<TransactionActivityEntry>) -> Self {
        Self {
            items: items
                .into_iter()
                .take(TRANSACTION_ACTIVITY_LIMIT)
                .map(normalize_restored_transaction_activity)
                .collect(),
        }
    }

    pub(crate) fn write_to_path(&self, path: &Path) -> Result<()> {
        let persisted = PersistedTransactionActivityLog {
            version: TRANSACTION_ACTIVITY_PERSISTENCE_VERSION,
            items: self.snapshot(),
        };
        let bytes = serde_json::to_vec_pretty(&persisted)
            .context("failed to serialize transaction activity")?;
        write_json_atomically(path, &bytes)
    }

    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }

    pub(crate) fn record_review(&mut self, request: &ReviewRequest) {
        let now = now_unix_ms();
        let status = transaction_activity_status(request);
        if let Some(item) = self
            .items
            .iter_mut()
            .find(|item| item.review_id == request.id)
        {
            item.status = status.to_owned();
            item.policy_decision = activity_summary_string(&request.summary, "policy", "decision");
            item.simulation_status =
                activity_summary_string(&request.summary, "simulation", "status");
            item.guidance = transaction_activity_guidance(request, status, None);
            item.updated_at_unix_ms = now;
            return;
        }

        let is_btc = request.kind == review::ReviewMethodKind::BtcTransaction;
        let entry = TransactionActivityEntry {
            id: request.id.clone(),
            review_id: request.id.clone(),
            provider_request_id: request.provider_request_id.clone(),
            method: request.method.clone(),
            origin: request.origin.clone(),
            chain_id: if is_btc {
                activity_summary_string(&request.summary, "network", "")
            } else {
                activity_summary_string(&request.summary, "chainId", "")
            },
            from: if is_btc {
                activity_summary_string(&request.summary, "fromAddress", "")
            } else {
                activity_summary_string(&request.summary, "from", "")
            },
            to: if is_btc {
                activity_summary_string(&request.summary, "toAddress", "")
            } else {
                activity_summary_string(&request.summary, "to", "")
            },
            value: if is_btc {
                activity_summary_string(&request.summary, "amountSat", "")
            } else {
                activity_summary_string(&request.summary, "value", "")
            },
            data_bytes: if is_btc {
                None
            } else {
                request.summary.get("dataBytes").and_then(Value::as_u64)
            },
            call: if is_btc {
                Some("btc_p2wpkh_transfer".to_owned())
            } else {
                activity_transaction_call_label(&request.summary)
            },
            policy_decision: activity_summary_string(&request.summary, "policy", "decision")
                .or_else(|| activity_summary_string(&request.summary, "decision", "")),
            simulation_status: if is_btc {
                activity_summary_string(&request.summary, "simulation", "")
            } else {
                activity_summary_string(&request.summary, "simulation", "status")
            },
            guidance: transaction_activity_guidance(request, status, None),
            status: status.to_owned(),
            address: request
                .execution
                .as_ref()
                .and_then(|execution| execution.address.clone()),
            transaction_hash: request
                .execution
                .as_ref()
                .and_then(|execution| execution.message_hash.clone()),
            local_transaction_hash: None,
            error: request
                .execution
                .as_ref()
                .and_then(|execution| execution.error.clone()),
            receipt_status: None,
            receipt: None,
            receipt_checked_at_unix_ms: None,
            created_at_unix_ms: request.received_at_unix_ms,
            updated_at_unix_ms: now,
        };
        self.items.push_front(entry);
        while self.items.len() > TRANSACTION_ACTIVITY_LIMIT {
            self.items.pop_back();
        }
    }

    pub(crate) fn record_broadcast(
        &mut self,
        request: &ReviewRequest,
        address: &str,
        tx_hash: &str,
        local_tx_hash: &str,
    ) {
        self.record_review(request);
        if let Some(item) = self
            .items
            .iter_mut()
            .find(|item| item.review_id == request.id)
        {
            item.status = "broadcast".to_owned();
            item.address = Some(address.to_owned());
            item.transaction_hash = Some(tx_hash.to_owned());
            item.local_transaction_hash = Some(local_tx_hash.to_owned());
            item.error = None;
            if request.kind == review::ReviewMethodKind::BtcTransaction {
                item.receipt_status = None;
                item.guidance = btc_transaction_activity_lifecycle_guidance("broadcast");
            } else {
                item.receipt_status = Some("pending".to_owned());
                item.guidance = transaction_activity_lifecycle_guidance("broadcast");
            }
            item.updated_at_unix_ms = now_unix_ms();
        }
    }

    pub(crate) fn record_failure(&mut self, request: &ReviewRequest, error: &str) {
        self.record_review(request);
        if let Some(item) = self
            .items
            .iter_mut()
            .find(|item| item.review_id == request.id)
        {
            item.status = "failed".to_owned();
            item.error = Some(truncate_for_event(error, 240));
            item.guidance = Some(transaction_activity_failure_guidance(error));
            item.updated_at_unix_ms = now_unix_ms();
        }
    }

    pub(crate) fn record_receipt(
        &mut self,
        tx_hash: &str,
        receipt: Option<TransactionReceiptSummary>,
    ) {
        let now = now_unix_ms();
        if let Some(item) = self
            .items
            .iter_mut()
            .find(|item| item.transaction_hash.as_deref() == Some(tx_hash))
        {
            match receipt {
                Some(receipt) => {
                    item.status = receipt.status.clone();
                    item.receipt_status = Some(receipt.status.clone());
                    item.receipt = Some(receipt);
                    item.guidance = transaction_activity_lifecycle_guidance(&item.status);
                }
                None => {
                    item.receipt_status = Some("pending".to_owned());
                    item.guidance = transaction_activity_lifecycle_guidance(&item.status);
                }
            }
            item.receipt_checked_at_unix_ms = Some(now);
            item.updated_at_unix_ms = now;
        }
    }

    pub(crate) fn receipt_refresh_hashes(&self, limit: usize) -> Vec<String> {
        self.items
            .iter()
            .filter(|item| item.transaction_hash.is_some())
            .filter(|item| item.method == "eth_sendTransaction")
            .filter(|item| !matches!(item.status.as_str(), "confirmed" | "reverted"))
            .filter_map(|item| item.transaction_hash.clone())
            .take(limit)
            .collect()
    }

    pub(crate) fn snapshot(&self) -> Vec<TransactionActivityEntry> {
        self.items.iter().cloned().collect()
    }
}

pub(crate) const TRANSACTION_ACTIVITY_PERSISTENCE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PersistedTransactionActivityLog {
    pub(crate) version: u32,
    pub(crate) items: Vec<TransactionActivityEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransactionActivityPersistenceStatus {
    pub(crate) enabled: bool,
    pub(crate) restored: bool,
    pub(crate) items_restored: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) last_saved_at_unix_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) warning: Option<String>,
}

impl TransactionActivityPersistenceStatus {
    pub(crate) fn disabled() -> Self {
        Self {
            enabled: false,
            restored: false,
            items_restored: 0,
            last_saved_at_unix_ms: None,
            warning: None,
        }
    }

    pub(crate) fn unavailable(warning: String) -> Self {
        Self {
            enabled: false,
            restored: false,
            items_restored: 0,
            last_saved_at_unix_ms: None,
            warning: Some(warning),
        }
    }

    pub(crate) fn enabled() -> Self {
        Self {
            enabled: true,
            restored: false,
            items_restored: 0,
            last_saved_at_unix_ms: None,
            warning: None,
        }
    }

    pub(crate) fn mark_saved(&mut self) {
        self.enabled = true;
        self.last_saved_at_unix_ms = Some(now_unix_ms());
        self.warning = None;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecoveryUiState {
    pub(crate) version: u32,
    pub(crate) updated_at_unix_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) backup_outcome: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) drill_outcome: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) recover_outcome: Option<Value>,
}

impl RecoveryUiState {
    pub(crate) fn new() -> Self {
        Self {
            version: RECOVERY_UI_STATE_VERSION,
            updated_at_unix_ms: now_unix_ms(),
            backup_outcome: None,
            drill_outcome: None,
            recover_outcome: None,
        }
    }

    pub(crate) fn read_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read recovery UI state {}", path.display()))?;
        let persisted: RecoveryUiState = serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse recovery UI state {}", path.display()))?;
        if persisted.version != RECOVERY_UI_STATE_VERSION {
            anyhow::bail!(
                "unsupported recovery UI state version {}",
                persisted.version
            );
        }
        Ok(persisted.sanitized())
    }

    pub(crate) fn write_to_path(&self, path: &Path) -> Result<()> {
        let bytes =
            serde_json::to_vec_pretty(self).context("failed to serialize recovery UI state")?;
        write_json_atomically(path, &bytes)
    }

    pub(crate) fn has_outcomes(&self) -> bool {
        self.backup_outcome.is_some()
            || self.drill_outcome.is_some()
            || self.recover_outcome.is_some()
    }

    pub(crate) fn clear(&mut self) {
        self.updated_at_unix_ms = now_unix_ms();
        self.backup_outcome = None;
        self.drill_outcome = None;
        self.recover_outcome = None;
    }

    pub(crate) fn remember(&mut self, outcome: &Value) -> bool {
        let Some(sanitized) = sanitize_recovery_ui_outcome(outcome) else {
            return false;
        };
        let Some(operation) = sanitized.get("operation").and_then(Value::as_str) else {
            return false;
        };
        match operation {
            "create_keychain_vault" => {
                self.backup_outcome = Some(sanitized);
                self.drill_outcome = None;
                self.recover_outcome = None;
            }
            "recovery_smoke_pack" => {
                self.drill_outcome = sanitized.get("recommendedDrill").cloned();
                self.backup_outcome = Some(sanitized);
                self.recover_outcome = None;
            }
            "validate_recovery_set" => {
                self.drill_outcome = Some(sanitized);
            }
            "recover_keychain_vault" => {
                self.recover_outcome = Some(sanitized);
            }
            _ => return false,
        }
        self.updated_at_unix_ms = now_unix_ms();
        true
    }

    pub(crate) fn sanitized(self) -> Self {
        let mut state = Self {
            version: RECOVERY_UI_STATE_VERSION,
            updated_at_unix_ms: self.updated_at_unix_ms,
            backup_outcome: None,
            drill_outcome: None,
            recover_outcome: None,
        };
        if let Some(outcome) = self.backup_outcome {
            state.remember(&outcome);
        }
        if let Some(outcome) = self.drill_outcome {
            state.remember(&outcome);
        }
        if let Some(outcome) = self.recover_outcome {
            state.remember(&outcome);
        }
        state.updated_at_unix_ms = self.updated_at_unix_ms;
        state
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecoveryUiStatePersistenceStatus {
    pub(crate) enabled: bool,
    pub(crate) restored: bool,
    pub(crate) outcomes_restored: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) last_saved_at_unix_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) warning: Option<String>,
}

impl RecoveryUiStatePersistenceStatus {
    pub(crate) fn unavailable(warning: String) -> Self {
        Self {
            enabled: false,
            restored: false,
            outcomes_restored: 0,
            last_saved_at_unix_ms: None,
            warning: Some(warning),
        }
    }

    pub(crate) fn enabled() -> Self {
        Self {
            enabled: true,
            restored: false,
            outcomes_restored: 0,
            last_saved_at_unix_ms: None,
            warning: None,
        }
    }

    pub(crate) fn mark_saved(&mut self) {
        self.enabled = true;
        self.restored = false;
        self.last_saved_at_unix_ms = Some(now_unix_ms());
        self.warning = None;
    }
}

pub(crate) fn load_recovery_ui_state_persistence() -> (
    Option<PathBuf>,
    RecoveryUiState,
    RecoveryUiStatePersistenceStatus,
) {
    let path = match recovery_ui_state_path() {
        Ok(path) => path,
        Err(error) => {
            return (
                None,
                RecoveryUiState::new(),
                RecoveryUiStatePersistenceStatus::unavailable(truncate_for_event(
                    &error.to_string(),
                    240,
                )),
            );
        }
    };
    let mut persistence = RecoveryUiStatePersistenceStatus::enabled();
    let state = match RecoveryUiState::read_from_path(&path) {
        Ok(state) => {
            persistence.restored = state.has_outcomes();
            persistence.outcomes_restored = [
                state.backup_outcome.is_some(),
                state.drill_outcome.is_some(),
                state.recover_outcome.is_some(),
            ]
            .into_iter()
            .filter(|restored| *restored)
            .count();
            state
        }
        Err(error) => {
            persistence.warning = Some(truncate_for_event(&error.to_string(), 240));
            RecoveryUiState::new()
        }
    };
    (Some(path), state, persistence)
}

pub(crate) fn recovery_ui_state_payload(
    state: &RecoveryUiState,
    persistence: &RecoveryUiStatePersistenceStatus,
) -> Value {
    json!({
        "operation": "recovery_state",
        "updatedAtUnixMs": state.updated_at_unix_ms,
        "backupOutcome": state.backup_outcome,
        "drillOutcome": state.drill_outcome,
        "recoverOutcome": state.recover_outcome,
        "persistence": persistence,
    })
}

pub(crate) fn sanitize_recovery_ui_outcome(outcome: &Value) -> Option<Value> {
    match outcome.get("operation")?.as_str()? {
        "create_keychain_vault" | "recovery_smoke_pack" => {
            sanitize_recovery_backup_outcome(outcome)
        }
        "validate_recovery_set" => sanitize_recovery_drill_outcome(outcome),
        "recover_keychain_vault" => sanitize_recovery_recover_outcome(outcome),
        _ => None,
    }
}

pub(crate) fn sanitize_recovery_backup_outcome(outcome: &Value) -> Option<Value> {
    let mut object = Map::new();
    copy_json_string(outcome, &mut object, "operation", 80);
    copy_json_bool(outcome, &mut object, "developmentOnly");
    copy_json_string(outcome, &mut object, "outDir", 4096);
    copy_json_u64(outcome, &mut object, "generation");
    copy_json_u64(outcome, &mut object, "saveSize");
    copy_json_string(outcome, &mut object, "saveImageBlake3", 128);
    copy_json_bool(outcome, &mut object, "keychainTouched");
    copy_json_bool(outcome, &mut object, "configuredVaultDeviceTouched");
    copy_json_bool(outcome, &mut object, "walletSecretTouched");
    copy_json_bool(outcome, &mut object, "walletSecretPrinted");
    copy_json_bool(outcome, &mut object, "recoveryRootKeyPrinted");
    copy_json_bool(outcome, &mut object, "recoveryShareBytesPrinted");
    copy_json_string(outcome, &mut object, "plaintextSecretProcess", 120);
    object.insert(
        "recoveryBackups".to_owned(),
        sanitize_recovery_backups(outcome.get("recoveryBackups")?)?,
    );
    if let Some(drill) = outcome
        .get("cloudOnlyDrill")
        .and_then(sanitize_recovery_drill_outcome)
    {
        object.insert("cloudOnlyDrill".to_owned(), drill);
    }
    if let Some(drill) = outcome
        .get("recommendedDrill")
        .and_then(sanitize_recovery_drill_outcome)
    {
        object.insert("recommendedDrill".to_owned(), drill);
    }
    if let Some(keychain) = outcome.get("keychain").and_then(sanitize_recovery_keychain) {
        object.insert("keychain".to_owned(), keychain);
    }
    Some(Value::Object(object))
}

pub(crate) fn sanitize_recovery_backups(backups: &Value) -> Option<Value> {
    let mut object = Map::new();
    copy_json_string(backups, &mut object, "outDir", 4096);
    copy_json_string(backups, &mut object, "backupSetId", 128);
    copy_json_string(backups, &mut object, "policyId", 128);
    copy_json_string(backups, &mut object, "walletId", 128);
    copy_json_u64(backups, &mut object, "generation");
    copy_json_u64(backups, &mut object, "shareFileCount");
    copy_json_u64(backups, &mut object, "backupFileCount");
    copy_json_u64(backups, &mut object, "bundleFileCount");
    copy_json_u64(backups, &mut object, "embeddedVaultBackupCount");
    copy_json_bool(backups, &mut object, "cloudAloneRecovers");
    let files = backups
        .get("files")
        .and_then(Value::as_array)?
        .iter()
        .take(16)
        .filter_map(sanitize_recovery_file_summary)
        .collect::<Vec<_>>();
    object.insert("files".to_owned(), json!(files));
    Some(Value::Object(object))
}

pub(crate) fn sanitize_recovery_file_summary(file: &Value) -> Option<Value> {
    let mut object = Map::new();
    copy_json_string(file, &mut object, "kind", 32);
    copy_json_string(file, &mut object, "path", 4096);
    copy_json_string(file, &mut object, "blake3", 128);
    copy_json_string(file, &mut object, "destination", 240);
    copy_json_string(file, &mut object, "group", 80);
    copy_json_string(file, &mut object, "member", 120);
    copy_json_string(file, &mut object, "encryptedVaultData", 80);
    copy_json_bool(file, &mut object, "shareBytesPrinted");
    copy_json_bool(file, &mut object, "containsSecretBytes");
    if object.contains_key("kind") && object.contains_key("path") {
        Some(Value::Object(object))
    } else {
        None
    }
}

pub(crate) fn sanitize_recovery_drill_outcome(outcome: &Value) -> Option<Value> {
    let mut object = Map::new();
    copy_json_string(outcome, &mut object, "operation", 80);
    copy_json_string(outcome, &mut object, "backupSetId", 128);
    copy_json_string(outcome, &mut object, "walletId", 128);
    copy_json_u64(outcome, &mut object, "generation");
    copy_json_string(outcome, &mut object, "policyId", 128);
    copy_json_u64(outcome, &mut object, "recoveryShareFileCount");
    copy_json_bool(outcome, &mut object, "canRecover");
    copy_json_string(outcome, &mut object, "failureReason", 240);
    copy_json_bool(outcome, &mut object, "walletSecretTouched");
    copy_json_bool(outcome, &mut object, "recoveryRootKeyPrinted");
    copy_json_bool(outcome, &mut object, "recoveryShareBytesPrinted");
    copy_json_bool(outcome, &mut object, "configuredVaultDeviceTouched");
    copy_json_string(outcome, &mut object, "plaintextSecretProcess", 120);
    copy_json_string_array(outcome, &mut object, "recoveryFiles", 16, 4096);
    copy_json_string_array(outcome, &mut object, "satisfiedGroups", 8, 80);
    if object.contains_key("operation") && object.contains_key("canRecover") {
        Some(Value::Object(object))
    } else {
        None
    }
}

pub(crate) fn sanitize_recovery_recover_outcome(outcome: &Value) -> Option<Value> {
    let mut object = Map::new();
    copy_json_string(outcome, &mut object, "operation", 80);
    copy_json_string(outcome, &mut object, "vaultBackupPath", 4096);
    copy_json_string(outcome, &mut object, "vaultBackupBlake3", 128);
    copy_json_u64(outcome, &mut object, "saveSize");
    copy_json_string(outcome, &mut object, "saveImageBlake3", 128);
    copy_json_string_array(outcome, &mut object, "recoveryFiles", 16, 4096);
    copy_json_u64(outcome, &mut object, "recoveryShareFileCount");
    copy_json_bool(outcome, &mut object, "walletSecretTouched");
    copy_json_bool(outcome, &mut object, "recoveryShareBytesPrinted");
    copy_json_string(outcome, &mut object, "plaintextSecretProcess", 120);
    if let Some(keychain) = outcome.get("keychain").and_then(sanitize_recovery_keychain) {
        object.insert("keychain".to_owned(), keychain);
    }
    if object.contains_key("operation") {
        Some(Value::Object(object))
    } else {
        None
    }
}

pub(crate) fn sanitize_recovery_keychain(keychain: &Value) -> Option<Value> {
    let mut object = Map::new();
    copy_json_string(keychain, &mut object, "service", 240);
    copy_json_string(keychain, &mut object, "account", 240);
    copy_json_string(keychain, &mut object, "accessPolicy", 240);
    copy_json_bool(keychain, &mut object, "createdKeychainKek");
    if object.is_empty() {
        None
    } else {
        Some(Value::Object(object))
    }
}

pub(crate) fn copy_json_string(
    source: &Value,
    target: &mut Map<String, Value>,
    key: &str,
    max_len: usize,
) {
    if let Some(value) = source.get(key).and_then(Value::as_str) {
        target.insert(key.to_owned(), json!(truncate_for_event(value, max_len)));
    }
}

pub(crate) fn copy_json_string_array(
    source: &Value,
    target: &mut Map<String, Value>,
    key: &str,
    max_items: usize,
    max_len: usize,
) {
    let Some(values) = source.get(key).and_then(Value::as_array) else {
        return;
    };
    let strings = values
        .iter()
        .take(max_items)
        .filter_map(Value::as_str)
        .map(|value| truncate_for_event(value, max_len))
        .collect::<Vec<_>>();
    target.insert(key.to_owned(), json!(strings));
}

pub(crate) fn copy_json_bool(source: &Value, target: &mut Map<String, Value>, key: &str) {
    if let Some(value) = source.get(key).and_then(Value::as_bool) {
        target.insert(key.to_owned(), json!(value));
    }
}

pub(crate) fn copy_json_u64(source: &Value, target: &mut Map<String, Value>, key: &str) {
    if let Some(value) = source.get(key).and_then(Value::as_u64) {
        target.insert(key.to_owned(), json!(value));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransactionActivityEntry {
    pub(crate) id: String,
    pub(crate) review_id: String,
    pub(crate) provider_request_id: String,
    pub(crate) method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) chain_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) call: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) policy_decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) simulation_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) guidance: Option<TransactionActivityGuidance>,
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) transaction_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) local_transaction_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) receipt_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) receipt: Option<TransactionReceiptSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) receipt_checked_at_unix_ms: Option<u64>,
    pub(crate) created_at_unix_ms: u64,
    pub(crate) updated_at_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransactionActivityGuidance {
    pub(crate) status: String,
    pub(crate) tone: String,
    pub(crate) title: String,
    pub(crate) message: String,
    pub(crate) primary_action: String,
    pub(crate) next_step: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reason_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransactionReceiptSummary {
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) block_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) transaction_index: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) gas_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) effective_gas_price: Option<String>,
}

pub(crate) fn transaction_activity_status(request: &ReviewRequest) -> &'static str {
    match request.status {
        ReviewStatus::Pending => "review_pending",
        ReviewStatus::Approved => "approved",
        ReviewStatus::Rejected => "rejected",
        ReviewStatus::Expired => "expired",
        ReviewStatus::Completed => "completed",
        ReviewStatus::Signed => "broadcast",
        ReviewStatus::SignFailed => "failed",
    }
}

pub(crate) fn normalize_restored_transaction_activity(
    mut entry: TransactionActivityEntry,
) -> TransactionActivityEntry {
    if matches!(entry.status.as_str(), "review_pending" | "approved") {
        entry.status = "expired".to_owned();
        entry.receipt_status = None;
        entry.guidance = Some(TransactionActivityGuidance {
            status: "expired".to_owned(),
            tone: "warn".to_owned(),
            title: "Request not active after restart".to_owned(),
            message:
                "FRAMKey restored this activity, but pending trusted approvals are not persisted."
                    .to_owned(),
            primary_action: "Retry from dApp".to_owned(),
            next_step: "Start the action again from the dApp so FRAMKey can build a fresh review."
                .to_owned(),
            reason_code: Some("review_not_restored".to_owned()),
        });
    }
    entry
}

pub(crate) fn activity_summary_string(
    summary: &Value,
    first: &str,
    second: &str,
) -> Option<String> {
    let value = if second.is_empty() {
        summary.get(first)
    } else {
        summary.get(first).and_then(|value| value.get(second))
    }?;
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

pub(crate) fn activity_transaction_call_label(summary: &Value) -> Option<String> {
    activity_summary_string(summary, "simulation", "decodedCall")
        .or_else(|| {
            summary
                .get("simulation")
                .and_then(|simulation| simulation.get("decodedCall"))
                .and_then(|decoded| decoded.get("function"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .or_else(|| {
            summary
                .get("simulation")
                .and_then(|simulation| simulation.get("transaction"))
                .and_then(|transaction| transaction.get("selector"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
}

pub(crate) fn transaction_activity_guidance(
    request: &ReviewRequest,
    status: &str,
    error: Option<&str>,
) -> Option<TransactionActivityGuidance> {
    if status == "failed" {
        return Some(transaction_activity_failure_guidance(
            error.unwrap_or("transaction failed"),
        ));
    }
    if status == "review_pending" {
        return transaction_activity_guidance_from_summary(&request.summary)
            .or_else(|| transaction_activity_lifecycle_guidance(status));
    }
    transaction_activity_lifecycle_guidance(status)
        .or_else(|| transaction_activity_guidance_from_summary(&request.summary))
}

pub(crate) fn transaction_activity_guidance_from_summary(
    summary: &Value,
) -> Option<TransactionActivityGuidance> {
    let guidance = summary.get("guidance")?;
    Some(TransactionActivityGuidance {
        status: activity_guidance_string(guidance, "status", "review").to_owned(),
        tone: activity_guidance_string(guidance, "tone", "warn").to_owned(),
        title: activity_guidance_string(guidance, "title", "Review transaction").to_owned(),
        message: activity_guidance_string(
            guidance,
            "message",
            "Review the transaction before deciding.",
        )
        .to_owned(),
        primary_action: activity_guidance_string(guidance, "primaryAction", "Review").to_owned(),
        next_step: activity_guidance_string(
            guidance,
            "nextStep",
            "Review the transaction details before deciding.",
        )
        .to_owned(),
        reason_code: guidance
            .get("reasonCode")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

pub(crate) fn activity_guidance_string<'a>(
    guidance: &'a Value,
    key: &str,
    fallback: &'a str,
) -> &'a str {
    guidance
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(fallback)
}

pub(crate) fn transaction_activity_lifecycle_guidance(
    status: &str,
) -> Option<TransactionActivityGuidance> {
    let guidance = match status {
        "approved" => TransactionActivityGuidance {
            status: "approved".to_owned(),
            tone: "warn".to_owned(),
            title: "Approved, not broadcast yet".to_owned(),
            message: "The trusted UI approved this transaction and signing or broadcast is still in progress.".to_owned(),
            primary_action: "Wait".to_owned(),
            next_step: "Keep the app open until the transaction is signed and broadcast or a failure appears.".to_owned(),
            reason_code: Some("awaiting_broadcast".to_owned()),
        },
        "broadcast" => TransactionActivityGuidance {
            status: "broadcast".to_owned(),
            tone: "good".to_owned(),
            title: "Broadcast sent".to_owned(),
            message: "The transaction was submitted to the configured RPC endpoint.".to_owned(),
            primary_action: "Refresh Receipt".to_owned(),
            next_step: "Refresh activity to check whether the transaction has been included or reverted.".to_owned(),
            reason_code: Some("awaiting_receipt".to_owned()),
        },
        "confirmed" | "included" => TransactionActivityGuidance {
            status: "confirmed".to_owned(),
            tone: "good".to_owned(),
            title: "Transaction confirmed".to_owned(),
            message: "The network returned an included transaction receipt.".to_owned(),
            primary_action: "Done".to_owned(),
            next_step: "No wallet action is needed for this transaction.".to_owned(),
            reason_code: Some("receipt_confirmed".to_owned()),
        },
        "reverted" => TransactionActivityGuidance {
            status: "reverted".to_owned(),
            tone: "bad".to_owned(),
            title: "Transaction reverted".to_owned(),
            message: "The transaction was included but reverted during execution.".to_owned(),
            primary_action: "Refresh dApp".to_owned(),
            next_step: "Return to the dApp, refresh the quote or position state, then retry only if the new request matches your intent.".to_owned(),
            reason_code: Some("receipt_reverted".to_owned()),
        },
        "rejected" => TransactionActivityGuidance {
            status: "rejected".to_owned(),
            tone: "warn".to_owned(),
            title: "Request rejected".to_owned(),
            message: "The trusted UI rejected this transaction before signing.".to_owned(),
            primary_action: "Retry from dApp".to_owned(),
            next_step: "Retry from the dApp only if you still want to perform this action.".to_owned(),
            reason_code: Some("user_rejected".to_owned()),
        },
        "expired" => TransactionActivityGuidance {
            status: "expired".to_owned(),
            tone: "warn".to_owned(),
            title: "Review expired".to_owned(),
            message: "The transaction was not approved before its trusted review window expired.".to_owned(),
            primary_action: "Retry from dApp".to_owned(),
            next_step: "Start the action again from the dApp so FRAMKey can build a fresh review.".to_owned(),
            reason_code: Some("review_expired".to_owned()),
        },
        _ => return None,
    };
    Some(guidance)
}

pub(crate) fn btc_transaction_activity_lifecycle_guidance(
    status: &str,
) -> Option<TransactionActivityGuidance> {
    let guidance = match status {
        "broadcast" => TransactionActivityGuidance {
            status: "broadcast".to_owned(),
            tone: "good".to_owned(),
            title: "BTC transaction broadcast".to_owned(),
            message: "The transaction was submitted to the configured BTC backend.".to_owned(),
            primary_action: "Refresh Balance".to_owned(),
            next_step:
                "Refresh the BTC account balance after the transaction appears on the network."
                    .to_owned(),
            reason_code: Some("btc_broadcast_submitted".to_owned()),
        },
        _ => return transaction_activity_lifecycle_guidance(status),
    };
    Some(guidance)
}

pub(crate) fn transaction_activity_failure_guidance(error: &str) -> TransactionActivityGuidance {
    let lower = error.to_ascii_lowercase();
    if lower.contains("insufficient funds")
        || lower.contains("insufficient balance")
        || lower.contains("exceeds balance")
        || lower.contains("cannot cover")
    {
        return TransactionActivityGuidance {
            status: "failed".to_owned(),
            tone: "bad".to_owned(),
            title: "Not enough gas funds".to_owned(),
            message:
                "The network rejected the transaction because the account cannot cover value plus gas."
                    .to_owned(),
            primary_action: "Fund Account".to_owned(),
            next_step:
                "Add native gas funds on this network, then retry the action from the dApp."
                    .to_owned(),
            reason_code: Some("insufficient_funds".to_owned()),
        };
    }
    if lower.contains("nonce too low")
        || lower.contains("already known")
        || lower.contains("replacement transaction underpriced")
        || lower.contains("transaction underpriced")
    {
        return TransactionActivityGuidance {
            status: "failed".to_owned(),
            tone: "warn".to_owned(),
            title: "Nonce or pending transaction conflict".to_owned(),
            message: "The network rejected the transaction because account pending state changed."
                .to_owned(),
            primary_action: "Refresh Activity".to_owned(),
            next_step:
                "Refresh activity and retry from the dApp after the pending transaction state settles."
                    .to_owned(),
            reason_code: Some("nonce_conflict".to_owned()),
        };
    }
    if lower.contains("chain id")
        || lower.contains("wrong chain")
        || lower.contains("invalid sender")
        || lower.contains("replay-protected")
    {
        return TransactionActivityGuidance {
            status: "failed".to_owned(),
            tone: "bad".to_owned(),
            title: "Check active network".to_owned(),
            message: "The signed transaction did not match the network accepted by the RPC endpoint."
                .to_owned(),
            primary_action: "Check Network".to_owned(),
            next_step:
                "Check RPC Health and switch FRAMKey to the network expected by the dApp before retrying."
                    .to_owned(),
            reason_code: Some("chain_mismatch".to_owned()),
        };
    }
    if lower.contains("execution reverted")
        || lower.contains("always failing transaction")
        || lower.contains("call exception")
        || lower.contains("revert")
    {
        return TransactionActivityGuidance {
            status: "failed".to_owned(),
            tone: "bad".to_owned(),
            title: "Transaction would revert".to_owned(),
            message: "The dApp transaction failed during EVM execution.".to_owned(),
            primary_action: "Refresh dApp".to_owned(),
            next_step:
                "Return to the dApp, refresh the quote or allowance state, then retry only after reviewing the new request."
                    .to_owned(),
            reason_code: Some("execution_reverted".to_owned()),
        };
    }
    if lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("connection")
        || lower.contains("transport")
        || lower.contains("temporarily unavailable")
        || lower.contains("503")
    {
        return TransactionActivityGuidance {
            status: "failed".to_owned(),
            tone: "warn".to_owned(),
            title: "RPC or network issue".to_owned(),
            message: "The transaction failed while talking to the configured RPC path.".to_owned(),
            primary_action: "Check RPC".to_owned(),
            next_step: "Check RPC Health, then retry from the dApp once the endpoint is healthy."
                .to_owned(),
            reason_code: Some("rpc_unavailable".to_owned()),
        };
    }

    TransactionActivityGuidance {
        status: "failed".to_owned(),
        tone: "warn".to_owned(),
        title: "Transaction failed".to_owned(),
        message: "The wallet or RPC rejected this transaction after review.".to_owned(),
        primary_action: "Review Error".to_owned(),
        next_step: "Read the activity error, check RPC Health, then retry from the dApp if the request still matches your intent.".to_owned(),
        reason_code: Some("unknown_failure".to_owned()),
    }
}

pub(crate) fn transaction_receipt_summary(
    receipt: &Value,
) -> Result<Option<TransactionReceiptSummary>> {
    if receipt.is_null() {
        return Ok(None);
    }
    let object = receipt.as_object().ok_or_else(|| {
        anyhow::anyhow!("eth_getTransactionReceipt returned a non-object receipt")
    })?;
    let raw_status = object.get("status").and_then(Value::as_str);
    let status = match raw_status {
        Some("0x1") => "confirmed",
        Some("0x0") => "reverted",
        Some(_) | None => "included",
    }
    .to_owned();

    Ok(Some(TransactionReceiptSummary {
        status,
        block_number: object
            .get("blockNumber")
            .and_then(Value::as_str)
            .map(str::to_owned),
        transaction_index: object
            .get("transactionIndex")
            .and_then(Value::as_str)
            .map(str::to_owned),
        gas_used: object
            .get("gasUsed")
            .and_then(Value::as_str)
            .map(str::to_owned),
        effective_gas_price: object
            .get("effectiveGasPrice")
            .and_then(Value::as_str)
            .map(str::to_owned),
    }))
}
