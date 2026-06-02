# FRAMKey

FRAMKey is a cartridge-based EOA wallet vault prototype.

The intended v0 path is:

- GBA save/FRAM area as a removable encrypted vault.
- macOS Keychain + Touch ID as the daily local unlock gate.
- Rust CLI and short-lived signer helper before GUI work.
- Tauri DeFi Browser as the next product surface.
- Chrome/Chromium extension as a later EIP-1193/EIP-6963 daily-browser entry point.
- Cloud folders as encrypted durability backups, not as sufficient recovery material.

This is not a hardware wallet security model. A GBA cartridge is storage, not a secure signing element. For the EOA MVP, the wallet secret must enter Mac process memory briefly during signing. The project goal is to make that window narrow and isolated, not to claim cold-wallet guarantees.

## Workspace

```text
crates/
  framkey-core              shared IDs, errors, wallet types
  framkey-crypto            secret containers and encrypted box metadata
  framkey-device            cartridge/save-image device abstraction
  framkey-gbxcart           GBxCart boundary, protocol pending
  framkey-vault             vault and save-image format types
  framkey-recovery          grouped recovery policy model
  framkey-ipc               JSON-RPC-like IPC and Chrome native-message framing
  framkey-evm               EVM address/chain/signing boundary types
  framkey-keychain-macos    macOS Keychain KEK wrapper
  framkey-native-host       Chrome native messaging host
  framkey-simulation        transaction simulation client and policy gate
  framkey-signer-helper     short-lived signer helper
  framkey-cli               development CLI
  framkey-testkit           in-memory device test support
apps/framkey-desktop/       Tauri DeFi Browser foundation
extension/chrome/           development Chrome extension
docs/                       security and format notes
```

## Current Development Slice

The project route is now Tauri-first: build the FRAMKey DeFi Browser and trusted approval UI before expanding the Chrome extension beyond read-only bridge work. The Chrome/Brave extension remains a long-term frontend, but it should reuse the same provider, permission, simulation, and signing core after the Tauri path proves the model.

The current prototype still avoids unverified implementations for:

- Broad GBxCart cartridge support beyond the verified GBA save paths listed below.
- Production transaction policy beyond trusted approval, local decoding, Alchemy simulation context, and signer-helper signing.
- Broad remote dApp compatibility beyond the explicit Local Test, Uniswap, and Aave targets.

Those should be added in small verified slices after the hardware, OS, and app trust boundaries are tested. See `docs/product-roadmap.md`.

