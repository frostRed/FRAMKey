# GBxCart Save Durability

Status: completed

## Goal

Fix the Connect-time `save image magic mismatch` from first principles by replacing the single-header/two-slot save format with a non-compatible Reed-Solomon-protected save image, and by making GBxCart save writes prove byte-stable persistence across the device boundary.

## Scope

- Replace the old A/B slot format with one v2 RS shard set; no v1 compatibility.
- Split the vault payload into data shards, add Reed-Solomon parity shards, store per-shard hashes, and interleave shard bytes across the GBA save image.
- Treat the GBA save image as invalid before helper/LocalAuthentication if the v2 shard set cannot be reconstructed and verified.
- Align AGB cleanup behavior with the upstream FlashGBX boundary: do not send the address-pin release command after AGB SRAM/FRAM operations.
- After GBxCart writes, perform a fresh-session readback verification so write success means more than same-session immediate echo/readback.
- Keep any repair of the currently corrupted card as an explicit operator action, not an automatic parser fallback.

## Invariants

- Do not loosen the core FRAMKey vault payload validation rules.
- Do not preserve v1 save-image compatibility.
- Do not treat Reed-Solomon reconstruction alone as authenticity; reconstructed payloads still need hash and VaultFile validation.
- Do not silently rewrite a configured vault device during Connect/open/sign.
- Do not print or persist wallet secret, KEK, DEK, recovery root key, recovery shares, or plaintext private key material.

## Likely Files

