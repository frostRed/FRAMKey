use crate::{
    decoder::{looks_like_eth_address, same_chain_id},
    model::KnownProtocolCounterparty,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct KnownCounterparty {
    pub(crate) chain_id: &'static str,
    pub(crate) address: &'static str,
    pub(crate) label: &'static str,
    pub(crate) protocol: &'static str,
}

// Source-backed protocol labels for the chains the desktop app can switch to.
// These labels are review/policy context; they are not a replacement for simulation.
pub(crate) const KNOWN_COUNTERPARTIES: &[KnownCounterparty] = &[
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

pub fn known_protocol_counterparty(
    chain_id: &str,
    address: &str,
) -> Option<KnownProtocolCounterparty> {
    known_counterparty(chain_id, address).map(|known| KnownProtocolCounterparty {
        chain_id: known.chain_id,
        address: known.address,
        label: known.label,
        protocol: known.protocol,
    })
}