## Commands

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
cargo run -p framkey-cli -- recovery policy
```

Vault test-image workflow:

```bash
cargo run -p framkey-cli -- vault build-test-image --out framkey-test-vault.sav --generation 1
cargo run -p framkey-cli -- vault inspect-image --path framkey-test-vault.sav
```

The default test image is 64 KiB, matching the validated `gba-sram-fram-512kbit` target for the current modified A88J cartridge. Pass `--image-size` only when building an explicit compatibility fixture.

macOS Keychain encrypted vault workflow:

```bash
cargo build -p framkey-signer-helper
cargo run -p framkey-cli -- vault init-keychain-kek
cargo run -p framkey-cli -- vault build-keychain-encrypted-image --out framkey-keychain-vault.sav --generation 1 --recovery-out-dir recovery-pack
cargo run -p framkey-cli -- vault open-keychain-encrypted-image --path framkey-keychain-vault.sav
```

The Keychain command stores a random 32-byte KEK in a local, non-synchronizing macOS login Keychain generic-password item. FRAMKey does not use entitlement-gated `SecAccessControl` or `kSecUseDataProtectionKeychain`, so a personal local build does not need an Apple Developer Program Team ID, provisioning profile, or Keychain access group. Instead, every KEK store/load first asks LocalAuthentication for Touch ID and stores a hash of the evaluated Touch ID domain state in the KEK blob. If the Touch ID enrollment set changes, the local KEK blob is rejected and the vault must be recovered to rebind this Mac. Creating a replacement vault or recovering a vault resets the local KEK instead of reusing an older Keychain item. Legacy local-auth and SecAccessControl KEK blobs are not accepted. The current default Keychain service is `io.framkey.local-kek`, so old development items under `io.framkey.kek` are ignored unless explicitly configured. You can explicitly rebind an existing current-format KEK to the current local-auth policy when needed:

```bash
cargo run -p framkey-cli -- vault rebind-keychain-kek
```

Rebinding does not decrypt the wallet secret and does not modify the card or save image; it only changes the local Keychain KEK protection policy. For the Keychain vault path, the CLI delegates wallet-secret generation, recovery wrapper generation, decryption, signing, and recovery rewrap to the short-lived `framkey-signer-helper`; the CLI writes encrypted save images and four recovery backup files, then prints public wrapper metadata plus BLAKE3 hashes, never the plaintext KEK, DEK, recovery root key, wallet secret, or recovery share bytes.

When `--recovery-out-dir` is passed, the CLI build command creates four plain-looking backup files in that directory using create-new semantics: `backup-01.dat`, `backup-02.dat`, `backup-03.dat`, and `backup-04.dat`. Each file is a structured recovery bundle containing encrypted vault data plus one recovery share. The desktop create flow treats the selected recovery folder as a parent and writes each new wallet into a fresh `framkey-backup-g<generation>-<backup-set>` child folder, so older recovery packs are not overwritten and do not block a new create. Store `backup-01.dat` in iCloud Drive, `backup-02.dat` in Google Drive, `backup-03.dat` on local physical storage, and `backup-04.dat` away from the main Mac and GBA card. Cloud files alone are intentionally insufficient for recovery.

Recovery rewrap binds an existing recovery-enabled vault image to the current Keychain item without decrypting the wallet secret. Pass either both cloud files plus one physical file, or one local physical plus one remote physical file:

```bash
cargo run -p framkey-cli -- vault recover-keychain-encrypted-image \
  --path recovery-pack/backup-01.dat \
  --out framkey-recovered-vault.sav \
  --recovery-file recovery-pack/backup-01.dat \
  --recovery-file recovery-pack/backup-02.dat \
  --recovery-file recovery-pack/backup-03.dat
