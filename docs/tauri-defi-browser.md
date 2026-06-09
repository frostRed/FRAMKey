# Tauri DeFi Browser Foundation

This slice adds the first FRAMKey desktop app surface. It proves the trusted UI, untrusted dApp WebView, provider injection, card-to-Keychain-to-helper account path, and a local approval broker. Enabled signing paths are SIWE-only `personal_sign` after trusted-UI approval, recognized ERC-20 Permit/Permit2 `eth_signTypedData_v4` after trusted-UI approval, and `eth_sendTransaction` after trusted-UI approval plus transaction policy authorization. Non-SIWE personal messages, unknown typed data, raw `eth_sign`, and `eth_signTransaction` remain blocked before signer access.

## Run

Build the signer helper first so the desktop binary can find it next to itself:

```bash
cargo build -p framkey-signer-helper
cargo run -p framkey-desktop
```

For a debug `.app` bundle, build the helper first, then bundle the desktop app:

```bash
cargo build -p framkey-signer-helper
cd apps/framkey-desktop/src-tauri
cargo tauri build --debug --bundles app --no-sign
```

The desktop build copies the already-built helper into `src-tauri/binaries/framkey-signer-helper-<target-triple>` so Tauri can package it as a sidecar. Runtime helper discovery checks the desktop executable directory and the bundled app resources before falling back to explicit config or `FRAMKEY_SIGNER_HELPER`. `framkey_status` reports sanitized helper readiness, location, sandbox mode, BLAKE3, and hash-pin state so the trusted UI can show whether Keychain-vault signing is available. The trusted System panel also exposes `Repair Signing Access`, which launches the real helper for a Keychain-only access probe without reading the card or passing vault image bytes.

Normal startup opens:

- `main`: trusted FRAMKey wallet UI.

The untrusted `dapp` WebView is created only when the user opens the local test app, Uniswap, Aave, or a user-entered `http`/`https` URL from the trusted UI, or when explicit startup/smoke configuration asks for it. Once opened, it receives injected `window.framkey` and remains a separate untrusted window.

Home is only the current wallet status and daily action surface. It shows the loaded account, network/RPC state, asset snapshot, signing readiness, and the next wallet action in consumer-facing language. Backup creation, backup placement, and restore are owned by the Safety workspace. Home Unlock loads the local vault account into the trusted in-memory account session and refreshes portfolio state; it does not grant dApp account access. macOS local authorization is part of Unlock and signing when needed, while the System `Repair Signing Access` action remains available for troubleshooting without a separate CLI step. Home Disconnect clears that account session, the portfolio snapshot, token-send selection, pending review queue, and current in-memory dApp account grants without deleting Keychain items, GBA data, backup files, watched-token preferences, or transaction activity history.

The Wallet workspace has a chain-account surface instead of a single address-only view. A loaded secp256k1 vault exposes an active EVM EOA card plus Bitcoin mainnet and Bitcoin Testnet4 P2WPKH account cards. Testnet4 is the default BTC test network; Signet is reserved for controlled integration testing and does not appear as a default user wallet account. The EVM card keeps trusted EVM send and dApp actions. The Bitcoin cards can show and copy receive addresses, refresh balances through configured Esplora-compatible backends, and launch trusted UI-only BTC sends after the wallet is connected. BTC balance refresh and send use only the connected account session and do not implicitly unlock Keychain, invoke the signer helper, or read the GBA card before a send reaches approved signing. BTC sends use a PSBT/UTXO review path with recipient, fee, input, change, dust, RBF, network, and ownership checks before signer-helper or mock signing. They are not exposed to the untrusted dApp WebView or EIP-1193 provider.

The trusted App Window panel keeps a process-local dApp navigation snapshot: target label, sanitized current URL, origin, load status, last event, and update time. The URL snapshot strips query strings and fragments before rendering. The product path leads with app choices and approval state; URL/load details sit behind a trusted collapsible details section. The panel also exposes reload, back, forward, and home controls that only navigate the untrusted WebView. They do not grant account permission, approve requests, switch networks, sign, submit transactions, expose the Alchemy endpoint/token, or give the dApp direct Tauri command access.

The trusted `main` UI is split into five workflow workspaces: Home, DeFi, Safety, Activity, and System. It has no app-level product header below the native macOS titlebar; the trusted body starts directly with workspace navigation and wallet content. Home keeps daily wallet status, portfolio, and trusted send actions together. DeFi is the consumer dApp cockpit: app choice, first-screen approval, current app, wallet access, next action, sites with wallet access, launch controls, and the latest transaction outcome. Safety owns vault creation, backup placement, and restore. Activity owns post-approval outcomes, receipt tracking, persistence state, recovery guidance, and transaction history. System keeps provider events, compatibility checks, capabilities, raw command output, and lower-level readiness state. The selected workspace is stored in local WebView storage only. Approvals remain visible in every workspace so a pending account/signature/transaction/network-switch approval is not hidden by navigation.

The injected provider also announces itself through EIP-6963 with a stable FRAMKey UUID and icon. It sets `window.framkey` and only sets `window.ethereum` when no provider already exists, so embedded compatibility testing does not clobber another wallet provider.

## Provider Diagnostics

The trusted `main` window includes `DApp Provider Events` for remote-site compatibility work. It records a bounded process-local log of provider injection lifecycle events, EIP-6963 provider requests/announcements, and provider request outcomes. Each entry includes only diagnostic metadata: origin, URL, method, status, duration, result shape, result preview, and sanitized error code/message. Telemetry details from the untrusted dApp WebView are schema-whitelisted on the Rust boundary, so unknown fields, nested objects, raw params, calldata, and signatures are omitted before storage. Raw provider params, calldata, signatures, RPC URLs, Alchemy tokens, wallet secrets, and recovery material are not stored in this log.

