# Product Roadmap

FRAMKey should not fork Rabby. The product path is to build FRAMKey's own wallet surface while keeping vault, signer, provider, simulation, and recovery code reusable across frontends.

## Direction

The near-term product should be a Tauri-based FRAMKey DeFi Browser. The long-term product should also include a Chrome/Brave extension, but only after the core provider, permission, simulation, and signing flows are stable.

```text
Short term:
  FRAMKey Tauri DeFi Browser
    -> trusted wallet UI
    -> untrusted dApp WebView
    -> provider injection
    -> simulation
    -> local confirmation
    -> signer helper
    -> GBA vault + Touch ID

Long term:
  FRAMKey browser extension
    -> shared provider core
    -> native host
    -> same signer / vault / recovery stack
```

Tauri is the preferred next product surface because it lets the GBA card state, Touch ID unlock, recovery status, simulation output, and trusted signing confirmation live in one app first. The browser extension remains important for daily DeFi compatibility, but it carries extra lifecycle, native-host registration, extension-store, and multi-tab complexity.

## Route Comparison

| Area | Own browser extension | Tauri DeFi Browser |
| --- | --- | --- |
| dApp compatibility | Best long-term fit for Chrome/Brave dApps | Medium; some dApps may not fully support embedded WebView behavior |
| MVP closure | More moving parts: extension, native host, desktop, signer | Faster single-app closure |
| Distribution | Extension install plus desktop/native-host setup | Desktop app first |
| Trusted confirmation | Split across extension and desktop unless carefully brokered | Natural trusted app UI |
| Transaction simulation | Still required | Still required |
| Long-term wallet shape | Traditional injected wallet | FRAMKey DeFi terminal |

The routes are not mutually exclusive. The architectural goal is a shared provider/signing core with multiple frontends.

## Version Plan

### v0.1: CLI + Hardware Vault

Status: mostly complete.

- GBxCart read/write.
- 64 KiB vault save image.
- Keychain + Touch ID KEK wrapper.
- Short-lived signer helper.
- EVM `personal_sign` smoke.
- Read-only Chrome native bridge smoke.

### v0.2: Tauri DeFi Browser

Status: usable development wallet path in progress.

- Created a Tauri app shell with two trust zones:
  - trusted wallet UI
  - untrusted dApp WebView