```

Signer helper personal-sign smoke workflow:

```bash
cargo build -p framkey-signer-helper
cargo run -p framkey-cli -- signer personal-sign --device file --path framkey-keychain-vault.sav --message "FRAMKey signer helper smoke"
cargo run -p framkey-cli -- signer personal-sign --device gbx-cart --port /dev/cu.usbserial-210 --save-type gba-sram-fram-512kbit --message "FRAMKey signer helper smoke"
```

The helper receives an encrypted save image and a message, loads the Keychain KEK after Touch ID authorization, decrypts the wallet secret inside the helper process, returns a `personal_sign` signature, and exits. The CLI verifies the returned signature by recovering the signer address.

On macOS, Keychain-vault helper requests run as the helper's own process identity instead of being wrapped by `/usr/bin/sandbox-exec`. This keeps the LocalAuthentication Touch ID prompt and Keychain access tied to the real helper process and avoids brittle ad-hoc sandbox identity behavior. The CLI hashes the helper binary before launch and includes that BLAKE3 value in command output. Pin it for local smoke tests with either:

```bash
export FRAMKEY_SIGNER_HELPER_BLAKE3=<helper_blake3>
cargo run -p framkey-cli -- signer personal-sign --device gbx-cart --port /dev/cu.usbserial-210 --save-type gba-sram-fram-512kbit --message "FRAMKey signer helper smoke"
```

or pass `--signer-helper-blake3 <helper_blake3>`. The old `sandbox-exec` wrapper is only available as a hidden experimental development mode. Packaged builds still need real code signing, notarization, and hardened runtime before real funds, but the personal local build path does not require Keychain access group entitlements.

Tauri DeFi Browser foundation:

```bash
cargo build -p framkey-signer-helper
cargo run -p framkey-desktop
```

Debug app bundle workflow:

```bash
cargo build -p framkey-signer-helper
cd apps/framkey-desktop/src-tauri
cargo tauri build --debug --bundles app --no-sign
```

The desktop build prepares `framkey-signer-helper` as a Tauri sidecar from the matching Cargo target directory when the helper has already been built. At runtime the desktop app first looks for the helper next to the desktop executable, then in bundled app resources, and finally uses explicit `~/.framkey/desktop.json` or `FRAMKEY_SIGNER_HELPER` overrides. The trusted Vault Account panel shows helper readiness, location, process-security mode, and hash-pin status without exposing wallet material.

The desktop app defaults to the current modified A88J GBxCart setup: `/dev/cu.usbserial-210`, `gba-sram-fram-512kbit`, chain id `0x1`, Keychain service `io.framkey.local-kek`, and Keychain account `default`. Normal startup opens only the trusted FRAMKey wallet window; the untrusted dApp WebView is opened from the Apps workspace or by explicit development startup/smoke configuration. Home is a wallet status surface for the loaded account, network, assets, and signer readiness; backup creation, placement, and restore live in the Safety workspace. The Home Connect action loads the local vault account into the trusted in-memory account session and refreshes portfolio state; address-only flows such as Refresh Assets, eth_accounts, eth_coinbase, and repeated approved account requests read that cached address without invoking the signer helper, Keychain unlock, or GBA read. Home Disconnect clears that loaded account session, the portfolio snapshot, token-send selection, pending review queue, and current in-memory dApp account grants without deleting Keychain, GBA, backup, watched-token, or transaction-history data. The injected provider supports read-only account/status methods, controlled `personal_sign`, controlled ERC-20 Permit/Permit2 `eth_signTypedData_v4`, and controlled `eth_sendTransaction`: each signing request is captured, the trusted UI approves or rejects it, and only an approved, unexpired, policy-authorized request reaches the signer path. Real Keychain-vault message, Permit, and transaction signing goes through `framkey-signer-helper`; mock mode signs in memory for UI/debug flows. The trusted UI shows an account balance snapshot, RPC Health, and a Portfolio panel when RPC is configured: `eth_chainId`, `eth_blockNumber`, ETH balance, latest block, and nonzero ERC-20 balances are queried through the trusted Alchemy RPC boundary and never exposed to dApp JavaScript. RPC Health shows chain match, latest block, latency, and sanitized errors while keeping the token and endpoint hidden. The trusted Wallet workspace also includes native-token and ERC-20 Send flows restricted to the trusted main window; they validate recipients and decimal amounts, encode either a no-calldata native transfer or ERC-20 `transfer(address,uint256)`, then reuse the same transaction review, signer-helper/mock signing, broadcast, and Activity pipeline as `eth_sendTransaction`. The trusted UI also shows structured review cards for transaction intent, amount, counterparty, policy state, warnings, approvals, transfers, and gas/nonce details. Transaction cards start with backend-generated signing guidance that says whether the request is ready for ordinary approval, requires explicit high-risk confirmation, or cannot be signed, plus the next action such as checking RPC health after simulation failure. They also include backend-generated risk, trust, and impact summaries: risk explains the required approval path and exact reason codes, trust labels known Uni/Aave/Permit2 counterparties across the current switchable chains and calls out unknown transaction or approval authorities, and impact summarizes native value movement, transfers, approvals, and live provider asset-change coverage before the raw details. The trusted Transaction Activity panel keeps a local sanitized history of transaction review, approval, broadcast hash or failure, and receipt status across app restarts; each item can also show sanitized recovery guidance such as checking RPC health, funding native gas, refreshing dApp state, or retrying after pending nonce state settles. Raw JSON remains available behind collapsible debugging panels. Transaction review includes a conservative decoded report for common transfer/approval selectors plus top-level Uniswap V2, Uniswap V3, Universal Router, multicall, and Aave V3 intents. When live Alchemy simulation is explicitly enabled, successful `alchemy_simulateAssetChanges` `result.changes` are normalized into the same approvals/transfers fields before display; decoded token contracts are then enriched with trusted Alchemy token metadata when available so approvals/transfers can show symbols and decimal-adjusted amounts. Typed-data review recognizes common ERC-20 Permit and Permit2 shapes so approved Permit requests can show owner, spender, token, amount, nonce, and deadline context before signing. Live Alchemy simulation permits ordinary transaction approval only when policy blockers are absent; local-only, unknown calldata, unknown active approval authority, and high-risk approval warnings require an explicit high-risk approval; malformed requests and provider failures remain blocked. Unknown typed data, raw `eth_sign`, and `eth_signTransaction` remain captured and blocked without signing. See `docs/tauri-defi-browser.md`.

The trusted desktop UI is organized into Wallet, DeFi, Recovery, and Diagnostics workspaces so normal wallet use, dApp sessions, backup operations, and raw audit logs are not all shown as one console. It has no app-level product header below the native macOS titlebar; the body starts directly with navigation and wallet content. The selected workspace is remembered locally, while Request Review remains visible in every workspace so pending approvals are not hidden. The trusted desktop UI can also create a new Keychain-encrypted vault and recovery pack. The `Create Vault + Backups` control requires an explicit configured-device write confirmation, then calls the short-lived signer helper to generate the wallet secret, recovery wrapper, encrypted save image, and grouped backup files; the desktop process writes four owner-only backup files, then writes the encrypted save image to the configured GBxCart/file device. The Recovery Backup Plan panel shows Cloud 1, Cloud 2, Local 1, and Local 2 placement cards; each generated file embeds the encrypted vault data plus its recovery share, so the user does not need to handle separate `.sav` and `.json` files. The sanitized backup plan and latest rewrap status are restored from owner-only local trusted state after restart, so the user can continue placement without recreating the vault; `Forget Plan` clears only that local UI state, not the generated backup files. Its local placement checklist computes whether checked backup files match the documented recovery policy: both cloud files plus one physical file, or one local physical plus one off-site physical file. Cloud-only placement remains explicitly insufficient. The Recover Vault form first asks the user to choose a recovery method, then shows either three slots for iCloud + Google + one physical backup or two slots for local + off-site physical backups; it does not expose a separate remembered-plan restore path. It does not connect to iCloud Drive or Google Drive APIs. `Recover Keychain Vault` reads encrypted vault data from one selected backup bundle, validates the selected backup files inside the signer helper, and if sufficient rebinds the vault to the current Keychain item and writes the configured vault device without decrypting the wallet secret. Command output contains metadata, paths, and BLAKE3 hashes only. The UI requires an explicit overwrite checkbox because the configured vault device is replaced.

For dApp compatibility testing, the desktop app can open the local test dApp, Uniswap, Aave, or a user-entered `http`/`https` URL in the untrusted WebView. The trusted DeFi Browser panel shows the current target, sanitized URL, origin, load state, and basic reload/back/forward/home controls; those controls only navigate the untrusted WebView and do not grant account access, switch networks, sign, or submit transactions. The trusted UI includes a DeFi Session panel that summarizes wallet/RPC/dApp/review readiness, the current origin grant, latest provider request, latest signature outcome, latest transaction outcome, and the next wallet action. When a pending or failed transaction has recovery guidance, that next action prefers the concrete repair step over the generic `Use dApp` state. The Transaction Activity panel also restores sanitized activity from owner-only local app state, auto-checks receipts for recently broadcast transactions on a bounded interval, and keeps manual `Refresh` and `Refresh Receipts` controls for diagnostics. It also includes a dApp Compatibility panel that summarizes process-local evidence for Local Test, Uniswap, and Aave across provider injection, read RPC, account connection, watched-token requests, Permit typed-data signing, `personal_sign`, and `eth_sendTransaction`. Each target has a `Check` action that opens the dApp and runs a read-only provider probe for injection, `eth_chainId`, `eth_accounts`, and `eth_blockNumber` without requesting account approval, signing, switching networks, or sending transactions. Each target card turns that evidence into a short status and next action, such as `Read-ready`, `Connected, signing untested`, `Signing path proven`, or `Wallet flow reached transaction`, while keeping the raw event details in Diagnostics. The DApp Provider Events panel records provider injection lifecycle, EIP-6963 requests/announcements, and provider request outcomes with origin, method, status, duration, result shape, and sanitized errors. It intentionally does not store raw params, calldata, signatures, RPC URLs, or tokens. The injected provider announces itself through EIP-6963, keeps EIP-1193 account/chain state, emits `connect`, `accountsChanged`, and `chainChanged`, and supports common compatibility aliases such as `enable`, `send`, and `sendAsync` without claiming to be MetaMask. `wallet_addEthereumChain` and `wallet_switchEthereumChain` are trusted-approval gated for known Alchemy-backed chains: Ethereum, Sepolia, Base, OP Mainnet, Arbitrum One, and Polygon. Add-chain requests verify FRAMKey's derived Alchemy endpoint and return success without trusting dApp-supplied RPC URLs, persisting dApp metadata, or silently switching the active session chain. `wallet_watchAsset` is trusted-approval gated for ERC-20 tokens; approved tokens are saved in owner-only local trusted wallet state and shown in Portfolio as watched zero-balance assets when Alchemy has not discovered a nonzero balance. DApp-provided token metadata is display-only and never affects transaction policy or signer access. The watched-token list is restored after restart, but dApp account grants, provider events, compatibility evidence, pending reviews, and dApp session state remain process-local. The Wallet workspace also has a trusted Active Network selector for the same supported chains, so the user can switch the session network before opening or using a DeFi app. Unsupported chain requests, missing Alchemy token, or an Alchemy endpoint that cannot prove the requested `eth_chainId` fail before changing session state. Account exposure is origin-scoped: `eth_accounts` returns an empty array until the dApp is approved through trusted UI, while `eth_requestAccounts`, `wallet_requestPermissions`, `wallet_getPermissions`, and `wallet_revokePermissions` use session-local grants visible in the Connected Sites panel. Signing and transaction requests from dApps also require that account grant first; a connection does not pre-approve any signature or transaction. Read-only Ethereum RPC methods are proxied through the configured Alchemy endpoint so the API key is not exposed to dApp JavaScript. Alchemy is the preferred RPC and transaction-simulation provider for the prototype: in debug builds, a repo `.env` containing only `ALCHEMY_TOKEN=<alchemy-api-key>` is enough to derive the default `eth-mainnet` endpoint and enable live `alchemy_simulateAssetChanges` review. Use `FRAMKEY_RPC_URL`, `FRAMKEY_ALCHEMY_RPC_URL`, `ALCHEMY_RPC_URL`, `FRAMKEY_ALCHEMY_NETWORK`, and `FRAMKEY_RPC_TIMEOUT_MS` to override the read RPC path; use `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only` only for deterministic development/offline smoke. `eth_sendTransaction` is completed with nonce/gas/fee data from Alchemy, reviewed in the trusted UI, then signed and submitted with `eth_sendRawTransaction` only when policy allows ordinary approval or the user chooses the explicit high-risk override for overrideable warnings. The default Keychain-vault path delegates signing to `framkey-signer-helper`; `FRAMKEY_WALLET_MODE=mock_in_memory` enables a process-lifetime mock EOA for UI/dApp flow testing without touching the card or Keychain.

Alchemy simulation smoke workflow:

```dotenv
ALCHEMY_TOKEN=<alchemy-api-key>
```

```bash
cargo run -p framkey-desktop
```

The debug app can read `ALCHEMY_TOKEN` from the shell environment or the repo `.env`, then derives `https://eth-mainnet.g.alchemy.com/v2/<token>` by default and uses the same endpoint for live Alchemy asset-change simulation. Use `FRAMKEY_ALCHEMY_NETWORK`, `FRAMKEY_ALCHEMY_RPC_URL`, `FRAMKEY_SIMULATION_TIMEOUT_MS`, and `FRAMKEY_SIMULATION_DEFAULT_GAS` to override that development default. Set `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only` when you need deterministic local-only smoke. The token or RPC URL is never returned in status output.