Use this panel when opening Uniswap, Aave, or another remote dApp to see whether the page discovered FRAMKey through EIP-6963, which EIP-1193 methods it called, and whether failures are unsupported methods, permission-gating behavior, RPC configuration, or policy blocking.

The trusted `main` window also includes `DeFi Session`, a compact readiness surface derived from the same status, provider-event, connection, review-queue, and transaction-activity state. It summarizes wallet, RPC, injected-provider, and pending-review state; shows the active dApp origin, account grant, latest provider request, latest signature review, latest transaction review, and a conservative next action. When a transaction is policy-blocked, awaiting receipt, reverted, or failed after signing/broadcast, the next action prefers that transaction's recovery guidance. This panel is trusted-UI-only rendering and does not grant dApp permissions or add signing paths.

The `dApp Compatibility` panel rolls that same process-local evidence up by target for Local Test, Uniswap, and Aave. Each target shows whether provider injection, read RPC, account connection, Permit typed-data signing, `personal_sign`, and `eth_sendTransaction` have been seen and whether the latest evidence passed, failed, or reached a mock-account broadcast error. Each target also has a trusted `Check` action that opens the target and starts a read-only provider probe inside the untrusted WebView. The probe records provider injection plus `eth_chainId`, `eth_accounts`, and `eth_blockNumber`; it does not request account approval, switch networks, sign data, or send transactions. Above the raw step grid, each target card shows product guidance derived from the evidence: not checked, checking, read-ready, connected but signing untested, signing path proven, transaction path proven, or needs attention. It is a run-status view for the current app process, not a persistent certification of future dApp behavior.

## Vault Creation

The trusted `main` window includes `Create Vault + Backups` for generating a new real Keychain-vault image. The command is restricted to the trusted window, stays disabled until the configured-device write is explicitly confirmed, and uses `framkey-signer-helper` for wallet-secret generation, Keychain KEK access, recovery wrapper generation, and save-image encryption.

On macOS, the trusted Safety workspace includes native picker buttons for the recovery output directory and backup files. These app commands are restricted to the trusted `main` window, return selected POSIX paths only, and do not read selected file contents until the user starts restore. The untrusted dApp WebView does not receive dialog, filesystem, Keychain, GBxCart, recovery, or signer-helper access.

The desktop process writes recovery backup files before it writes the configured vault device. The output directory receives four plain-looking files, `backup-01.dat` through `backup-04.dat`; each file contains encrypted vault data plus one recovery share. Files are created with no-overwrite semantics, and on Unix/macOS the output directory and generated files are set owner-only before the user places cloud or physical copies. The returned UI output includes only paths, BLAKE3 hashes, public metadata, and signer-helper identity. It does not return recovery share bytes or plaintext wallet material.

After creation, the Recovery Backup Plan panel groups the generated files into concrete destinations:

- Cloud 1: iCloud Drive
- Cloud 2: Google Drive
- Local 1: local physical storage
- Local 2: off-site physical storage

It also shows the backup set id, wallet id, generation, BLAKE3 hashes, and `cloudAloneRecovers=false`. The first view is a destination checklist for iCloud Drive, Google Drive, local physical storage, and off-site physical storage. Each row has a short hash, local placement checkbox, and trusted-UI-only `Show` action that opens Finder for the existing local path. The recover form is prefilled with the two cloud backup files plus one local physical backup file so a recovery smoke can use a valid non-cloud-only set.

Its Recovery Health summary keeps backup-pack creation, placement state, and restore status visible together. It computes whether the checked placement boxes match the policy: iCloud plus Google Drive plus one physical backup, or one local physical plus one off-site physical backup. Cloud-only placement is always shown as insufficient. Those checks are UI state only; the app writes files locally and never uploads to cloud storage or copies to physical media.

The sanitized Recovery Backup Plan view is persisted to trusted local desktop state and restored on restart. It stores only public operation metadata, generated file paths, BLAKE3 hashes, placement roles, and restore status; it does not store recovery share bytes, wallet secret material, KEK, DEK, RRK, or recovery root key bytes. `Forget Plan` removes that local UI state without deleting generated backup files. If the user moves files outside FRAMKey, reveal and restore actions surface the normal missing-file errors.

The Recover Vault form chooses the recovery method before it asks for files. `Cloud + physical` shows three explicit slots: iCloud, Google Drive, and one physical backup file. `Two physical files` shows two slots: local and off-site. After creating a backup set in the same app session, the trusted UI may prefill the matching slots from the local sanitized plan, but there is no separate remembered-plan recovery path. Manual paths remain supported for files moved outside the generated directory. Restore itself is the authority: `framkey-signer-helper` validates the selected bundles, reconstructs recovery material internally only if sufficient, binds this Mac, and returns a rewritten encrypted save image for the configured GBA/file device.

Vault image size comes from the configured device:

- `gba-sram-fram-256k`: 32 KiB
- `gba-sram-fram-512kbit`: 64 KiB, the current default A88J target
- `gba-sram-fram-1mbit`: 128 KiB
- file device: existing file length, or 64 KiB when the file does not exist yet

`gba-eeprom-64k` is rejected for vault creation because its 8 KiB save area is below the current signer-helper minimum. The UI requires explicit configured-device overwrite confirmation before creating the vault.

## Recovery Rewrap