- `crates/framkey-gbxcart/src/transport.rs`
- `crates/framkey-vault/src/save_image.rs`
- `crates/framkey-vault/src/constants.rs`
- `crates/framkey-vault/src/types.rs`
- `crates/framkey-ipc/src/messages.rs`
- `crates/framkey-signer-helper/src/metadata.rs`
- `apps/framkey-desktop/src-tauri/src/signer_runtime.rs`
- `apps/framkey-desktop/src-tauri/src/tests.rs`
- `docs/vault-format.md`
- `Cargo.toml`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-vault`
- `cargo check -p framkey-gbxcart`
- `cargo check -p framkey-desktop`
- `cargo check -p framkey-signer-helper`
- `cargo check -p framkey-cli`
- `cargo nextest run -p framkey-vault`
- `cargo nextest run -p framkey-ipc -p framkey-signer-helper`
- `cargo nextest run -p framkey-desktop read_configured_save_image_rejects_invalid_vault_before_helper`
- `cargo nextest run -p framkey-gbxcart`
- `git diff --check`
- Manual GBxCart read evidence was captured before any card repair write; no card repair write was performed.

## Main Risks

- Reed-Solomon parity can recover bounded byte/shard corruption but cannot recover a cartridge/game ROM rewriting most of the save area.
- v1.3 GBxCart cannot automatically power-cycle the cartridge, so fresh-session verification is still weaker than physical unplug/replug or v1.4 power-cycle verification.
- The current card already has at least a byte-level header corruption; repairing it should require an explicit write after preserving the current readout artifact.

# DeFi and Activity Workspace Productization

Status: completed

## Goal

Make the DeFi and Activity workspaces feel like a consumer wallet dApp cockpit instead of a diagnostic console, especially after a DeFi app requests connection, signatures, token approvals, transactions, or broadcast/receipt follow-up.

This continuation raises the bar from "clearer console" to a consumer-ready DeFi usage flow: start from wallet readiness, choose an app, connect only when the wallet is actually ready, promote the current approval in plain language, and keep low-level provider/debug surfaces in System.

## Scope

- Rework the trusted DeFi tab layout around current app, wallet access, next action, pending approval, and latest outcome.
- Rework the Activity tab around recent outcome, pending receipt state, failed/retry guidance, and transaction history.
- Add a first-screen DeFi journey surface that tells ordinary users what is ready, what to do next, and what FRAMKey will still protect.
- Make Home route users toward the next sensible wallet action instead of exposing device/system terminology first.
- Rewrite primary approval titles, button labels, and badges toward user intent and consequence while preserving access to technical details.
- Keep app launch, connection management, review queue, and recent transaction status visible in the DeFi flow.
- Make review cards lead with user intent and consequence, while moving raw method/params detail behind secondary affordances.
- Keep compatibility checks, provider events, raw command output, and low-level readiness detail in System.

## Invariants

- Do not change signing, transaction, Permit, account-grant, network-switch, or watch-asset authorization policy.
- Do not grant untrusted dApps filesystem, Keychain, GBxCart, signer-helper, recovery, backup, or secret access.
- Do not log or expose raw params, calldata, signatures, RPC URLs, Alchemy tokens, wallet secret, KEK, DEK, RRK, or recovery shares.
- Request Review must remain visible across workspaces so pending approvals are not hidden.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- Runtime UI smoke in mock/local-simulation mode if the app can be started cleanly.

## Main Risks

- Over-simplifying approvals could hide critical risk details; the design must surface action, risk, counterparty, and impact before technical details.
- Sharing transaction outcome information between DeFi and Activity may duplicate content, so DeFi should show the latest actionable outcome while Activity remains the deeper history page.
- The tabs should improve the DeFi user journey without making System diagnostics harder to reach during development.
- Visual polish should not turn into a wholesale app rewrite unless a layout issue blocks the consumer flow.

# Keychain Helper Authorization

Status: completed

## Goal

Make the FRAMKey local KEK item usable by the configured signer helper without repeated login-Keychain prompts, while keeping normal open/sign flows read-only with respect to existing Keychain items.

## Scope

- Use macOS device-owner LocalAuthentication as the only local KEK access policy.
- Store and parse only the current FRAMKey local KEK blob format.
- Add a signer-helper Keychain access probe that does not read the card, pass vault image bytes, or touch wallet-secret material.
- Keep the helper authorization probe out of the primary wallet actions; expose it only as an advanced diagnostics/setup action.
- Add a CLI diagnostic command that binds the login-Keychain ACL partition list to the configured signer-helper `CDHash`.
- Validate Keychain service/account values before invoking Keychain or `/usr/bin/security` boundaries.

## Invariants

- Do not pass KEK, DEK, wallet secret, recovery root key, recovery shares, Keychain blob bytes, or login-Keychain passwords through command-line arguments.
- Do not use `-A`, `unsigned:`, or allow-all-applications ACL settings.
- Do not modify the Keychain item during normal open/sign reads.
- Do not grant untrusted dApps filesystem, Keychain, GBxCart, signer-helper, recovery, vault backup, or secret access.
- Keep signing and transaction approval policy unchanged.

## Likely Files

- `crates/framkey-ipc/src/messages.rs`
- `crates/framkey-signer-helper/src/handler.rs`
- `crates/framkey-keychain-macos/src/platform.rs`
- `crates/framkey-keychain-macos/src/types.rs`
- `crates/framkey-cli/src/args.rs`
- `crates/framkey-cli/src/signer_helper.rs`
- `crates/framkey-cli/src/vault.rs`
- `apps/framkey-desktop/src-tauri/src/signer_runtime.rs`
- `apps/framkey-desktop/src-tauri/src/commands.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/vault-format.md`
- `docs/threat-model.md`

## Verification

- `cargo fmt --all -- --check`
- `node --check apps/framkey-desktop/ui/main.js`
- `node --check extension/chrome/src/service-worker.js`
- `cargo check -p framkey-ipc -p framkey-keychain-macos -p framkey-signer-helper -p framkey-cli -p framkey-desktop -p framkey-native-host`
- Focused Keychain, IPC, signer-helper, native-host, and desktop tests.
- `cargo clippy -p framkey-ipc -p framkey-keychain-macos -p framkey-signer-helper -p framkey-cli -p framkey-desktop -p framkey-native-host --all-targets -- -D warnings`
- Real app Keychain authorization probe with the packaged helper, then real app connect/sign smoke.

## Main Risks

- Ad-hoc debug helper identity is `CDHash`-based, so rebuilding the helper can require reauthorizing the current helper.
- macOS may still require the owner to approve the signer-helper Keychain item from a system prompt; the GUI must initiate and explain that flow.
- Existing local KEK items written with a non-current format must be deleted, recreated, or recovered through an explicit flow.

# Cloud Vault Backup Artifacts

Status: completed

## Goal

Complete the recovery v1 durability loop with compact backup bundle files, so each recovery material includes encrypted vault durability plus one recovery share without asking users to manage separate `.sav` and `.json` artifacts.

## Scope

- During trusted vault creation, write four recovery bundle files named `backup-01.dat` through `backup-04.dat`.
- Each bundle contains encrypted vault data plus one recovery share.
- Recovery-file parsing accepts only the current bundle format for recovery drills and recovery rewrap.
- Keep cloud recovery authorization unchanged: iCloud + Google backup bundles remain insufficient without one physical group, and local plus off-site physical backups remain sufficient.

## Invariants

- Do not store or print wallet secret, plaintext DEK, KEK, RRK, recovery root key, private key, or recovery share bytes in UI state or logs.
- Encrypted vault data inside each bundle is durability material; recovery authorization still comes from the bundle shares and grouped threshold policy.
- Do not change transaction signing policy or dApp permission behavior.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/recovery-policy.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests around recovery pack artifact writing and sanitized persistence.
- JS syntax checks for trusted UI and dApp scripts.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop recovery`
- Runtime UI check that Safety shows Cloud 1, Cloud 2, Local 1, and Local 2 bundle placement.