Runtime UI smoke workflow:

```bash
FRAMKEY_WALLET_MODE=mock_in_memory \
FRAMKEY_SIMULATION_PROVIDER=local_decoder_only \
FRAMKEY_DESKTOP_AUTOSMOKE=1 \
cargo run -p framkey-desktop
```

`FRAMKEY_DESKTOP_AUTOSMOKE=1` is development-only. It explicitly opens the local dApp WebView, logs Tauri main/dApp window visibility, and lets that local dApp drive account connection, Permit typed-data signing, `personal_sign`, and `eth_sendTransaction` while the trusted UI WebView auto-approves mock-mode review requests. Use it only with mock mode.

Set `FRAMKEY_DESKTOP_WALLET_SEND_AUTOSMOKE=1` alongside mock mode when you also want the trusted Wallet UI to fill and submit the native-token Send form and then the Portfolio ERC-20 Send form. This heavier smoke still goes through the review queue and mock signing path, and the expected result for the unfunded mock account is a sanitized insufficient-funds broadcast failure recorded in Transaction Activity.

Set `FRAMKEY_DESKTOP_RECOVERY_AUTOSMOKE=1` alongside mock mode to generate a disposable development recovery pack without touching Keychain, Touch ID, GBxCart, or the configured vault device. The trusted UI writes the same four backup bundles used by real vault creation, then runs read-only recovery drills: cloud-only backups must fail, while the recommended cloud-plus-physical set must pass. This smoke prints paths and BLAKE3 hashes only, not recovery share bytes or the recovery root key.