The trusted `main` window includes `Recover Keychain Vault` for binding an existing recovery-enabled vault image to the current macOS Keychain item. The user selects one backup bundle as the encrypted vault source and selects enough backup bundles for recovery authorization. The desktop process reads those files, then asks `framkey-signer-helper` to reconstruct the recovery root key, decrypt only the recovery DEK wrapper, and add a current Keychain DEK wrapper.

Recovery rewrap does not decrypt the wallet secret. The returned output includes the rewritten encrypted save-image hash, public metadata, Keychain wrapper metadata, recovery file paths, and share count. It does not return recovery share bytes, recovery root key bytes, DEK bytes, or wallet-secret bytes. Cloud-only backup sets fail because iCloud plus Google Drive is intentionally insufficient.

After recovery, the Recovery Backup Plan panel appends a recovery result view alongside the backup plan. It shows the used backup paths, share count, rewritten save-image hash, and explicit `walletSecretTouched=false` / `recoveryShareBytesPrinted=false` status.

The UI requires explicit configured-device overwrite confirmation before writing the recovered save image back to the configured GBxCart/file device.

## Supported Provider Methods

- `eth_chainId`
- `net_version`
- `eth_accounts`
- `eth_requestAccounts`
- `eth_blockNumber`
- `eth_call`
- `eth_estimateGas`
- `eth_feeHistory`
- `eth_gasPrice`
- `eth_getBalance`
- `eth_getBlockBy*`
- `eth_getCode`
- `eth_getLogs`
- `eth_getProof`
- `eth_getStorageAt`
- `eth_getTransaction*`
- `eth_maxPriorityFeePerGas`
- `eth_syncing`
- `framkey_getStatus`
- `framkey_getAccount`
- `wallet_getCapabilities`
- `wallet_getPermissions`
- `wallet_requestPermissions`
- `wallet_revokePermissions`
- `wallet_addEthereumChain` for known trusted chains after trusted approval.
- `wallet_switchEthereumChain` for the current chain, or for a known trusted chain after trusted approval.
- `wallet_watchAsset` for trusted-approval ERC-20 watch requests.

The Ethereum read methods are allowlisted and proxied by the desktop process through the configured trusted RPC endpoint. The dApp never receives the Alchemy token or RPC URL.

The supported trusted-chain set includes Ethereum, Sepolia, Base, OP Mainnet, Arbitrum One, Polygon, and HyperEVM mainnet (`0x3e7`). Alchemy-backed chains derive a trusted Alchemy endpoint from the configured token/network. HyperEVM uses the official Hyperliquid JSON-RPC endpoint (`https://rpc.hyperliquid.xyz/evm`) and native symbol `HYPE`; dApp-supplied RPC URLs are still ignored as wallet configuration. HyperEVM does not expose Alchemy token-balance or token-metadata methods through the official RPC, so Portfolio falls back to native balance, latest block, and approved watched ERC-20 entries while transaction review keeps local decoded token contract context without Alchemy metadata.

The injected provider keeps local compatibility state for `selectedAddress`, `chainId`, `networkVersion`, and `isConnected()`. Successful account and chain requests emit the standard `connect`, `accountsChanged`, and `chainChanged` events. `wallet_addEthereumChain` requires trusted approval, verifies FRAMKey's own trusted endpoint, ignores dApp-supplied RPC URLs as wallet configuration, and returns success without silently switching the active session chain. `wallet_switchEthereumChain` never silently changes the wallet network: switching to a supported trusted chain is captured in the trusted review queue, and approval updates only the current app session's chain id plus trusted read-RPC endpoint. `wallet_watchAsset` accepts only ERC-20 token watch requests with valid address, symbol, and decimals; approval stores public token metadata in owner-only local trusted wallet state and shows the token in Portfolio, but does not grant account access, expose a token-list API to dApps, or affect transaction policy. Watched-token decimals from dApps remain display-only; trusted ERC-20 sends read `decimals()` from the token contract through FRAMKey's trusted RPC before encoding raw amounts. The watched-token list is restored after restart, while account grants, provider events, compatibility evidence, pending reviews, and dApp session state remain process-local. The Home workspace exposes the same supported-chain set through its trusted Active Network selector for user-initiated switching before or during DeFi use. Config files and `.env` are not rewritten. Unsupported chains, missing Alchemy token for Alchemy-backed chains, or a trusted endpoint that cannot prove the requested `eth_chainId` fail before mutating session state. The provider also exposes common legacy aliases used by connector libraries: `addListener`, `off`, `once`, `listenerCount`, `listeners`, `enable`, `send`, and `sendAsync`. These shims only route back into the same `framkey_provider_request` command; policy, unsupported-method handling, signing, and RPC allowlisting remain in Rust.

Account exposure is origin-scoped. `eth_accounts` returns `[]` until the dApp origin receives an account grant. `eth_requestAccounts` and `wallet_requestPermissions` capture a trusted UI review request; approval grants only `eth_accounts` for that origin during the current Tauri process session. `wallet_getPermissions` reports the session grant, and `wallet_revokePermissions` or the trusted UI Connected Sites panel can remove it. A dApp must have this grant before it can request signature or transaction review; otherwise FRAMKey returns an unauthorized provider error and does not add a review item. The grant only makes the origin eligible to ask for review, and does not authorize signing or transactions by itself. This provider account exposure is EVM-only; the trusted UI/status `accounts` array can include BTC receive accounts, but dApps do not receive a BTC account through EIP-1193.

The bridge captures these methods into the trusted UI approval broker, then rejects them before signer-helper access:

- `eth_sign`
- `eth_signTransaction`
- unrecognized `eth_signTypedData*`

