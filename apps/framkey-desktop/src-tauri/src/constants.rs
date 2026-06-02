use std::time::Duration;

pub(crate) const DEFAULT_KEYCHAIN_SERVICE: &str = "io.framkey.local-kek";
pub(crate) const DEFAULT_KEYCHAIN_ACCOUNT: &str = "default";
pub(crate) const DEFAULT_CHAIN_ID: &str = "0x1";
pub(crate) const DEFAULT_GBXCART_PORT: &str = "/dev/cu.usbserial-210";
pub(crate) const DEFAULT_ALCHEMY_NETWORK: &str = "eth-mainnet";
pub(crate) const DEFAULT_SIMULATION_TIMEOUT_MS: u64 = 5_000;
pub(crate) const DEFAULT_SIMULATION_DEFAULT_GAS: &str = "0x7a1200";
pub(crate) const DEFAULT_RPC_TIMEOUT_MS: u64 = 10_000;
pub(crate) const SIGNER_HELPER_TIMEOUT: Duration = Duration::from_secs(45);
pub(crate) const DEFAULT_MOCK_NATIVE_TRANSFER_GAS: &str = "0x5208";
pub(crate) const DEFAULT_MOCK_CONTRACT_CALL_GAS: &str = "0x7a120";
pub(crate) const PROVIDER_EVENT_LOG_LIMIT: usize = 200;
pub(crate) const TRANSACTION_ACTIVITY_LIMIT: usize = 32;
pub(crate) const TRANSACTION_RECEIPT_REFRESH_LIMIT: usize = 8;
pub(crate) const PORTFOLIO_TOKEN_BALANCE_MAX_COUNT: u64 = 100;
pub(crate) const PORTFOLIO_TOKEN_METADATA_LIMIT: usize = 16;
pub(crate) const TRANSACTION_TOKEN_METADATA_LIMIT: usize = 8;
pub(crate) const WALLET_UI_STATE_VERSION: u32 = 1;
pub(crate) const WALLET_WATCHED_ASSET_LIMIT: usize = 128;
pub(crate) const RECOVERY_UI_STATE_VERSION: u32 = 1;
pub(crate) const PRIVATE_DIR_MODE: u32 = 0o700;
pub(crate) const PRIVATE_FILE_MODE: u32 = 0o600;
pub(crate) const TRUSTED_UI_ORIGIN: &str = "framkey://trusted-ui";
pub(crate) const LOCAL_DAPP_URL: &str = "tauri://localhost/dapp.html";
pub(crate) const LOCAL_DAPP_ORIGIN: &str = "tauri://localhost";
pub(crate) const UNISWAP_URL: &str = "https://app.uniswap.org/";
pub(crate) const AAVE_URL: &str = "https://app.aave.com/";
pub(crate) const MACOS_NO_NETWORK_SANDBOX_PROFILE: &str =
    "(version 1) (allow default) (deny network*)";
pub(crate) const SIGNER_HELPER_BASENAME: &str = "framkey-signer-helper";

#[derive(Debug, Clone, Copy)]
pub(crate) struct SupportedAlchemyChain {
    pub(crate) chain_id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) alchemy_network: &'static str,
    pub(crate) native_symbol: &'static str,
}

pub(crate) const SUPPORTED_ALCHEMY_CHAINS: &[SupportedAlchemyChain] = &[
    SupportedAlchemyChain {
        chain_id: "0x1",
        name: "Ethereum",
        alchemy_network: "eth-mainnet",
        native_symbol: "ETH",
    },
    SupportedAlchemyChain {
        chain_id: "0xaa36a7",
        name: "Sepolia",
        alchemy_network: "eth-sepolia",
        native_symbol: "ETH",
    },
    SupportedAlchemyChain {
        chain_id: "0x2105",
        name: "Base",
        alchemy_network: "base-mainnet",
        native_symbol: "ETH",
    },
    SupportedAlchemyChain {
        chain_id: "0xa",
        name: "OP Mainnet",
        alchemy_network: "opt-mainnet",
        native_symbol: "ETH",
    },
    SupportedAlchemyChain {
        chain_id: "0xa4b1",
        name: "Arbitrum One",
        alchemy_network: "arb-mainnet",
        native_symbol: "ETH",
    },
    SupportedAlchemyChain {
        chain_id: "0x89",
        name: "Polygon",
        alchemy_network: "polygon-mainnet",
        native_symbol: "MATIC",
    },
];