Remote dApp startup smoke workflow:

```bash
FRAMKEY_WALLET_MODE=mock_in_memory \
FRAMKEY_RPC_TIMEOUT_MS=30000 \
FRAMKEY_DESKTOP_START_URL=uniswap \
FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE=read \
FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1 \
cargo run -p framkey-desktop
```

Use `FRAMKEY_DESKTOP_START_URL=aave`, `local`, or a full `http`/`https` URL to choose the initial untrusted dApp WebView target. `FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE=read` (or `1`) asks the injected provider to run read-only `eth_chainId`, `eth_accounts`, and `eth_blockNumber` checks after page load. `FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE=interactive` also drives `eth_requestAccounts`, `personal_sign`, Permit2 `eth_signTypedData_v4`, and a minimal `eth_sendTransaction`; pair it with `FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE=1` and `FRAMKEY_WALLET_MODE=mock_in_memory` so the trusted UI approves through the real review broker without touching the card or Keychain. Set `FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE_CHAIN_ID=0xaa36a7`, `0x2105`, or another supported chain id enabled for the Alchemy app to make interactive smoke request `wallet_switchEthereumChain`, verify the switched `eth_chainId`, and then continue the signing/transaction path on that session chain. The app probes the derived chain RPC before mutating session state, so an Alchemy app with that network disabled fails the switch instead of leaving the dApp on a half-switched session. Slow remote pages can set `FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE_DURATION_MS=90000` to keep the mock-only approval loop alive longer. Use `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only` for a deterministic transaction signing smoke while RPC still comes from `.env` Alchemy; otherwise Alchemy simulation is the default when an Alchemy endpoint is configured and unfunded mock transactions can be blocked by policy before signing. The current build has passed interactive remote smoke for both Uniswap and Aave through provider injection, read RPC, account approval, `personal_sign`, Permit signing, and transaction review/signing, ending with the expected mock-account insufficient-funds broadcast error in deterministic local-simulation mode; it has also passed Uniswap multi-chain smoke after a trusted switch to Sepolia. The stderr telemetry stream is development-only and prints sanitized provider lifecycle/request metadata so remote-site compatibility can be checked without WebKit devtools or macOS accessibility capture. It does not print raw params, calldata, signatures, Alchemy token, RPC URL, or vault/recovery secrets.