Recognized ERC-20 Permit and Uniswap Permit2 `eth_signTypedData_v4` requests are enabled after account connection and trusted-window approval. The desktop process verifies more than the Permit shape before approval can authorize signing: the RPC signer account must be present, the EIP-712 type definitions must exactly match the recognized Permit or Permit2 schema, ERC-20 `message.owner` must match that signer, Permit2 owner must match when present, `domain.chainId` must match the active chain, Permit2 `verifyingContract` must be the known Permit2 contract on that chain, spender must be a known protocol counterparty, deadlines/expirations must be valid bounded future timestamps, and max-allowance Permit amounts are blocked. Only then does the app delegate the EIP-712 digest/signature operation to `framkey-signer-helper` in Keychain-vault mode or to the process-lifetime mock EOA in mock mode.

`eth_sendTransaction` is enabled in both Keychain-vault and mock wallet modes. The desktop app fills missing nonce/gas/fee fields through the trusted RPC, preferring EIP-1559 `eth_feeHistory` defaults and falling back to legacy `eth_gasPrice` only when no EIP-1559 fee fields were supplied. It rejects unsupported transaction envelopes before review: type 1 access-list envelopes, type 3 blob envelopes, blob sidecar fields, EIP-7702 `authorizationList`, non-empty `accessList`, invalid EIP-1559 fee relationships, and extreme fee values are not silently ignored. When the dApp omits nonce, FRAMKey still queries `eth_getTransactionCount(..., "pending")`, then reserves the next nonce in process memory so two local sends cannot prepare the same nonce while the RPC pending count lags. It captures the prepared request for trusted UI review and signs only when the default conservative policy allows ordinary approval. Allowed transaction shapes are native/ERC-20 transfers, finite approvals to known Uni/Aave counterparties, recognized bounded Uniswap swaps with short deadlines, bounded Universal Router Permit2 commands, and recognized Aave supply/repay/borrow/withdraw/collateral flows when known-pool, signer-owned-account, health-factor, and exact dry-run evidence requirements are met. Unknown calldata, unknown approval authority, unsupported Universal Router commands, unbounded Universal Router Permit2 authority, multicall incomplete semantics, zero-output, missing-deadline, stale, or long-deadline Uniswap swaps, third-party swap recipients, third-party Aave accounts/recipients, missing Aave risk evidence, malformed requests, and provider failures are blocked before signer-helper or mock signing. In the default real Keychain-vault mode, signing is delegated to the short-lived `framkey-signer-helper`; in mock mode, signing uses the process-lifetime mock EOA.

The trusted UI is now the primary confirmation surface rather than a raw JSON viewer. The account panel displays a balance snapshot through the same proxied RPC path when an account is connected. Review cards summarize the dApp origin, request intent, transaction counterparty, native value, decoded protocol/function, policy state, gas, nonce, calldata size, warnings, approvals, and transfers. Transaction cards start with backend-generated signing guidance that maps policy state to the user's available action: ordinary approval or cannot sign. Blocked live-simulation failures point the user toward RPC health/retry instead of presenting only raw policy fields. Transaction cards also use backend-generated risk and impact summaries to put risk level, required approval action, simulation mode/status, broadcast permission, native value movement, approval/transfer counts, and exact reason codes/messages in the top-level confirmation area before the raw details. Raw review summary and params remain collapsible for debugging and audit.

Pending approvals are visible from every trusted workspace through the `Approvals` panel. In the DeFi workspace, the first pending request is also promoted into a first-screen approval card with user-intent title, consequence text, risk/impact badges, and approve/reject actions wired to the same review broker decision path. Method names, technical summary, and request data remain available as collapsible details instead of leading the confirmation.

The trusted Activity workspace records the post-review transaction lifecycle in local trusted desktop state. It starts when an `eth_sendTransaction` review is captured, updates when the user approves or rejects it, records successful broadcast hashes or signing/broadcast errors, and refreshes `eth_getTransactionReceipt` for recent broadcast hashes through the configured trusted RPC. The Activity top surface summarizes latest outcome, receipt/network status, local history persistence, and the next user action before showing the full transaction list. DeFi also mirrors the latest actionable outcome so the dApp flow stays understandable without leaving the DeFi tab. The UI automatically polls pending receipts on a bounded interval while refreshable transaction hashes exist, and still keeps manual `Refresh` and `Refresh Receipts` controls for diagnostics and transient state recovery. Sanitized activity entries are persisted to local app state and restored on restart, while provider events, review queue items, account grants, and dApp compatibility evidence remain process-local. On Unix/macOS, the activity directory and JSON file are written owner-only. Restored `review_pending` or `approved` activities are marked expired because the trusted review queue itself is not persisted; the user must retry from the dApp to create a fresh approval. If the activity file is corrupt or unavailable, the app starts with an empty activity log and shows a sanitized persistence warning instead of blocking wallet startup. Each activity item also carries sanitized recovery guidance: live simulation failures point to RPC health or retrying simulation, insufficient-funds broadcast errors point to funding native gas on the active network, nonce conflicts point to refreshing pending activity, chain mismatches point to checking the active network, and reverted execution points back to refreshing dApp quote/allowance state. Receipt rendering keeps only a small sanitized subset: status, block number, transaction index, gas used, and effective gas price. It does not expose a dApp history method and does not store raw calldata, raw signed transactions, signatures, Alchemy credentials, wallet secrets, or recovery material.

The bridge captures SIWE-only `personal_sign`, waits for a trusted-window approval, and then delegates the signature to `framkey-signer-helper`. Non-SIWE messages are captured for review and blocked before signer access:

- `personal_sign`

## Trusted Wallet Send

The Home workspace includes a trusted native-token send form for ordinary ETH/MATIC/HYPE-style transfers. It accepts a recipient EVM address and a plain decimal native amount, rejects ambiguous amount formats, and constructs a no-calldata `eth_sendTransaction` request from the trusted UI origin. The untrusted dApp WebView cannot call this command. This send flow remains EVM-only even when a BTC account is present.

Native send uses the same backend path as a dApp transaction after input validation: nonce, gas, and fee data are completed through the trusted RPC boundary; the request is captured in the review queue; the user approves or rejects the review card; signing goes through `framkey-signer-helper` in Keychain-vault mode or the mock EOA in mock mode; broadcast uses `eth_sendRawTransaction`; and the result or sanitized failure is recorded in Transaction Activity.

The Wallet workspace also includes a BTC send form for Bitcoin mainnet/Testnet4 accounts when a backend is configured and an account session is connected. It accepts a BTC recipient address, satoshi amount, and bounded fee rate, fetches confirmed owned UTXOs from the selected Esplora-compatible backend, builds a P2WPKH PSBT with change back to the selected account, captures a BTC transaction review, signs only after BTC-specific policy authorization, validates the final raw transaction against the reviewed plan, and broadcasts through the selected BTC backend. This command is trusted UI-only and does not create a dApp-callable BTC provider API.

The Home workspace also includes trusted ERC-20 transfer controls in Portfolio. A token card can be selected for `transfer(address,uint256)` when the contract and trusted decimals are known. The trusted UI accepts a recipient and plain decimal amount, but the backend reads `decimals()` from the token contract through the trusted RPC before converting that amount to raw token units. It then locally encodes the ERC-20 transfer calldata and reuses the same review, signer-helper/mock signing, broadcast, and Activity path as any other transaction. Portfolio token metadata is display/input context only; the signed transaction is determined by the token contract, contract-returned decimals, and encoded calldata shown in review.

The trusted wallet send commands are not provider APIs and are not exposed to the untrusted dApp WebView. Approval/allowance-management UX remains separate from direct ERC-20 transfer.

## Default Configuration

The default development config matches the current verified card path:

- GBxCart port: `/dev/cu.usbserial-210`
- save type: `gba-sram-fram-512kbit`
- chain id: `0x1`
- Keychain service: `io.framkey.local-kek`
- Keychain account: `default`
- signer helper: `target/debug/framkey-signer-helper`

Optional JSON config is read from:

```text
~/.framkey/desktop.json
```

Sanitized Transaction Activity is persisted separately to:

```text
~/.framkey/transaction-activity.json
```

Set `FRAMKEY_DESKTOP_ACTIVITY_PATH` to override that development path.

Sanitized Recovery Backup Plan state is persisted separately to:

```text
~/.framkey/recovery-state.json
```

Set `FRAMKEY_DESKTOP_RECOVERY_STATE_PATH` to override that development path.

Trusted watched-token wallet UI state is persisted separately to:

```text
~/.framkey/wallet-state.json
```

Set `FRAMKEY_DESKTOP_WALLET_STATE_PATH` to override that development path. This file stores only approved ERC-20 watched-token metadata such as chain id, contract address, symbol, decimals, optional image URL, origin, and watched timestamp. It does not store account grants, provider events, pending reviews, raw params, calldata, signatures, RPC credentials, wallet secrets, or recovery material. On Unix/macOS, the file and containing directory are owner-only.

Example GBxCart config:

```json
{
  "chain_id": "0x1",
  "device": {
    "kind": "gbx_cart",
    "port": "/dev/cu.usbserial-210",
    "save_type": "gba-sram-fram-512kbit"
  },
  "keychain": {
    "service": "io.framkey.local-kek",
    "account": "default"
  },
  "signer_helper": {
    "path": "/absolute/path/to/FRAMKey/target/debug/framkey-signer-helper",
    "blake3": "<optional-helper-blake3>",
    "allow_unsandboxed": false
  },
  "simulation": {
    "kind": "local_decoder_only"
  },
  "rpc": {
    "kind": "alchemy",
    "rpc_url": "https://eth-mainnet.g.alchemy.com/v2/<api-key>",
    "network": "eth-mainnet",
    "timeout_ms": 10000
  }
}
```

Example HyperEVM read-RPC config:

```json
{
  "chain_id": "0x3e7",
  "simulation": {
    "kind": "local_decoder_only"
  },
  "rpc": {
    "kind": "json_rpc",
    "rpc_url": "https://rpc.hyperliquid.xyz/evm",
    "network": "hyperliquid-mainnet",
    "timeout_ms": 10000
  }
}
```

Example Alchemy simulation config:

```json
{
  "simulation": {
    "kind": "alchemy_asset_changes",
    "rpc_url": "https://eth-mainnet.g.alchemy.com/v2/<api-key>",
    "network": "eth-mainnet",
    "timeout_ms": 5000,
    "default_gas": "0x7a1200"
  }
}
```

For UI testing without GBxCart:

```json
{
  "device": {
    "kind": "file",
    "path": "/absolute/path/to/FRAMKey/save_image_samples/20260531-signer-helper-live-smoke/keychain-signer-helper-readback.sav"
  }
}
```

Environment overrides:

- `FRAMKEY_DESKTOP_CONFIG`
- `FRAMKEY_DESKTOP_ACTIVITY_PATH`
- `FRAMKEY_DESKTOP_WALLET_STATE_PATH`
- `FRAMKEY_DESKTOP_RECOVERY_STATE_PATH`
- `FRAMKEY_DESKTOP_START_URL`, `FRAMKEY_DESKTOP_START_DAPP`, or `FRAMKEY_DESKTOP_DAPP_URL`
- `FRAMKEY_DESKTOP_CHAIN_ID`
- `FRAMKEY_SAVE_IMAGE_PATH`
- `FRAMKEY_GBXCART_PORT`
- `FRAMKEY_GBA_SAVE_TYPE`
- `FRAMKEY_EXPECTED_SAVE_SIZE`
- `FRAMKEY_KEYCHAIN_SERVICE`
- `FRAMKEY_KEYCHAIN_ACCOUNT`
- `FRAMKEY_SIGNER_HELPER`
- `FRAMKEY_SIGNER_HELPER_BLAKE3`
- `FRAMKEY_DESKTOP_ALLOW_UNSANDBOXED_HELPER`
- `FRAMKEY_WALLET_MODE` or `FRAMKEY_DESKTOP_WALLET_MODE`: `keychain_vault` or `mock_in_memory`
- `FRAMKEY_SIMULATION_PROVIDER`: `local_decoder_only` or `alchemy_asset_changes`
- `FRAMKEY_RPC_URL`
- `FRAMKEY_ALCHEMY_RPC_URL`
- `ALCHEMY_RPC_URL`
- `FRAMKEY_ALCHEMY_TOKEN` or `ALCHEMY_TOKEN`
- `FRAMKEY_ALCHEMY_NETWORK` (default `eth-mainnet` when token-derived)
- `FRAMKEY_RPC_TIMEOUT_MS`
- `FRAMKEY_SIMULATION_TIMEOUT_MS`
- `FRAMKEY_SIMULATION_DEFAULT_GAS`
- `FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR`
- `FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE`: `read`, `interactive`, or `1` for read-only
- `FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE_CHAIN_ID`: optional supported target chain id for interactive smoke
- `FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE`
- `FRAMKEY_DESKTOP_WALLET_SEND_AUTOSMOKE`

When an Alchemy-backed chain has Alchemy RPC configured, the debug desktop app uses Alchemy live asset-change simulation by default for transaction review. It can read `ALCHEMY_TOKEN` from the shell environment or the repo `.env`, derives the RPC URL from `FRAMKEY_ALCHEMY_NETWORK`, and does not expose the token or URL in `framkey_status`. Use `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only` for deterministic development/offline smoke, or `FRAMKEY_SIMULATION_PROVIDER=alchemy_asset_changes` to force live simulation when the simulation endpoint is configured separately. For HyperEVM (`FRAMKEY_DESKTOP_CHAIN_ID=0x3e7`), a stray `ALCHEMY_TOKEN` does not switch simulation to Ethereum mainnet; the default remains local decoding unless Alchemy simulation is explicitly configured. For macOS bundle-style launches, prefer putting values in `.env` or `~/.framkey/desktop.json` instead of relying on a one-shot shell prefix.

For read-only dApp RPC, Alchemy is the preferred provider on the Alchemy-backed supported chains. `ALCHEMY_TOKEN` alone is enough in debug builds: the app derives `https://eth-mainnet.g.alchemy.com/v2/<api-key>` unless the active supported chain, `FRAMKEY_ALCHEMY_NETWORK`, or an explicit RPC URL selects another endpoint. HyperEVM is the exception: its trusted endpoint is the official Hyperliquid JSON-RPC URL, no Alchemy token is required, and Alchemy token discovery/metadata are reported as unsupported rather than queried.

The trusted RPC Health panel uses the same configured trusted RPC without exposing the endpoint or token to dApp JavaScript. It probes `eth_chainId` and `eth_blockNumber`, measures latency, verifies the endpoint matches the active session chain, and shows sanitized errors for missing, failed, or wrong-chain RPC. The trusted Portfolio panel uses that configured RPC for assets. `Refresh Assets` requires an already connected account session, then uses only the cached address to query `eth_getBalance` and `eth_blockNumber`; Alchemy token discovery/metadata run only when the configured RPC supports those methods. It does not unlock Keychain, invoke the signer helper, or read the GBA card. The UI shows nonzero token balances where provider discovery is available, approved watched ERC-20 tokens, metadata when available, and partial-error states; token metadata fan-out is capped so historical token spam cannot hang the trusted window.

Transaction review uses the same trusted metadata path only as display context. When local decoding identifies token contracts in ERC-20 approvals/transfers or NFT approvals/transfers, the desktop process attaches an `assetContext` summary with token metadata when available. Policy decisions still come from simulation and policy evaluation, not from metadata; missing or misleading metadata cannot turn a blocked transaction into an allowed one. The contract address remains visible in the raw review summary.

Typed-data review is structured and signable only for recognized Permit shapes. The trusted UI recognizes common ERC-20 Permit and Uniswap Permit2 shapes and shows owner, spender, token, amount, nonce, and deadline context before an approval can sign. Unknown EIP-712 payloads keep the bounded raw preview and remain blocked before signer-helper access.

For UI/dApp development without card or local authentication prompts:

```bash
FRAMKEY_WALLET_MODE=mock_in_memory cargo run -p framkey-desktop
```

The mock wallet is a process-lifetime EOA. It is visibly reported as mock in status and account metadata, supports the same trusted-UI `personal_sign` and `eth_sendTransaction` approval flows, and must not be used as a production wallet mode.