# ETH DeFi Policy Hardening

Status: completed

## Goal

Move the current ETH/DeFi signing layer from a simulation-assisted prototype toward a safer daily-use boundary by enforcing typed-data semantics, adding protocol-aware transaction policy blockers, improving fee preparation defaults, and constraining untrusted dApp telemetry metadata.

## Scope

- Enforce backend Permit/Permit2 semantic checks before typed-data signing reaches the signer helper.
- Keep unknown typed-data, raw `eth_sign`, and unsupported signing methods blocked.
- Add local protocol semantic blockers for transaction policy where the existing decoder can already identify high-risk or under-specified DeFi intents.
- Prefer EIP-1559 fee defaults where the RPC endpoint supports them, while preserving explicit dApp fee fields and existing fail-closed behavior.
- Schema-whitelist dApp provider telemetry detail fields on the Rust boundary.

## Invariants

- Untrusted dApps must not gain access to trusted commands, Keychain, filesystem, GBxCart, recovery, or signer-helper internals.
- No policy change may allow signing without trusted-window approval and backend authorization.
- Live simulation remains required for ordinary transaction approval; local-only or semantically incomplete DeFi review must not become ordinary-signable.
- Permit signing must bind to the connected wallet, active chain, expected verifying contract semantics, and bounded approval risk.
- Do not log or persist raw params, calldata, signatures, RPC URLs, Alchemy tokens, wallet secret, KEK, DEK, RRK, or recovery shares.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/review/summary.rs`
- `apps/framkey-desktop/src-tauri/src/review/authorization.rs`
- `apps/framkey-desktop/src-tauri/src/review/tests.rs`
- `apps/framkey-desktop/src-tauri/src/provider.rs`
- `apps/framkey-desktop/src-tauri/src/transactions.rs`
- `apps/framkey-desktop/src-tauri/src/config.rs`
- `apps/framkey-desktop/src-tauri/src/tests.rs`
- `apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `crates/framkey-simulation/src/assessment.rs`
- `crates/framkey-simulation/src/decoder.rs`
- `crates/framkey-simulation/src/tests.rs`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `docs/threat-model.md`

## Verification