- Normal product startup now opens the trusted wallet UI first; the untrusted dApp WebView opens only from Apps or explicit startup/smoke configuration.
- Injected an EIP-1193/EIP-6963 provider into the local dApp WebView.
- Supported read-only account connection through the existing save-image, Keychain, and signer-helper stack.
- Captured signing and transaction methods into a trusted UI request-review queue.
- Added a dry-run local approval broker with expiry, one-time decision tokens, and trusted-window decision commands.
- Added controlled `personal_sign` for testnet/dev use: approved unexpired requests reach the signer helper, which validates the requested account before signing.
- Added a local decoded-review foundation for `eth_sendTransaction` that normalizes transaction fields, decodes common token approvals/transfers, and surfaces warnings.
- Added a simulation client boundary plus Alchemy `alchemy_simulateAssetChanges` adapter.
- Defaulted transaction review to live Alchemy asset-change simulation when an Alchemy endpoint is configured, while keeping local-only simulation as an explicit development override.
- Added controlled `eth_sendTransaction`: approved, policy-authorized requests are prepared, signed through signer-helper in Keychain mode, and broadcast through the configured RPC.
- Added transaction policy states for ordinary approval, explicit high-risk override, and non-overrideable blocked failures.
- Added trusted UI vault creation with recovery backup pack generation.
- Added trusted UI recovery backup guidance with per-file destinations and recovery rewrap result status.
- Added recovery rewrap for binding a recovery-enabled vault to the current Keychain item without decrypting the wallet secret.
- Hardened the injected provider compatibility surface with EIP-6963 announcement metadata, account/chain state, provider events, and legacy `send`/`sendAsync` aliases.
- Added trusted UI provider telemetry so remote dApp compatibility work can inspect injection, EIP-6963 discovery, provider methods, and sanitized failures without devtools.
- Added a development startup URL, read-only remote provider smoke, and stderr telemetry mode for repeatable Uniswap/Aave WebView smoke tests.
- Added interactive remote provider smoke for Uniswap/Aave that drives account connection, `personal_sign`, Permit typed-data signing, and `eth_sendTransaction` through mock-mode trusted review.
- Verified current-build interactive remote smoke for Uniswap and Aave with `.env` Alchemy read RPC, mock wallet approval/signing, Permit signing, and transaction review/signing.
- Added a trusted DeFi Session readiness panel plus recovery backup placement checks so normal wallet and backup state is easier to scan without inspecting raw JSON.
- Added trusted UI workspaces for Wallet, DeFi, Recovery, and Diagnostics so the app reads as a wallet product instead of one long debug console.
- Removed the in-app product header from the trusted desktop shell; the body now starts directly with workspace navigation and wallet content below the native macOS titlebar.
- Added a trusted dApp Compatibility run-status panel for Local Test, Uniswap, and Aave, summarizing provider/read/connect/Permit/sign/tx evidence from the current process.
- Added trusted UI `Check` actions for Local Test, Uniswap, and Aave that run read-only provider/RPC probes without signing or account approval.
- Added dApp compatibility guidance that converts raw target evidence into a status and next action such as read-ready, connect in dApp, or signing path proven.
- Added trusted dApp navigation state and reload/back/forward/home controls for the embedded DeFi Browser, with sanitized URL/origin display and no permission or signing side effects.
- Added a recovery placement checklist with destination cards for iCloud Drive, Google Drive, local physical storage, and remote physical storage; it reuses local placement checks and computes whether checked files satisfy the cloud-plus-physical or local-plus-remote recovery policy.
- Replaced separate vault-backup and recovery-share artifacts with four plain `.dat` backup bundles, each embedding encrypted vault durability plus one recovery authorization share.
- Added a Recovery Set Builder that fills recovery drill/rewrap inputs from the generated backup plan and shows live policy status while keeping manual moved-file paths possible.
- Added trusted macOS recovery file/folder pickers that return selected paths only to the trusted UI and do not expose filesystem access to dApps.
- Added a Recovery Health summary that keeps generated backup files visible while showing placement, drill, and recovery rewrap status together.
- Added a trusted Portfolio panel backed by Alchemy RPC for ETH balance, latest block, and nonzero ERC-20 balances with capped metadata enrichment.
- Added a trusted RPC Health panel that checks Alchemy chain id, latest block, and latency without exposing the token or endpoint.
- Added display-only transaction asset metadata enrichment so decoded approvals/transfers can show token symbols and decimal-adjusted amounts without changing policy decisions.
- Added a top-level transaction risk summary that shows policy decision, required approval path, simulation status, and exact blocker reasons before raw review details.
- Added top-level transaction signing guidance that explains whether the user can approve, must use high-risk confirmation, or cannot sign because policy/live simulation failed.
- Added Alchemy asset-change normalization so live `alchemy_simulateAssetChanges` responses populate the same transaction transfers/approvals UI as local decoded reviews.
- Added a trusted Transaction Activity panel for local sanitized transaction review, approval, broadcast hash/error, automatic pending-receipt polling, manual receipt refresh status, and restart restore.
- Hardened desktop local writes so Transaction Activity state and generated recovery packs start as owner-only files/directories on Unix/macOS.
- Added restart restore for sanitized Recovery Backup Plan state so generated backup paths, hashes, placement roles, drill result, and rewrap result remain visible while the user places cloud and physical copies.
- Added packaged signer-helper readiness: desktop builds prepare the helper as a Tauri sidecar, runtime discovery checks bundled app locations, and trusted status reports helper readiness without exposing wallet material.
- Added transaction recovery guidance in Activity and DeFi Session so blocked simulation, insufficient gas funds, nonce conflicts, wrong-network errors, and reverted transactions point to a concrete next action.
- Added controlled ERC-20 Permit and Uniswap Permit2 typed-data signing after trusted UI approval, while keeping unknown typed-data signing blocked.
- Added session-local per-origin account permissions with trusted UI approval and connected-site disconnect controls.
- Required connected account permission before dApps can request signature or transaction review.
- Added trusted-approval-gated `wallet_switchEthereumChain` for known Alchemy-backed session networks without rewriting config files or exposing RPC credentials.
- Added trusted-approval-gated `wallet_addEthereumChain` for the same known Alchemy-backed chains, verifying FRAMKey's own Alchemy endpoint while ignoring dApp-supplied RPC URLs as wallet configuration.
- Added trusted-approval-gated `wallet_watchAsset` for ERC-20 tokens, owner-only local watched-token persistence, and restart restore in Portfolio.
- Added a trusted Wallet-native native-token Send form that reuses the transaction review, signer-helper/mock signing, broadcast, and Activity pipeline without exposing a new dApp API.
- Added a trusted Portfolio ERC-20 Send flow that selects a token, validates decimal amount input, encodes `transfer(address,uint256)`, and reuses the same review/sign/broadcast/Activity pipeline without exposing a new dApp API.
- Simplified Home into a wallet status and daily action surface; backup creation, placement, and restore now stay in the Safety workspace, Home Connect/Disconnect only changes trusted local UI session state, and address-only refresh/account queries use the connected session address instead of touching the vault device.
- Made the trusted `Create Vault + Backups` action explicitly write-gated so the UI cannot invoke real vault creation until the configured-device write is confirmed.
- Added local intent decoding for common top-level Uniswap V2/V3, Universal Router, multicall, and Aave V3 transaction selectors, with protocol labels in trusted transaction review.
- Added backend-generated transaction risk summaries that combine policy blockers, simulation warnings, live-simulation state, and decoded protocol intent into a review-only level/action/reason model.
- Added backend-generated transaction impact summaries for native value movement, decoded transfers, approval changes, and live provider asset-change coverage.
- Added a backend-generated counterparty trust summary for known Uniswap, Permit2, and Aave contracts across the current switchable chains, with unknown active approval authorities requiring explicit high-risk approval.
- Keep unknown typed-data and raw `eth_sign` request capture without signing.
- Show account balance snapshots and structured simulation/decoded transaction summaries in the trusted wallet UI, with raw JSON kept as collapsible debug context.

