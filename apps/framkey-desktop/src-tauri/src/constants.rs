use std::time::Duration;

pub(crate) const DEFAULT_KEYCHAIN_SERVICE: &str = "io.framkey.local-kek";
pub(crate) const DEFAULT_KEYCHAIN_ACCOUNT: &str = "default";
pub(crate) const DEFAULT_CHAIN_ID: &str = "0x1";
pub(crate) const DEFAULT_ALCHEMY_NETWORK: &str = "eth-mainnet";
pub(crate) const HYPEREVM_CHAIN_ID: &str = "0x3e7";
pub(crate) const HYPEREVM_NETWORK: &str = "hyperliquid-mainnet";
pub(crate) const HYPEREVM_RPC_URL: &str = "https://rpc.hyperliquid.xyz/evm";
pub(crate) const DEFAULT_SIMULATION_TIMEOUT_MS: u64 = 5_000;
pub(crate) const DEFAULT_SIMULATION_DEFAULT_GAS: &str = "0x7a1200";
pub(crate) const DEFAULT_RPC_TIMEOUT_MS: u64 = 10_000;
pub(crate) const DEFAULT_BTC_ESPLORA_TIMEOUT_MS: u64 = 10_000;
pub(crate) const DEFAULT_BTC_MAINNET_ESPLORA_URL: &str = "https://blockstream.info/api";
pub(crate) const DEFAULT_BTC_TESTNET4_ESPLORA_URL: &str = "https://mempool.space/testnet4/api";
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SupportedChainRpc {
    Alchemy {
        network: &'static str,
    },
    StaticJsonRpc {
        provider: &'static str,
        network: &'static str,
        endpoint_url: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SupportedChain {
    pub(crate) chain_id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) native_name: &'static str,
    pub(crate) native_symbol: &'static str,
    pub(crate) rpc: SupportedChainRpc,
    pub(crate) block_explorer_url: Option<&'static str>,
}

impl SupportedChain {
    pub(crate) fn rpc_network(self) -> &'static str {
        match self.rpc {
            SupportedChainRpc::Alchemy { network } => network,
            SupportedChainRpc::StaticJsonRpc { network, .. } => network,
        }
    }

    pub(crate) fn alchemy_network(self) -> Option<&'static str> {
        match self.rpc {
            SupportedChainRpc::Alchemy { network } => Some(network),
            SupportedChainRpc::StaticJsonRpc { .. } => None,
        }
    }

    pub(crate) fn rpc_provider(self) -> &'static str {
        match self.rpc {
            SupportedChainRpc::Alchemy { .. } => "alchemy",
            SupportedChainRpc::StaticJsonRpc { provider, .. } => provider,
        }
    }

    pub(crate) fn rpc_kind(self) -> &'static str {
        match self.rpc {
            SupportedChainRpc::Alchemy { .. } => "alchemy_rpc",
            SupportedChainRpc::StaticJsonRpc { .. } => "json_rpc",
        }
    }

    pub(crate) fn requires_alchemy_token(self) -> bool {
        matches!(self.rpc, SupportedChainRpc::Alchemy { .. })
    }

    pub(crate) fn supports_alchemy_token_api(self) -> bool {
        matches!(self.rpc, SupportedChainRpc::Alchemy { .. })
    }
}

pub(crate) const ETHEREUM_CHAIN: SupportedChain = SupportedChain {
    chain_id: "0x1",
    name: "Ethereum",
    native_name: "Ether",
    native_symbol: "ETH",
    rpc: SupportedChainRpc::Alchemy {
        network: "eth-mainnet",
    },
    block_explorer_url: Some("https://etherscan.io"),
};

pub(crate) const SEPOLIA_CHAIN: SupportedChain = SupportedChain {
    chain_id: "0xaa36a7",
    name: "Sepolia",
    native_name: "Ether",
    native_symbol: "ETH",
    rpc: SupportedChainRpc::Alchemy {
        network: "eth-sepolia",
    },
    block_explorer_url: Some("https://sepolia.etherscan.io"),
};

pub(crate) const BASE_CHAIN: SupportedChain = SupportedChain {
    chain_id: "0x2105",
    name: "Base",
    native_name: "Ether",
    native_symbol: "ETH",
    rpc: SupportedChainRpc::Alchemy {
        network: "base-mainnet",
    },
    block_explorer_url: Some("https://basescan.org"),
};

pub(crate) const OP_MAINNET_CHAIN: SupportedChain = SupportedChain {
    chain_id: "0xa",
    name: "OP Mainnet",
    native_name: "Ether",
    native_symbol: "ETH",
    rpc: SupportedChainRpc::Alchemy {
        network: "opt-mainnet",
    },
    block_explorer_url: Some("https://optimistic.etherscan.io"),
};

pub(crate) const ARBITRUM_ONE_CHAIN: SupportedChain = SupportedChain {
    chain_id: "0xa4b1",
    name: "Arbitrum One",
    native_name: "Ether",
    native_symbol: "ETH",
    rpc: SupportedChainRpc::Alchemy {
        network: "arb-mainnet",
    },
    block_explorer_url: Some("https://arbiscan.io"),
};

pub(crate) const POLYGON_CHAIN: SupportedChain = SupportedChain {
    chain_id: "0x89",
    name: "Polygon",
    native_name: "Matic",
    native_symbol: "MATIC",
    rpc: SupportedChainRpc::Alchemy {
        network: "polygon-mainnet",
    },
    block_explorer_url: Some("https://polygonscan.com"),
};

pub(crate) const HYPEREVM_CHAIN: SupportedChain = SupportedChain {
    chain_id: HYPEREVM_CHAIN_ID,
    name: "Hyperliquid",
    native_name: "HYPE",
    native_symbol: "HYPE",
    rpc: SupportedChainRpc::StaticJsonRpc {
        provider: "hyperliquid",
        network: HYPEREVM_NETWORK,
        endpoint_url: HYPEREVM_RPC_URL,
    },
    block_explorer_url: Some("https://hyperevmscan.io"),
};

pub(crate) const SUPPORTED_CHAINS: &[SupportedChain] = &[
    ETHEREUM_CHAIN,
    SEPOLIA_CHAIN,
    BASE_CHAIN,
    OP_MAINNET_CHAIN,
    ARBITRUM_ONE_CHAIN,
    POLYGON_CHAIN,
    HYPEREVM_CHAIN,
];