- `echo $RUSTC_WRAPPER`
- `sccache --show-stats`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-simulation`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-simulation`
- `cargo nextest run -p framkey-desktop`

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --test apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`

## Main Risks

- Over-tightening typed-data semantics may block legitimate dApp Permit flows until the review UI and policy registry know enough about more protocols.
- Protocol intent decoding is intentionally partial; blockers must fail safe without pretending to be a full local EVM simulator.
- EIP-1559 defaults need conservative fallback behavior because some supported RPC endpoints may not expose useful fee history.

# ETH DeFi Protocol Semantics and Execution Reliability

Status: completed

## Goal

Complete the next four ETH DeFi hardening slices in order: Universal Router / Permit2 deep decoding, Aave account-level risk evidence, transaction execution reliability, and counterparty registry productization.

## Scope

- Decode supported Universal Router command inputs deeply enough for local policy to reason about swaps, recipients, payer direction, and Permit2 transfer/permit intent.
- Require or attach Aave account-level risk evidence for borrow, withdraw, and collateral toggle reviews before those flows can leave high-risk review.
- Harden transaction preparation and send behavior around unsupported transaction envelopes, blob fields, access lists, nonce selection, and fee bounds.
- Move protocol counterparty knowledge out of ad-hoc assessment code into a small reusable registry surface.

## Invariants

- Untrusted dApps must not gain filesystem, Keychain, GBxCart, recovery, signer-helper, or trusted command access.
- No transaction or typed-data signing path may bypass trusted-window approval, backend authorization, active-chain checks, or connected-wallet binding.
- DeFi decoding is advisory unless complete enough for policy; malformed, unsupported, or semantically incomplete protocol calls fail closed or remain high risk.
- Live simulation remains required for ordinary transaction approval.
- Do not log or persist RPC URLs, API keys, raw signatures, wallet secrets, KEK, DEK, RRK, recovery shares, or plaintext vault material.
- Do not widen recovery, backup, or Keychain behavior while implementing this DeFi slice.

## Likely Files

- `crates/framkey-simulation/src/decoder.rs`
- `crates/framkey-simulation/src/assessment.rs`
- `crates/framkey-simulation/src/lib.rs`
- `crates/framkey-simulation/src/model.rs`
- `crates/framkey-simulation/src/tests.rs`
- `apps/framkey-desktop/src-tauri/src/transactions.rs`
- `apps/framkey-desktop/src-tauri/src/review/summary.rs`
- `apps/framkey-desktop/src-tauri/src/review/tests.rs`
- `apps/framkey-desktop/src-tauri/src/tests.rs`
- `apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `docs/threat-model.md`

## Verification

- `echo $RUSTC_WRAPPER`
- `sccache --show-stats`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-simulation`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-simulation`
- `cargo nextest run -p framkey-desktop`
- `cargo clippy -p framkey-simulation -p framkey-desktop --all-targets -- -D warnings`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `node --test apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `git diff --check`

## Main Risks

- Universal Router command coverage is version-sensitive; unsupported command IDs must stay visible to policy instead of being treated as safe.
- Aave health-factor RPC evidence can be stale between review and mining; local policy must keep conservative thresholds and still rely on simulation.
- Nonce reservation can reduce duplicate-nonce races locally, but it cannot prevent replacement or pending-pool drift caused outside this app.

# ETH DeFi Review Fixes and Approval UX

Status: completed

## Goal

Fix the concrete DeFi review issues found in static review, then make dApp account-connection approvals discoverable from the trusted UI so remote apps do not spin indefinitely after selecting FRAMKey.

## Scope

- Keep Aave borrow, withdraw, and collateral-disable conservative unless policy has post-transaction health evidence.
- Flag Aave third-party withdraw recipients as high risk.
- Make nonce reservation release correct when multiple local prepared transactions fail out of order.
- Validate Permit/Permit2 typed-data schema, not just `primaryType` and message field names.
- Investigate and fix the trusted approval UI path for `eth_requestAccounts` / `wallet_requestPermissions` pending requests.

## Invariants

- Untrusted dApps must not gain filesystem, Keychain, GBxCart, recovery, signer-helper, vault, or trusted command access.
- No signing or account exposure may bypass trusted-window approval, active origin checks, and backend authorization.
- Current Aave account evidence may block or inform, but must not prove post-transaction safety by itself.
- Permit/Permit2 signing must remain limited to exact known EIP-712 semantics and bounded authority.
- UI fixes must make the pending approval actionable without auto-approving remote origins.

## Likely Files

- `crates/framkey-simulation/src/assessment.rs`
- `crates/framkey-simulation/src/tests.rs`
- `apps/framkey-desktop/src-tauri/src/state.rs`
- `apps/framkey-desktop/src-tauri/src/review/summary.rs`
- `apps/framkey-desktop/src-tauri/src/review/tests.rs`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `apps/framkey-desktop/src-tauri/src/tests.rs`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/threat-model.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-simulation -p framkey-desktop`
- `cargo nextest run -p framkey-simulation -p framkey-desktop`
- `cargo clippy -p framkey-simulation -p framkey-desktop --all-targets -- -D warnings`
- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --test apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `git diff --check`