A normal mainnet mock account is usually unfunded, so the final broadcast may return an Alchemy/provider error such as insufficient funds after the app has successfully reviewed and signed the transaction. If gas estimation fails for that reason, mock mode uses a visible development fallback gas limit before signing: `0x5208` for plain native transfers and `0x7a120` for contract calldata. Real Keychain-vault mode does not use that fallback; gas estimation must succeed or be supplied by the dApp.

For repeatable runtime UI smoke without local authentication or external GUI automation:

```bash
FRAMKEY_WALLET_MODE=mock_in_memory \
FRAMKEY_SIMULATION_PROVIDER=local_decoder_only \
FRAMKEY_DESKTOP_AUTOSMOKE=1 \
cargo run -p framkey-desktop
```

This development-only mode explicitly opens the local dApp WebView, logs `framkey_window_smoke` and `framkey_runtime_smoke` lines, and lets the local dApp send provider requests while the trusted UI WebView auto-approves mock-mode account connection, message signing, and policy-allowed transaction review requests. It proves the real WebViews, provider injection, Tauri commands, approval broker, and mock signing path are wired together; it should not be enabled for real-vault use. Add `FRAMKEY_DESKTOP_RECOVERY_AUTOSMOKE=1` to generate a disposable recovery smoke pack without touching Keychain, local authentication, GBxCart, or the configured vault device. That smoke writes the same four backup bundle files as real vault creation, then verifies through the read-only recovery drill that cloud-only backups fail and the recommended cloud-plus-physical set passes.

Add `FRAMKEY_DESKTOP_WALLET_SEND_AUTOSMOKE=1` only when you also want runtime smoke to submit the trusted Wallet send forms. In mock wallet mode, the trusted UI fills the native send form with a one-wei transfer, waits for a watched ERC-20 token to appear in Portfolio, selects it, and submits the token send form. Both actions still create normal review requests and rely on the autosmoke approval loop. On an unfunded mock account, sanitized insufficient-funds broadcast failures are expected and are recorded in Transaction Activity.

For remote dApp startup smoke without relying on manual clicks:

```bash
FRAMKEY_WALLET_MODE=mock_in_memory \
FRAMKEY_RPC_TIMEOUT_MS=30000 \
FRAMKEY_DESKTOP_START_URL=uniswap \
FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE=read \
FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1 \
cargo run -p framkey-desktop
```

`FRAMKEY_DESKTOP_START_URL` accepts `local`, `uniswap`, `aave`, or a full `http`/`https` URL and uses the same validation path as the trusted UI open command. Normal product startup omits this setting and does not open a dApp window. For manual product checks, use the `Check` action in the dApp Compatibility panel; it starts the same read-only `eth_chainId`, `eth_accounts`, and `eth_blockNumber` probe after opening the target. For startup automation, `FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE=read` (or `1`) opens the local dApp when no explicit start URL is supplied, then runs that read-only probe through the injected provider after page load. `FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1` mirrors the trusted UI provider event log to terminal with sanitized fields only. Use it to capture evidence that a remote page loaded, received the injected provider, requested EIP-6963 provider announcements, and called or did not call expected EIP-1193 methods.

For interactive remote smoke, use mock mode and let the trusted UI approve through the same review broker a user would use manually:

```bash
FRAMKEY_WALLET_MODE=mock_in_memory \
FRAMKEY_SIMULATION_PROVIDER=local_decoder_only \
FRAMKEY_RPC_TIMEOUT_MS=30000 \
FRAMKEY_DESKTOP_START_URL=uniswap \
FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE=interactive \
FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE_CHAIN_ID=0xaa36a7 \
FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE=1 \
FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE_DURATION_MS=90000 \
FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1 \
cargo run -p framkey-desktop
```

Interactive smoke first runs the read-only checks, then optionally requests `wallet_switchEthereumChain` when `FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE_CHAIN_ID` is set. The switch uses the same trusted review broker and supported-chain allowlist as manual dApp requests; unsupported chains, missing Alchemy token for Alchemy-backed chains, or a trusted endpoint that cannot prove the target `eth_chainId` fail before session mutation. HyperEVM can be selected with `0x3e7` and uses the built-in Hyperliquid RPC endpoint. After the switched `eth_chainId` is verified, smoke requests account connection, sends a short-lived SIWE `personal_sign`, signs deterministic Permit2 `eth_signTypedData_v4`, and submits a deterministic transaction request; whether that request signs or is blocked depends on the same conservative transaction policy used in normal review. The trusted auto-approval loop starts only when status reports the mock wallet, so this mode does not approve real Keychain-vault/card requests. `FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE_DURATION_MS` is optional and exists for slow remote pages such as Aave; omit it to use the normal development default. Use `local_decoder_only` for deterministic local transaction review while read RPC, nonce/fee completion, and broadcast still use the active trusted RPC. Without that override, Alchemy simulation is the default only when an Alchemy endpoint is configured, and an unfunded mock transaction can be blocked by policy before signing. A final `eth_sendTransaction` insufficient-funds error from the mock account on the active session chain is expected only when the deterministic request is policy-allowed and reaches signing/broadcast; otherwise a blocked policy outcome is expected.

Current-build remote smoke evidence: Uniswap and Aave interactive smoke both reached provider injection/EIP-6963 discovery, Alchemy-backed read RPC, trusted account approval, SIWE `personal_sign`, controlled Permit `eth_signTypedData_v4`, and `eth_sendTransaction` review in `mock_in_memory` mode. Depending on the deterministic transaction shape, policy may block before signing or allow signing/broadcast before the expected mock mainnet insufficient-funds error. Uniswap multi-chain smoke has also passed after a trusted switch to Sepolia (`0xaa36a7`): the switched read RPC succeeded, SIWE `personal_sign` and Permit2 signing succeeded, and transaction review followed the same conservative policy gate. Aave may navigate embedded frames through `about:blank` and `app.family.co`, but the top-level origin still received the FRAMKey provider and completed the smoke flow.