Initial dApp targets should be a small explicit set, such as Uniswap, Aave, Pendle, 1inch, and LlamaSwap. The goal is compatibility learning, not broad coverage.

### v0.3: Shared Provider Core

Extract frontend-independent logic so Tauri and browser extension can reuse the same behavior:

- `framkey-provider-core`
- `framkey-rpc-router`
- `framkey-permission-store`
- `framkey-simulation-client`
- `framkey-signing-client`

This avoids duplicating origin/session/account/chain behavior between Tauri and Chrome.

### v0.4: Own Browser Extension

Return to the Chrome/Brave extension as a production frontend after the Tauri path proves the signing and simulation model.

- Reuse shared provider core.
- Keep extension secret-free.
- Keep native host as relay/orchestrator.
- Use the same approval broker and signer helper.
- Add packaging and native-host registration flows.

## Simulation Strategy

FRAMKey should not implement a full EVM simulator as the first path. The simulation boundary keeps local decoding, third-party simulation adapters, and policy evaluation separate. The provider/signing flow should treat live simulation as required input for ordinary transaction approval, while local-only or locally suspicious transactions need an explicit high-risk override and provider failures remain blocked.

The current simulation layer returns a conservative normalized summary and can attach a raw Alchemy `alchemy_simulateAssetChanges` response. Successful live Alchemy `result.changes` entries are normalized into the same transfer and approval fields as local decoding, while the raw provider response remains available for audit:

- chain id
- from/to/value/data
- asset balance changes
- approvals
- gas estimate
- warnings
- unknown calldata marker
- raw provider response for audit

Transaction signing is exposed only through trusted approval, transaction policy authorization, and signer-helper account validation. The known-counterparty registry now covers a narrow source-backed set across the current switchable chains: Uniswap V2 Router02, Uniswap V3 SwapRouter/SwapRouter02, Uniswap Universal Router, Permit2, and Aave V3 Pool where deployed on Ethereum, Sepolia, Base, OP Mainnet, Arbitrum One, and Polygon. It labels review cards and makes unknown active approval authority require high-risk approval. Before real funds, the policy gate still needs broader protocol coverage, deeper protocol semantics, clearer allowlists, and more production-grade risk policy.

Protocol-specific local decoding currently covers common token approvals/transfers plus selected Uniswap and Aave top-level intents. It is a review aid only: local-only decoded DeFi transactions still require the explicit high-risk path unless live simulation and policy evaluation allow ordinary approval.

## Security Invariants

- Remote dApp content is untrusted, including when loaded inside Tauri.
- dApp WebViews must not receive direct filesystem, Keychain, GBxCart, or signer-helper access.
- The browser extension must remain secret-free.
- The native host must remain a relay/orchestrator, not a signer.
- The signer helper remains the only process that may touch decrypted EOA wallet material.
- Signing requires trusted local confirmation.
- Transaction signing requires a successful policy evaluation: ordinary approval for low-risk live-simulated requests, explicit high-risk approval for overrideable warnings, and hard blocking for malformed requests or provider failures.

## Current Decision

The Chrome extension remains parked at read-only bridge status. The active large task is hardening the Tauri wallet app into a safer daily DeFi surface: better transaction policy, richer UI, recovery drills, and packaging/security hardening.