## Main Risks

- Tightening typed-data schema may block a legitimate dApp variant until explicitly modeled.
- Holding Aave account-changing actions in high-risk review is safer but less convenient until post-state protocol simulation exists.
- Approval UX must avoid training users to approve hidden or unactionable requests.

# ETH DeFi Wallet Picker Icon and Badge UX

Status: completed

## Goal

Make FRAMKey show the product icon in remote dApp wallet pickers and avoid duplicating one pending approval badge across trusted workspace tabs.

## Scope

- Replace the EIP-6963 provider announcement icon with the existing bundled product icon instead of the temporary letter mark.
- Stop rendering pending approval badges on trusted workspace tabs.
- Keep the Apps approval callout and review panel behavior intact so pending approvals remain discoverable.

## Invariants

- Do not change account exposure, signing, permission, or approval semantics.
- Do not add remote asset loads for wallet icons; provider metadata must remain self-contained.
- Pending approvals must remain actionable from the review surface without implying that Home, Apps, Safety, Activity, or System each has separate work.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `apps/framkey-desktop/ui/main.js`

## Verification

- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `node --check apps/framkey-desktop/ui/main.js`
- `node --test apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `git diff --check`

## Main Risks

- Some dApp wallet pickers may cache EIP-6963 provider metadata until the page is refreshed.
- Over-hiding counts would make approvals hard to find, so the in-page approval callout and review count must remain.

# HyperEVM Chain Support Investigation

Status: completed

## Goal

Support HyperEVM as a first-class FRAMKey desktop network for EVM account exposure, trusted network switching, native HYPE send, ERC-20 sends, transaction review, signing, broadcast, activity tracking, and dApp compatibility checks.

## Scope

- Add HyperEVM mainnet chain metadata: chain id `0x3e7`, name `Hyperliquid`, native symbol `HYPE`, official RPC `https://rpc.hyperliquid.xyz/evm`, and explorer links for display.
- Split the current `SupportedAlchemyChain` model into a more general supported-chain model so known non-Alchemy chains can be switched safely without trusting dApp-supplied RPC URLs.
- Keep read RPC proxy, RPC health, nonce/gas/fee preparation, raw transaction broadcast, portfolio refresh, and transaction activity working against the trusted chain endpoint.
- Treat Alchemy-specific token discovery, token metadata, and `alchemy_simulateAssetChanges` as provider capabilities instead of chain requirements.
- Preserve local decoder coverage and policy behavior for HyperEVM transactions when live asset-change simulation is unavailable.
- Update trusted UI labels so native balance/send/review surfaces show `HYPE`, not hardcoded `ETH`, on HyperEVM.
- Keep Chrome native-host bridge support limited to read-only chain/account reporting unless the desktop path proves the chain support first.

## Invariants