## Trust Boundary

The dApp WebView is untrusted even though it is local in this foundation slice. It receives only the injected provider as the supported wallet API, and the Tauri global API is not exposed with `withGlobalTauri`. It does not receive direct filesystem, Keychain, GBxCart, or signer-helper access.

The signer helper remains the only process that may touch decrypted EOA wallet material. The desktop app can ask the helper to probe Keychain KEK access by service/account only; that probe returns public Keychain wrapper metadata and explicitly reports that it did not touch the card, a vault image, or the wallet secret. The desktop app reads the save image and asks the helper to open the Keychain vault for public account metadata only during explicit account loading. For approved SIWE `personal_sign` requests, the desktop app passes only the save image, message bytes, and requested account to the helper. For approved real-vault `eth_sendTransaction` requests, the desktop app passes only the prepared transaction, save image, and requested account to the helper; the helper signs offline with network access denied. The helper derives the vault address before signing and refuses an account mismatch.

## Approval Broker

Captured requests are kept in process memory only. The trusted UI shows:

- dApp origin
- provider method
- pending/approved/rejected/expired/signed/sign-failed status
- structured summary
- bounded params preview
- expiry time
- one-time local decision-token state

For account connection, approval grants only `eth_accounts` for the requesting origin in memory. For `personal_sign`, approval is real only for short-lived SIWE messages whose domain and URI match the requesting origin, whose account and chain match the active session, and whose expiration is present and no more than 30 minutes out. Non-SIWE personal messages are captured for review but return a blocked provider error before approval or signer access. Recognized Permit/Permit2 `eth_signTypedData_v4` requests follow the same approval rule, but unknown typed-data payloads still return a blocked error. For `eth_sendTransaction`, approval is real only when the origin is connected and the captured policy permits it; blocked transaction policies do not reach signer-helper or mock signing. For raw `eth_sign` and `eth_signTransaction`, connected origins can capture a dry-run review, but the provider still returns a blocked error. The broker enforces a short TTL, consumes decision tokens once, binds each request to the current broker session and origin metadata, and restricts decision commands to the trusted `main` window.

`eth_sendTransaction` review now includes a conservative decoded-review report. It normalizes the transaction fields, decodes common ERC-20 transfer/approve, ERC-20 transferFrom, ERC-721/1155 operator approval, NFT transfer selectors, top-level Uniswap V2/V3 swap calls, supported Universal Router `execute` command inputs, generic multicall, and Aave V3 Pool supply/withdraw/borrow/repay/collateral toggles, and flags unknown or malformed calldata. Supported Universal Router decoding summarizes V2/V3 swap recipients, path endpoints, minimum-output or maximum-input fields, payer direction, Permit2 PermitSingle details, Permit2 transfer details, and token sweep/transfer/wrap recipients; unsupported command IDs are blocked instead of being treated as safe. Policy also requires every Universal Router swap to include a short transaction-level deadline, and embedded Permit2 permit commands must keep amount below the maximum uint160 allowance with valid bounded expiration and signature deadlines. Dynamic router payloads are summarized with bounded counts or byte lengths rather than expanded as raw calldata. The default `framkey-simulation` client is offline local decoding. The preferred live provider is Alchemy `alchemy_simulateAssetChanges` (https://www.alchemy.com/docs/data/simulation-apis/transaction-simulation-endpoints/alchemy-simulate-asset-changes); when enabled, successful `result.changes` entries are normalized into the existing `assetTransfers` and `approvals` report fields so the trusted confirmation card can show live-simulated asset movement before raw debugging details. Aave borrow, withdraw, and collateral-disable reviews also attach sanitized protocol evidence from `eth_call getUserAccountData(address)` and an exact transaction `eth_call` dry run when the pool is recognized and read RPC is configured. The evidence records current account-level health-factor fields plus dry-run status; it is not labeled as post-transaction position evidence, so policy still requires conservative thresholds and blocks missing or unsafe evidence. The raw Alchemy response is still attached for audit, and missing/failed provider responses remain fail-closed.

The simulation layer also emits a counterparty trust summary backed by `framkey-simulation::registry`. The current registry is deliberately narrow and chain-aware for the wallet's switchable chains: Ethereum, Sepolia, Base, OP Mainnet, Arbitrum One, and Polygon. It labels source-backed Uniswap V2 Router02, Uniswap V3 SwapRouter/SwapRouter02, Uniswap Universal Router, Permit2, and Aave V3 Pool entries where those deployments exist in the official Uniswap deployment docs and the Aave address book. Known labels are review context, not a guarantee that the dApp request is safe. Unknown transaction recipients are shown to the user, and unknown active approval spenders/operators block signing.

Transaction signing is available only after trusted approval, policy authorization, and account validation. Policy decisions are serialized as `allowed`, `requires_user_override`, or `blocked`, but the default conservative transaction policy only emits ordinary-signable `allowed` or fail-closed `blocked` outcomes. `blocked` must not reach signer-helper or mock signing. The simulation layer also emits display-only risk, trust, and impact summaries. Risk uses `low`, `caution`, `high`, or `blocked` level plus reason/action fields. Trust labels known counterparties and approval authorities while highlighting unknown ones. Impact summarizes native value, decoded transfers, decoded approvals, live simulation presence, and whether provider asset changes were attached. Those summaries are review UX only; signer access still depends on policy and decision-token validation.