Browser bridge read-only workflow:

```bash
cargo build -p framkey-native-host -p framkey-signer-helper
```

Load `extension/chrome` from `chrome://extensions` with Developer mode enabled, then register a Chrome Native Messaging manifest named `dev.framkey.native_host` that points to `target/debug/framkey-native-host`. The extension currently supports `eth_chainId`, `eth_accounts`, `eth_requestAccounts`, `framkey_getStatus`, and `wallet_getCapabilities`; signing and transaction methods are explicitly blocked. See `docs/browser-bridge.md`.

Explicit dev/test encrypted vault workflow:

```bash
cargo run -p framkey-cli -- vault generate-dev-kek
export FRAMKEY_DEV_KEK_HEX=<dev_kek_hex>
cargo run -p framkey-cli -- vault build-dev-encrypted-image --out framkey-dev-vault.sav --generation 1
cargo run -p framkey-cli -- vault open-dev-encrypted-image --path framkey-dev-vault.sav
```

The dev KEK path is only for deterministic local plumbing tests. The Keychain wrapper is the default local-machine protection path.

Save-image fixture workflow:

```bash
cargo run -p framkey-cli -- device probe --device file --path save.bin
cargo run -p framkey-cli -- device read-save --device file --path save.bin --out copy.bin
cargo run -p framkey-cli -- device verify-save --device file --path copy.bin --blake3 <hash>
cargo run -p framkey-cli -- device write-save --device file --path save.bin --input copy.bin
```