- Do not change vault, Keychain, recovery, or signer-helper secret handling.
- Do not allow dApp-provided RPC URLs to become trusted endpoints.
- Do not loosen trusted-window approval, account grant, transaction policy, typed-data policy, or raw signing blockers.
- Do not treat missing Alchemy simulation or token discovery as equivalent to live simulation success.
- Do not log or expose RPC URLs containing tokens, wallet secrets, calldata beyond existing sanitized review paths, or signed raw transactions beyond existing activity policy.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/constants.rs`
- `apps/framkey-desktop/src-tauri/src/chains.rs`
- `apps/framkey-desktop/src-tauri/src/config.rs`
- `apps/framkey-desktop/src-tauri/src/wallet.rs`
- `apps/framkey-desktop/src-tauri/src/transactions.rs`
- `apps/framkey-desktop/src-tauri/src/review/summary.rs`
- `apps/framkey-desktop/src-tauri/src/state.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- Live HyperEVM RPC probe confirmed `eth_chainId` returns `0x3e7`, `eth_feeHistory` succeeds with `latest`, and `alchemy_getTokenBalances` returns method-not-found on the official RPC.
- `node --check apps/framkey-desktop/ui/main.js` passed.
- `cargo fmt --all -- --check` passed.
- `cargo check -p framkey-desktop --tests` passed.
- `cargo check -p framkey-simulation` passed.
- `cargo nextest run -p framkey-desktop hyperevm` passed: 7 tests.
- `cargo nextest run -p framkey-desktop eip1559_fee_history_falls_back_from_pending_to_latest` passed: 1 test.
- `cargo nextest run -p framkey-desktop wallet_assets_queries_alchemy_token_balances_and_metadata` passed: 1 test.
- `cargo nextest run -p framkey-simulation` passed: 36 tests.
- Mock-mode read-only runtime smoke with `FRAMKEY_DESKTOP_CHAIN_ID=0x3e7`, `FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE=read`, and local decoder simulation passed: provider injection completed; `eth_chainId`, `eth_accounts`, and `eth_blockNumber` returned ok through the real desktop/WebView path.

## Main Risks

- HyperEVM's official JSON-RPC supports standard EVM reads and writes but does not expose Alchemy-specific methods; portfolio token discovery and live asset-change simulation need graceful capability fallbacks or another trusted provider.
- The official RPC currently supports only latest-state reads for several methods, so review and portfolio paths should avoid historical-state assumptions.
- HyperEVM has dual small/big block behavior and next-eight-nonces mempool constraints; FRAMKey's pending nonce reservation should be checked against rejected or pruned pending transactions.
- Native HYPE transfers to HyperCore system addresses have chain-specific consequences; initially treat them as ordinary native transfers plus explicit review text only if a later slice adds HyperCore-aware warnings.

# Conservative Uni/Aave Policy And Trusted Token Sends

Status: completed

## Goal

Align the EVM signing surface with FRAMKey's intended positioning as a safe, conservative holder wallet while still supporting the agreed core Uniswap and Aave workflows.

## Scope

- Keep native transfers and trusted ERC-20 transfers as first-class wallet actions.
- Keep Uniswap support for recognized swap/permit paths only when semantics are fully decoded and bounded.
- Require every supported Uniswap swap path, including Universal Router swaps, to carry a short transaction-level deadline.
- Apply the same bounded amount, expiration, and signature-deadline policy to Universal Router embedded Permit2 permit commands that typed-data Permit2 signing already uses.
- Keep Aave support for recognized supply, repay, borrow, withdraw, and collateral toggle paths, but require known pools, signer-owned accounts, bounded semantics, and conservative health-factor evidence for debt/collateral-risk actions.
- Remove transaction high-risk override from the default signing authorization path; unknown or incomplete transaction semantics must block rather than rely on user override.
- Ensure dApp-provided `wallet_watchAsset` metadata remains display-only and cannot determine trusted ERC-20 transfer amount encoding.
- Keep HyperEVM support scoped to trusted RPC, native HYPE transfers, ERC-20 transfer review, and local decode when Alchemy-only capabilities are unavailable.

## Invariants

- Do not loosen origin binding, trusted-window approval, review TTL, signer-helper isolation, account grants, or raw `eth_sign`/`eth_signTransaction` blockers.
- Do not let dApp-supplied RPC URLs, token symbols, token decimals, or images affect trusted signing semantics.
- Do not label current Aave health-factor evidence as post-transaction safety; debt/collateral-risk actions need exact transaction dry-run evidence plus conservative current-account thresholds.
- Do not add broad DeFi compatibility to satisfy Uniswap/Aave; support only named protocol actions with explicit policy.
- Do not allow partially bounded Universal Router semantics to reach signing; missing deadlines or unbounded embedded Permit2 authority must block.

## Likely Files

- `crates/framkey-simulation/src/assessment.rs`
- `crates/framkey-simulation/src/decoder.rs`
- `apps/framkey-desktop/src-tauri/src/review/authorization.rs`
- `apps/framkey-desktop/src-tauri/src/review/summary.rs`
- `apps/framkey-desktop/src-tauri/src/transactions.rs`
- `apps/framkey-desktop/src-tauri/src/wallet.rs`
- `apps/framkey-desktop/src-tauri/src/config.rs`
- `apps/framkey-desktop/src-tauri/src/tests.rs`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `cargo check -p framkey-simulation --tests` (passed)
- `cargo check -p framkey-desktop --tests` (passed)
- Focused nextest for conservative policy, Aave health evidence, Uniswap blockers, and trusted token send decimals (passed)
- Focused nextest for Uni deadline and Universal Router Permit2 bounded-authority blockers (passed)
- `cargo nextest run -p framkey-simulation` (passed)
- `cargo nextest run -p framkey-desktop --no-fail-fast` (passed)
- `node --check apps/framkey-desktop/ui/main.js` (passed)
- `cargo fmt --all -- --check` (passed)
- `git diff --check` (passed)

## Main Risks

- Aave borrow/collateral safety is easy to overstate. This slice uses exact transaction dry-run plus conservative current-account thresholds, but still does not model full post-mining health factor.
- Token metadata can come from dApps, RPC providers, or token contracts; only trusted/provider or contract-returned decimals may encode transfer amounts.
- Removing high-risk override may break existing development smoke paths that expected unfunded mock sends to reach signing under local-only simulation.

# SIWE-Only Personal Sign Policy

Status: completed

## Goal

Align `personal_sign` with FRAMKey's conservative holder-wallet positioning by allowing only structured Sign-In with Ethereum messages and blocking arbitrary message signatures before signer-helper access.

## Scope

- Parse `personal_sign` payloads as EIP-4361/SIWE when possible.
- Permit signing only when the message domain, account, URI, chain id, nonce, issue time, expiration, not-before, and resources satisfy FRAMKey's local policy.
- Keep non-SIWE text and hex messages reviewable for diagnostics but not signable.
- Update remote/local smoke fixtures to use SIWE-shaped messages when they expect signing.
- Keep CLI helper smoke for direct signer-helper plumbing separate from dApp `personal_sign` policy.

## Invariants

- Do not loosen account grants, trusted-window approval, review TTL, raw `eth_sign`, unknown typed-data, or transaction policy.
- Do not allow high-risk override for `personal_sign`.
- Do not rely on dApp UI, dApp origin claims, or raw message text alone as authority.
- Do not persist signed messages, signatures, wallet secrets, or recovery material.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/review/summary.rs`
- `apps/framkey-desktop/src-tauri/src/review/authorization.rs`
- `apps/framkey-desktop/src-tauri/src/provider.rs`
- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `apps/framkey-desktop/src-tauri/src/tests.rs`
- `apps/framkey-desktop/src-tauri/src/review/tests.rs`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/threat-model.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- `cargo check -p framkey-desktop` (passed)
- `cargo nextest run -p framkey-desktop review::` (passed)
- `cargo nextest run -p framkey-desktop personal_sign` (passed)
- `cargo nextest run -p framkey-desktop` (passed)
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js` (passed)
- `node --check apps/framkey-desktop/ui/main.js` (passed)
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs` (passed)
- `cargo fmt -p framkey-desktop` (passed)
- `cargo fmt --all -- --check` (passed)
- `git diff --check` (passed)

## Main Risks

- Some dApps still use arbitrary `personal_sign` for login; they will now fail until they adopt SIWE or a later explicitly-scoped compatibility mode is added.
- SIWE parsing should be strict enough to block replay-prone messages but simple enough to remain auditable without a new dependency.
- Existing mock smoke flows must use a valid SIWE fixture or they will correctly stop at review.