Native GBxCart GBA save workflow:

```bash
cargo run -p framkey-cli -- device probe --device gbx-cart --port /dev/cu.usbserial-210 --save-type gba-eeprom-64k
cargo run -p framkey-cli -- device read-save --device gbx-cart --port /dev/cu.usbserial-210 --save-type gba-eeprom-64k --out read.sav
cargo run -p framkey-cli -- device write-save --device gbx-cart --port /dev/cu.usbserial-210 --save-type gba-eeprom-64k --input read.sav
cargo run -p framkey-cli -- device read-save --device gbx-cart --port /dev/cu.usbserial-210 --save-type gba-eeprom-64k --out after.sav
cmp read.sav after.sav
```

Supported native save types are currently `gba-eeprom-64k`, `gba-sram-fram-256k`, `gba-sram-fram-512kbit`, and `gba-sram-fram-1mbit`.
The native GBxCart path currently requires an explicit save type. Auto-detecting save type from ROM metadata/database is intentionally deferred.
For the current modified A88J cartridge, `gba-sram-fram-512kbit` is the recommended target because the 64 KiB window is stable and large enough for the FRAMKey vault.
The 1 Mbit path is conservative: if the current 128 KiB read shows mirrored 64 KiB banks, non-mirrored 1 Mbit writes are refused before modifying the cartridge.

## Security Invariants

- Browser extension never stores or handles wallet secrets.
- Native messaging host is a relay/orchestrator, not a signer.
- Desktop/UI should parse and confirm transactions but should not keep long-lived plaintext wallet secrets.
- Signer helper is the only EOA MVP process that may touch the decrypted wallet secret, and it should be short-lived.
- Device layer reads and writes save images; it does not understand wallets.
- Cloud data is encrypted client-side and is not enough to recover by itself.

## Recovery Policy

The default policy model is `2-of-3` recovery groups:

- Cloud group: iCloud + Google Drive, internally `2-of-2`.
- Local physical group: one local physical backup file.
- Remote physical group: one off-site physical backup file.

Cloud alone must not recover a wallet. Recovery requires either cloud plus one physical group, or both physical groups.
