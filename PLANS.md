# Cloud Vault Backup Artifacts

Status: active

## Goal

Complete the recovery v1 durability loop with plain-looking backup bundle files, so each recovery material includes encrypted vault durability and recovery authorization without asking users to manage separate `.sav` and `.json` artifacts.

## Scope

- During trusted vault creation, write four compact recovery bundle files named `backup-01.dat` through `backup-04.dat`.
- Each bundle contains encrypted vault data plus one recovery share; no separate encrypted-vault `.sav`, manifest file, placement guide, or bare share JSON is generated.
- Recovery-file parsing accepts only the current bundle format for recovery drills and recovery rewrap; old bare share JSON and bare `.sav` backup sources are not supported.
- Keep cloud recovery authorization unchanged: iCloud + Google backup bundles must still be insufficient without one physical group, or without local plus off-site physical.

## Invariants

- Do not store or print wallet secret, plaintext DEK, KEK, RRK, recovery root key, private key, or recovery share bytes in UI state or logs.
- Encrypted vault data inside each bundle is durability material; recovery authorization still comes only from the bundle's recovery share and the grouped threshold policy.
- Do not grant untrusted dApps filesystem, Keychain, GBxCart, signer-helper, recovery, vault backup, or secret access.
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
- JS syntax checks for trusted UI and dApp script.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Visual/runtime QA that Safety shows Cloud 1, Cloud 2, Local 1, and Local 2 bundle placement.
- Recovery UI static/runtime check that restore reads encrypted vault data from a selected bundle and checks selected bundle shares.

Completed verification for four-file backup bundle format:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-recovery`: passed.
- `cargo check -p framkey-cli`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-recovery -p framkey-vault -p framkey-signer-helper`: passed, 22 tests.
- `cargo nextest run -p framkey-desktop recovery`: passed, 11 tests.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- Browser plugin visual check could not run because the plugin blocks local `file://` navigation by policy; static DOM checks confirmed the restore form has the local vault backup input/button and sends `vaultBackupPath` to the backend.

Follow-up fix for create-time `backup-03.dat` collisions:

- Root cause: the desktop app can be rebuilt while the bundled signer-helper sidecar is stale, causing the helper to return an older recovery pack shape that maps multiple members to the same current bundle file name.
- The desktop backend now validates that a recovery pack has exactly four unique target bundle file names before writing anything, and reserves each new backup-set directory atomically so failed or concurrent creates cannot reuse the same path accidentally.
- The debug app bundle must be rebuilt after rebuilding and copying `framkey-signer-helper-aarch64-apple-darwin`.

Follow-up UX correction for bundled backup recovery:

- Recovery should expose one user task: choose the backup files available for this restore.
- The UI must not ask separately for a vault backup file, because every current `.dat` backup bundle already embeds encrypted vault data.
- Restore can derive the vault source from the selected backup file set internally, while the safety check remains the visible gate before writing the Mac/GBA vault.
- Remove the separate safety-check step from restore. Restore itself performs the required recovery validation in the signer helper, then binds this Mac and writes the connected GBA only if the selected backup files are sufficient.
- Restore readiness must follow the real grouped policy, not whether both picker boxes contain files: `backup-01.dat` plus `backup-02.dat` plus either physical backup, or `backup-03.dat` plus `backup-04.dat`.

Follow-up Home wallet UX cleanup: completed

- Home should not duplicate Safety backup/restore flows; backup and recovery placement remain under the Safety workspace.
- Home connect is local wallet loading only. It does not grant dApps account access.
- Home disconnect clears the trusted UI's loaded account, portfolio snapshot, token-send selection, pending review queue, and current in-memory dApp account grants. It does not delete Keychain items, GBA data, backup files, watched-token preferences, or transaction activity history.
- Removed the Home setup-path and backup controls from the DOM/JS/CSS instead of only hiding them.
- Verification: `node --check apps/framkey-desktop/ui/main.js`, `cargo fmt --all -- --check`, and `cargo tauri build --debug --bundles app --no-sign` passed. Restarted rebuilt app as pid `19982`.

Follow-up address-only wallet session boundary:

- Status: completed.
- Goal: address-only operations must never load the Keychain vault, signer helper, or GBA after an account is already connected.
- Scope: audit all desktop `load_account` callers and route read-only account/address consumers through a trusted in-memory account session set only by explicit connect/account-permission approval.
- Invariants: signing, vault creation, restore, and explicit connect may still read the configured vault device; asset/RPC refresh, `eth_accounts`, `eth_coinbase`, already-approved account requests, and transaction review address preparation must use the connected session address when available.
- Likely files: `apps/framkey-desktop/src-tauri/src/main.rs`, `apps/framkey-desktop/ui/main.js`, `PLANS.md`, `README.md`, `docs/tauri-defi-browser.md`.
- Verification: focused Rust tests for account-session address-only behavior, `node --check`, `cargo fmt --all -- --check`, `cargo check -p framkey-desktop`, and a rebuilt debug Tauri app.
- Completed implementation: explicit connect/account-approval now populates a trusted in-memory account session; address-only assets and account queries read only that session; disconnect clears the session, grants, and pending reviews; signing paths require both origin permission and a connected account session.
- Completed verification: `node --check apps/framkey-desktop/ui/main.js`, `cargo fmt --all -- --check`, `cargo check -p framkey-desktop`, focused `cargo test` slices for assets/account/session behavior, and `cargo nextest run -p framkey-desktop` passed.

Follow-up restore scheme-first file slots:

- Status: completed.
- Goal: make restore choose the recovery scheme first, then expose exactly the required backup-file slots for that scheme.
- Scope: replace the two abstract Cloud/Physical restore buckets with explicit scheme cards and file slots in the trusted desktop UI; keep the backend recovery validation and grouped threshold policy unchanged.
- Invariants: cloud-only files remain insufficient; Cloud + Physical requires iCloud, Google, and one physical backup file; Physical Pair requires local plus off-site physical files; restore still validates inside the signer helper before binding this Mac and writing the connected GBA.
- Likely files: `apps/framkey-desktop/ui/index.html`, `apps/framkey-desktop/ui/main.js`, `apps/framkey-desktop/ui/styles.css`, `README.md`, `docs/tauri-defi-browser.md`, `PLANS.md`.
- Verification: `node --check apps/framkey-desktop/ui/main.js`, `cargo fmt --all -- --check`, focused desktop recovery tests, and a rebuilt debug app bundle.
- Completed implementation: restore now starts with two recovery-method cards. Cloud + Physical renders iCloud, Google, and Physical slots; Two Physical Files renders Local and Off-site slots. Slot selection remains UI input help only; restore validation still happens in the signer helper before Mac binding and GBA write.
- Completed verification: `node --check apps/framkey-desktop/ui/main.js`, `cargo fmt --all -- --check`, `cargo nextest run -p framkey-desktop recovery`, `cargo tauri build --debug --bundles app --no-sign`, and Computer Use visual checks for both three-slot and two-slot restore methods passed. Restarted rebuilt app as pid `39468`.

Follow-up remove remembered-plan restore shortcut:

- Status: completed.
- Goal: remove the duplicate restore shortcut that exposes remembered local backup-plan paths as a separate user choice.
- Scope: remove the visible `Use remembered backup plan` restore controls and their dead JS/CSS; keep generated-plan auto-fill after create/placement where it does not add another recovery path to the UI.
- Invariants: the only visible restore input model is method selection plus required file slots; real recovery must still work without any remembered local plan; signer-helper validation remains authoritative.
- Likely files: `apps/framkey-desktop/ui/index.html`, `apps/framkey-desktop/ui/main.js`, `apps/framkey-desktop/ui/styles.css`, `README.md`, `docs/tauri-defi-browser.md`, `PLANS.md`.
- Verification: `node --check apps/framkey-desktop/ui/main.js`, `cargo fmt --all -- --check`, focused desktop recovery tests, rebuilt debug app, and visual check.
- Completed implementation: removed the visible remembered-plan restore details/buttons and deleted their JS listeners, helper functions, and CSS. Restore now exposes only method selection, required file slots, advanced manual paths, and the final write confirmation.
- Completed verification: `node --check apps/framkey-desktop/ui/main.js`, `cargo fmt --all -- --check`, `cargo nextest run -p framkey-desktop recovery`, `cargo tauri build --debug --bundles app --no-sign`, and Computer Use visual check passed. Restarted rebuilt app as pid `99184`.

# Distinct Recovery Group Members

Status: completed

## Goal

Make each generated recovery-share member file independently encoded so Local Physical and Remote Physical are not byte-identical copies inside their `1-of-2` groups, while preserving the documented recovery matrix.

## Scope

- Extend the recovery backup share file schema with member-share metadata.
- Generate distinct member payloads for Cloud, Local Physical, and Remote Physical groups.
- Reconstruct group shares through the member-share layer before the outer `2-of-3` group interpolation.
- Update deterministic recovery-pack tests.
- Document that `1-of-2` physical groups are independently encoded redundant member shares, not duplicated JSON payloads.

## Invariants

- Cloud group remains internally `2-of-2`; iCloud + Google alone must not recover.
- Local and Remote groups remain internally `1-of-2`; either member can satisfy its group, but a single physical group alone must not recover.
- Recovery rewrap must still decrypt only the recovery DEK wrapper and must not decrypt the wallet secret.
- Do not print or persist plaintext RRK, DEK, KEK, wallet secret, private key, or recovery share bytes in logs/UI state.

## Likely Files

- `crates/framkey-recovery/src/lib.rs`
- `crates/framkey-vault/src/lib.rs`
- `crates/framkey-signer-helper/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `docs/recovery-policy.md`
- `README.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo nextest run -p framkey-recovery -p framkey-vault -p framkey-signer-helper`
- `cargo nextest run -p framkey-desktop recovery`

Completed verification:

- `cargo fmt --all -- --check`: passed.
- `cargo nextest run -p framkey-recovery -p framkey-vault -p framkey-signer-helper`: passed, 22 tests.
- `cargo nextest run -p framkey-desktop recovery`: passed, 8 tests.

# Connect Session Authorization Race

Status: completed

## Goal

Make Home Connect and Disconnect behave as a single explicit wallet-session state machine: Connect should show immediate progress, require the real unlock path for a new session, and no stale in-flight connect may repopulate the session after Disconnect.

## Scope

- Add trusted UI pending state for connect/disconnect so repeated clicks cannot enqueue overlapping wallet loads.
- Add backend session sequencing around explicit connect/disconnect so stale connect results are discarded after a newer disconnect or connect intent.
- Keep address-only reads using the connected session after a successful connection.
- Preserve dApp account grants as separate origin-scoped approvals.

## Invariants

- Do not weaken Keychain, Touch ID, signer-helper, GBxCart, or signing policy.
- Disconnect must clear loaded account session, dApp account grants, pending reviews, portfolio snapshot, and send selection, without deleting Keychain, GBA, backups, watched-token preferences, or transaction history.
- Existing connected address-only operations must not reload the vault or invoke Touch ID.
- Real signing requests still require an approved origin grant and connected account session before the signer helper can run.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/main.js`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- Focused Rust tests for connect/disconnect race behavior and address-only session reuse.
- `cargo check -p framkey-desktop`
- Rebuild or run the debug Tauri app when feasible.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `echo ${RUSTC_WRAPPER:-<unset>}`: `sccache`.
- `sccache --show-stats`: available, local cache configured.
- `cargo fmt --all -- --check`: passed.
- `cargo nextest run -p framkey-desktop supersedes`: passed, 2 tests.
- `cargo nextest run -p framkey-desktop repeated_request_accounts_uses_connected_session_without_loading_vault disconnect_account_session_clears_account_state`: passed, 2 tests.
- `cargo nextest run -p framkey-keychain-macos`: passed, 6 tests.
- `cargo check -p framkey-keychain-macos -p framkey-signer-helper -p framkey-desktop`: passed.
- `cargo build -p framkey-signer-helper`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.

## Main Risks

- Holding a state lock across the Touch ID/helper operation could deadlock or block Disconnect, so the backend must reserve intent without holding unrelated locks during I/O.
- Forcing every address-only read to re-authenticate would regress the previous explicit session boundary.
- UI-only debouncing would hide the symptom but leave stale backend writes possible.

## Follow-up: Password Fallback And Connect Latency

Status: completed

### Goal

Restore the prior Touch ID-only prompt behavior and make Connect complete when the wallet account is loaded, without waiting for slow portfolio/RPC refresh.

### Scope

- Remove the explicit LocalAuthentication Touch ID reuse-duration override that changed prompt behavior on this machine.
- Keep backend connect/disconnect sequence guards from the previous fix.
- Start portfolio refresh after connect as a background follow-up instead of part of the Connect critical path.

### Invariants

- Do not reintroduce stale in-flight Connect writes after Disconnect.
- Do not make address-only reads reload the vault.
- Do not hide portfolio/RPC errors; only decouple them from the Connect button lifecycle.

### Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- Focused desktop session tests from the previous fix.
- `cargo check -p framkey-keychain-macos -p framkey-signer-helper -p framkey-desktop`
- Rebuild signer helper and debug Tauri app bundle.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `echo ${RUSTC_WRAPPER:-<unset>}`: `sccache`.
- `sccache --show-stats`: available, local cache configured.
- `cargo nextest run -p framkey-desktop supersedes`: passed, 2 tests.
- `cargo nextest run -p framkey-desktop repeated_request_accounts_uses_connected_session_without_loading_vault disconnect_account_session_clears_account_state`: passed, 2 tests.
- `cargo nextest run -p framkey-keychain-macos`: passed, 6 tests.
- `cargo check -p framkey-keychain-macos -p framkey-signer-helper -p framkey-desktop`: passed.
- `cargo build -p framkey-signer-helper`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- `rg -n "touch_id_authentication_allowable_reuse_duration|allowable_reuse_duration" ...`: no matches in the touched runtime files.

## Follow-up: Visible Connect Failure State

Status: completed

### Goal

Make Home Connect show the real failure reason when the wallet cannot reach Touch ID, especially macOS biometry lockout, instead of returning to an indistinguishable disconnected state.
Also fix the Keychain KEK load/rebind sequencing so existing items are authorized with their stored policy instead of assuming the old biometry-only policy before the blob header is read.
Keep the desktop UI from remaining indefinitely in `Connecting` when macOS SecurityAgent/helper authentication does not return.

### Scope

- Preserve the backend connect/disconnect sequence guard.
- Keep Touch ID-only LocalAuthentication policy unchanged.
- Surface provider-envelope errors from `framkey_getAccount` in the Home wallet card.
- Add specific wording for macOS `Biometry is locked out` so the user knows this is an OS Touch ID state, not a FRAMKey no-op.
- Make `load_existing_kek`, `ensure_kek`, and `rebind_kek` read the stored KEK policy before asking LocalAuthentication to authorize that item.
- Add a bounded wait for desktop signer-helper calls that can be blocked inside macOS LocalAuthentication.

### Invariants

- Do not fall back to storing or unlocking wallet material with a password.
- Do not hide the JSON diagnostics output.
- Do not change dApp account grant or signing policy.
- Do not parse or migrate a KEK blob with a target policy hash before proving access under the stored policy.
- Do not let a hidden or stalled macOS auth agent hold the Connect button forever.

### Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- Focused desktop session tests from the previous fix.
- Focused Keychain crate tests.
- `cargo check -p framkey-desktop`
- Rebuild debug Tauri app bundle.
- Manual desktop Connect observation.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo nextest run -p framkey-keychain-macos`: passed, 6 tests.
- `cargo nextest run -p framkey-desktop supersedes`: passed, 2 tests.
- `cargo nextest run -p framkey-desktop repeated_request_accounts_uses_connected_session_without_loading_vault disconnect_account_session_clears_account_state`: passed, 2 tests.
- `cargo check -p framkey-keychain-macos -p framkey-signer-helper -p framkey-desktop`: passed.
- `cargo check -p framkey-desktop`: passed after UI timeout wording change.
- `cargo build -p framkey-signer-helper`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- Manual desktop Connect observation on PID `60224`: while macOS SecurityAgent/helper did not return, Connect stayed disabled during pending, then failed after 45 seconds with `signer helper timed out after 45000 ms waiting for macOS LocalAuthentication`; Home showed a connection failure and the Connect button became usable again.

## Follow-up: Recover Biometry Lockout Without Weakening Final Unlock

Status: completed

### Goal

Make Connect recover from macOS `LAErrorBiometryLockout` for FRAMKey's biometry-only KEK policy without changing the final KEK unlock from Touch ID to password fallback.

### Scope

- Keep `LocalBiometryCurrentSet` as the default signer-helper and CLI policy.
- On `BiometryLockout`, use `DeviceOwnerAuthentication` only to clear the macOS lockout state, then retry the original biometry-only authorization.
- Ensure final KEK authorization and domain-state hash still come from the original stored policy.
- Restore the invariant that the Keychain KEK blob is not loaded before an authorization attempt.
- Keep the desktop pending/error UI and bounded helper timeout.

### Invariants

- Password/device-owner recovery must not by itself authorize KEK use.
- If the final biometry-only retry still fails, the KEK remains unavailable.
- No wallet secret, KEK, or recovery material is logged.

### Verification

- Swift `canEvaluatePolicy` diagnostic for current machine state.
- Focused Keychain tests.
- Focused desktop connect/session tests.
- `cargo check -p framkey-keychain-macos -p framkey-signer-helper -p framkey-desktop`.
- Rebuild debug Tauri app bundle.

Completed verification:

- Swift `canEvaluatePolicy` before fix exercise: `biometryOnly` returned `com.apple.LocalAuthentication code=-8`, while `deviceOwner` returned `ok=true`.
- `cargo nextest run -p framkey-keychain-macos`: passed, 6 tests.
- `cargo fmt --all -- --check`: passed.
- `cargo nextest run -p framkey-desktop supersedes`: passed, 2 tests.
- `cargo nextest run -p framkey-desktop repeated_request_accounts_uses_connected_session_without_loading_vault disconnect_account_session_clears_account_state`: passed, 2 tests.
- `cargo check -p framkey-keychain-macos -p framkey-signer-helper -p framkey-desktop`: passed.
- `cargo build -p framkey-signer-helper`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- Manual desktop Connect observation on PID `67935`: `framkey_getAccount` completed with result and Home connected as `0xceb255...4d906c`.
- Swift `canEvaluatePolicy` after successful Connect: `biometryOnly` returned `ok=true`, `deviceOwner` returned `ok=true`.

# Workspace Rust Module Boundaries

Status: completed

## Goal

Turn the current mostly single-file Rust workspace into a modular, reviewable Rust repository by splitting large app/crate files along stable runtime responsibilities while preserving behavior, public APIs, and the Tauri desktop product flow.

## Scope

- Modularize the large desktop backend file into focused modules for configuration, state/persistence, provider/dApp handling, RPC/chain management, wallet actions, recovery, signer-helper execution, and file/system helpers.
- Modularize the larger library crates around domain boundaries: EVM signing/typed-data/transaction encoding, vault save-image/keychain/recovery flows, recovery policy/bundle/share reconstruction, simulation client/decoder/policy/risk/trust/impact, CLI command families, native host, GBxCart, IPC, Keychain, and signer helper.
- Keep small crates simple when extra files would only add navigation overhead.
- Use mechanical extraction plus minimal visibility changes, not behavioral rewrites.

## Invariants

- Do not weaken Keychain, Touch ID, signer-helper, GBxCart, dApp permission, transaction policy, or recovery validation behavior.
- Do not print or persist wallet secrets, KEKs, DEKs, RRKs, recovery-root keys, recovery share bytes, private keys, raw signatures, raw calldata beyond existing sanitized behavior, Alchemy tokens, or RPC URLs.
- Keep recovery durability backup files distinct from recovery authorization shares.
- Preserve documented four-file backup bundle behavior and the grouped recovery policy.
- Preserve public crate APIs unless a local private boundary can be tightened without breaking callers.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- New files under `apps/framkey-desktop/src-tauri/src/`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `crates/*/src/*.rs`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- Narrow package checks after each risky extraction.
- `cargo check --workspace`
- Focused nextest slices for recovery, desktop review/session, EVM, vault, and simulation behavior where available.
- `cargo nextest run --workspace` if the narrow checks stay clean.

## Main Risks

- Over-fragmenting files can make the repo harder to navigate, so modules should be named by stable responsibilities rather than generic buckets.
- Moving private helpers across sibling modules can accidentally widen API surface; prefer private submodules and `pub(crate)` only where cross-boundary use is real.
- The desktop backend has many coupled state transitions; extraction must not hold locks differently, reorder side effects, or blur trusted/untrusted boundaries.

## Completed Verification

- `cargo check -p framkey-core -p framkey-crypto -p framkey-device -p framkey-testkit`: passed.
- `cargo check -p framkey-evm` and `cargo nextest run -p framkey-evm`: passed, 8 tests.
- `cargo check -p framkey-recovery` and `cargo nextest run -p framkey-recovery`: passed, 6 tests.
- `cargo check -p framkey-vault` and `cargo nextest run -p framkey-vault`: passed, 7 tests.
- `cargo check -p framkey-simulation` and `cargo nextest run -p framkey-simulation`: passed, 23 tests.
- `cargo check -p framkey-ipc` and `cargo nextest run -p framkey-ipc`: passed, 5 tests.
- `cargo check -p framkey-gbxcart` and `cargo nextest run -p framkey-gbxcart`: passed, 5 tests.
- `cargo check -p framkey-keychain-macos` and `cargo nextest run -p framkey-keychain-macos`: passed, 6 tests.
- `cargo check -p framkey-signer-helper` and `cargo nextest run -p framkey-signer-helper`: passed, 9 tests.
- `cargo check -p framkey-native-host` and `cargo nextest run -p framkey-native-host`: passed, 3 tests.
- `cargo check -p framkey-cli` and `cargo nextest run -p framkey-cli --no-tests pass`: passed, 0 tests.
- `cargo check -p framkey-desktop --tests` and `cargo nextest run -p framkey-desktop`: passed, 101 tests.
- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --tests`: passed.
- `cargo nextest run --workspace`: passed, 177 tests.

# Crate-by-Crate Quality and Security Audit

Status: completed

## Goal

Review and harden the Rust workspace one crate at a time, in local dependency order, until the codebase has no known quality or security issues worth fixing in this pass.

## Scope

- Audit each crate's public API, error handling, parsing boundaries, filesystem/network/device interactions, secret handling, logging/output, and tests.
- Prefer narrow fixes that remove real hazards or make an invariant enforceable.
- Keep existing product flow and recovery/security model intact unless the audit finds a concrete bug.
- Use dependency order so lower-level contracts are clean before higher-level crates build on them.

Initial crate order:

- `framkey-core`
- `framkey-simulation`
- `framkey-crypto`
- `framkey-device`
- `framkey-recovery`
- `framkey-testkit`
- `framkey-evm`
- `framkey-gbxcart`
- `framkey-ipc`
- `framkey-keychain-macos`
- `framkey-vault`
- `framkey-native-host`
- `framkey-signer-helper`
- `framkey-cli`
- `framkey-desktop`

## Invariants

- Do not log or persist wallet secrets, KEKs, DEKs, RRKs, recovery-root keys, recovery share bytes, private keys, raw signatures, Alchemy tokens, RPC URLs, or other sensitive material beyond existing sanitized public metadata.
- Do not weaken Keychain, Touch ID, signer-helper, GBxCart, dApp permission, transaction policy, recovery validation, or owner-only local file behavior.
- Keep encrypted vault backup durability separate from recovery authorization shares.
- Preserve the documented backup policy: cloud-only is insufficient; recovery requires the grouped policy validated by signer-helper/library code.
- Avoid broad refactors, dependency churn, or style-only edits.

## Likely Files

- `crates/*/Cargo.toml`
- `crates/*/src/*.rs`
- `apps/framkey-desktop/src-tauri/Cargo.toml`
- `apps/framkey-desktop/src-tauri/src/*.rs`
- `PLANS.md`

## Verification

- Check `RUSTC_WRAPPER` and `sccache --show-stats` before expensive Rust checks.
- For each crate: run `cargo check -p <crate>` first, then `cargo nextest run -p <crate>` where tests exist.
- Use focused tests for changed behavior before broader package checks.
- Run `cargo fmt --all -- --check` after edits.
- Run `cargo check --workspace --tests` and `cargo nextest run --workspace` before marking this audit complete.

## Main Risks

- Security-relevant code can look cleaner while changing trust boundaries; behavior changes must be backed by tests.
- Higher-level crates may rely on permissive lower-level parsing; tightening must be compatible or explicitly migrated.
- Desktop and helper flows involve hardware/Keychain/runtime state, so local automated checks may not cover every real-device path.

## Current Pass 2026-06-02

Status: completed

The user requested another whole-repository quality/security review and refactor pass, one crate at a time, with a broad first-principles scope. Treat the earlier completed audit as useful context, but re-review the current checkout instead of assuming prior results are still sufficient.

Current dependency-order pass:

- `framkey-core`: completed; reviewed current checkout with no new code changes needed. Verification passed: `cargo check -p framkey-core`, `cargo nextest run -p framkey-core`, `cargo clippy -p framkey-core --all-targets -- -D warnings`.
- `framkey-simulation`: completed. Renamed public simulation evidence from `raw_provider_response` to `provider_evidence` with a serde alias for old JSON, kept tests on sanitized evidence, and redacted Alchemy endpoint URLs from config/client Debug output. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-simulation`, `cargo nextest run -p framkey-simulation`, `cargo clippy -p framkey-simulation --all-targets -- -D warnings`, and `cargo nextest run -p framkey-desktop transaction_guidance_marks_live_simulated_request_ready`.
- `framkey-crypto`: completed. Added `AeadBox::decrypt_secret` for fixed-size secret material so callers can avoid leaving copied DEK/KEK/wallet-secret plaintext in ordinary heap buffers longer than needed; the helper wipes the intermediate plaintext vector after copying into `SecretBytes`. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-crypto`, `cargo nextest run -p framkey-crypto`, `cargo clippy -p framkey-crypto --all-targets -- -D warnings`.
- `framkey-device`: completed. Tightened existing Unix save-image files to owner-only `0600` on write, not only on first creation, and added regression coverage for overwriting a broader-permission file. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-device`, `cargo nextest run -p framkey-device`, `cargo clippy -p framkey-device --all-targets -- -D warnings`.
- `framkey-recovery`: completed. Added `reconstruct_recovery_root_key_candidates` so higher layers can try every satisfied 2-of-3 group pair when one supplied recovery group is corrupted, while preserving the existing single-root API. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-recovery`, `cargo nextest run -p framkey-recovery`, `cargo clippy -p framkey-recovery --all-targets -- -D warnings`.
- `framkey-testkit`: completed; current in-memory device behavior reviewed with no code changes needed. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-testkit`, `cargo nextest run -p framkey-testkit`, `cargo clippy -p framkey-testkit --all-targets -- -D warnings`.
- `framkey-evm`: completed. Moved transaction and typed-data validation before private-key/signing-key construction in signing APIs, and added tests that invalid requests fail before invalid private keys are inspected. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-evm`, `cargo nextest run -p framkey-evm`, `cargo clippy -p framkey-evm --all-targets -- -D warnings`.
- `framkey-gbxcart`: completed. Sanitized GBA header label fields from device input so non-printable ROM bytes cannot flow into labels/log-like surfaces as control characters. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-gbxcart`, `cargo nextest run -p framkey-gbxcart`, `cargo clippy -p framkey-gbxcart --all-targets -- -D warnings`.
- `framkey-ipc`: completed. Added redacted Debug implementations for signer-helper IPC requests/responses so save images, recovery files, typed-data JSON, message bytes, calldata, signatures, and raw transactions do not leak through formatting while preserving JSON wire format. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-ipc`, `cargo nextest run -p framkey-ipc`, `cargo clippy -p framkey-ipc --all-targets -- -D warnings`.
- `framkey-keychain-macos`: completed. Tightened Keychain service/account validation to reject leading/trailing whitespace and all control characters, and wiped the temporary KEK blob buffer after storing it in Keychain. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-keychain-macos`, `cargo nextest run -p framkey-keychain-macos`, `cargo clippy -p framkey-keychain-macos --all-targets -- -D warnings`.
- `framkey-vault`: completed. Reworked fixed-size DEK and wallet-secret decrypt paths to use `AeadBox::decrypt_secret`, made recovery rewrap try every satisfied recovery group-pair candidate before failing, aligned vault Keychain item id validation with the stricter Keychain boundary, and redacted recovery backup material from vault image Debug output. Verification passed: `cargo fmt --all`, `cargo check -p framkey-vault`, `cargo nextest run -p framkey-vault`, `cargo clippy -p framkey-vault --all-targets -- -D warnings`.
- `framkey-native-host`: completed. Added an in-process account session so `eth_accounts` only returns already-authorized accounts instead of triggering implicit device reads or Keychain unlocks, redacted local save-image paths from status output, and tightened native-host validation for Keychain names and device hints. Verification passed: `cargo fmt --all`, `cargo check -p framkey-native-host`, `cargo nextest run -p framkey-native-host`, `cargo clippy -p framkey-native-host --all-targets -- -D warnings`.
- `framkey-signer-helper`: completed. Pre-validated malformed `expected_address` values before Keychain unlock on all signing paths while keeping the actual account mismatch check after deriving the vault address, preserving the existing request-size and transaction/typed-data preflight order. Verification passed: `cargo fmt --all`, `cargo check -p framkey-signer-helper`, `cargo nextest run -p framkey-signer-helper`, `cargo clippy -p framkey-signer-helper --all-targets -- -D warnings`.
- `framkey-cli`: completed. Made recovery-backup output directories owner-only, validated recovery pack target names before writing, and cleaned up newly created backup files on mid-write failures while preserving create-new owner-only file writes. Verification passed: `cargo fmt --all`, `cargo check -p framkey-cli`, `cargo nextest run -p framkey-cli`, `cargo clippy -p framkey-cli --all-targets -- -D warnings`.
- `framkey-desktop`: completed. Redacted signer-helper stderr content from desktop runtime error messages so helper failures cannot echo sensitive stderr through provider envelopes or event logs, while preserving status/timeout context; reviewed provider/account/recovery/UI boundaries with no broader structural split needed in this pass. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-desktop`, `cargo nextest run -p framkey-desktop`, `cargo clippy -p framkey-desktop --all-targets -- -D warnings`, `node --check apps/framkey-desktop/ui/main.js`, `node --check apps/framkey-desktop/ui/dapp.js`, `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`, and `node --test apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`.

Additional current-pass invariants:

- Re-check public Debug/Display, CLI/stdout/stderr, app telemetry, persisted state, and JSON responses for secret or token exposure.
- Re-check parsing and validation boundaries before any device, filesystem, Keychain, signer-helper, RPC, or dApp permission side effect.
- Prefer enforcing sequencing in APIs/state machines over relying on callers to remember temporal ordering.

Completed current-pass verification:

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --tests`: passed.
- `cargo nextest run --workspace`: passed, 227 tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `node --test apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.

## Progress

- `framkey-core`: completed. Runtime code reviewed without behavioral changes; added focused tests for core serde wire format and error display. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-core`, `cargo nextest run -p framkey-core`, `cargo clippy -p framkey-core --all-targets -- -D warnings`.
- `framkey-simulation`: completed. Tightened local calldata review by rejecting non-canonical ABI address padding and bool encodings instead of displaying potentially misleading decoded approvals/transfers; changed Alchemy evidence stored in simulation reports from full provider JSON to sanitized provider evidence. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-simulation`, `cargo nextest run -p framkey-simulation`, `cargo clippy -p framkey-simulation --all-targets -- -D warnings`.
- `framkey-crypto`: completed. Removed unused invalid `AeadBox::placeholder` constructor, redacted `AeadBox` debug output, and added focused tests for AEAD tamper rejection, strict hex parsing, and non-leaking secret/sealed-material debug output. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-crypto`, `cargo nextest run -p framkey-crypto`, `cargo clippy -p framkey-crypto --all-targets -- -D warnings`.
- `framkey-device`: completed. Hardened file-image creation to use owner-only `0600` permissions on Unix/macOS and added tests for file permissions plus malformed save-image hashes. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-device`, `cargo nextest run -p framkey-device`, `cargo clippy -p framkey-device --all-targets -- -D warnings`.
- `framkey-recovery`: completed. Replaced secret-bearing derived Debug output with redacted Debug for recovery files, vault-backup bytes, and recovery entropy; added regression coverage and fixed a clippy-reported hex parser style issue. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-recovery`, `cargo nextest run -p framkey-recovery`, `cargo clippy -p framkey-recovery --all-targets -- -D warnings`.
- `framkey-testkit`: completed. Reviewed in-memory device semantics and added a regression test for probe/read/write behavior. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-testkit`, `cargo nextest run -p framkey-testkit`, `cargo clippy -p framkey-testkit --all-targets -- -D warnings`.
- `framkey-evm`: completed. Required `0x`-prefixed EVM addresses, rejected transactions that mix legacy `gasPrice` with EIP-1559 fee fields, exposed a no-secret transaction validation API for signer-helper preflight, and redacted reusable signatures/raw transactions from Debug output. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-evm`, `cargo nextest run -p framkey-evm`, `cargo clippy -p framkey-evm --all-targets -- -D warnings`.
- `framkey-gbxcart`: completed. Rejected empty explicit port hints before serial open and fixed the clippy-reported selected-save-type branch. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-gbxcart`, `cargo nextest run -p framkey-gbxcart`, `cargo clippy -p framkey-gbxcart --all-targets -- -D warnings`.
- `framkey-ipc`: completed. Boxed the large successful signer-helper response variant behind constructor/decoder helpers without changing JSON wire format, migrated direct callers, and added native-message framing boundary tests. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-ipc`, `cargo nextest run -p framkey-ipc`, `cargo clippy -p framkey-ipc --all-targets -- -D warnings`, plus `cargo check -p framkey-signer-helper`, `cargo check -p framkey-native-host`, `cargo check -p framkey-cli`, and `cargo check -p framkey-desktop`.
- `framkey-keychain-macos`: completed. Removed module-inception structure, rejected blank Keychain service/account values, and made LocalAuthentication error wording policy-neutral. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-keychain-macos`, `cargo nextest run -p framkey-keychain-macos`, `cargo clippy -p framkey-keychain-macos --all-targets -- -D warnings`.
- `framkey-vault`: completed. Strengthened vault validation for wrapper bindings and recovery policy consistency, redacted save-image payload previews and vault image debug output, and fixed the clippy-reported layout divisibility check. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-vault`, `cargo nextest run -p framkey-vault`, `cargo clippy -p framkey-vault --all-targets -- -D warnings`.
- `framkey-native-host`: completed. Added a bounded signer-helper wait matching desktop's LocalAuthentication timeout, rejected blank Keychain config names, preserved LocalAuthentication-to-TouchIdFailed IPC classification after error wording cleanup, and fixed clippy-reported config style issues. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-native-host`, `cargo nextest run -p framkey-native-host`, `cargo clippy -p framkey-native-host --all-targets -- -D warnings`.
- `framkey-signer-helper`: completed. Moved typed-data and transaction structure validation ahead of Keychain/Touch ID unlock by reusing EVM parsers, preserved LocalAuthentication error classification, and added focused validation tests. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-signer-helper`, `cargo nextest run -p framkey-signer-helper`, `cargo clippy -p framkey-signer-helper --all-targets -- -D warnings`.
- `framkey-cli`: completed. Centralized output-file creation as create-new owner-only writes for save images and recovery bundles, added a bounded signer-helper wait, improved empty-helper-output errors, and fixed clippy-reported helper hash checking. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-cli`, `cargo nextest run -p framkey-cli`, `cargo clippy -p framkey-cli --all-targets -- -D warnings`.
- `framkey-desktop`: completed. Cleared desktop clippy findings, rejected blank Keychain service/account config at the app boundary, preserved LocalAuthentication cancellation mapping, and kept existing trusted-review/recovery/UI behavior intact. Verification passed: `cargo fmt --all -- --check`, `cargo check -p framkey-desktop`, `cargo nextest run -p framkey-desktop`, `cargo clippy -p framkey-desktop --all-targets -- -D warnings`, `node --check apps/framkey-desktop/ui/main.js`, `node --check apps/framkey-desktop/ui/dapp.js`, and `node --test apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`.

## Completed Verification

- `cargo build -p framkey-signer-helper`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --tests`: passed.
- `cargo nextest run --workspace`: passed, 210 tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `node --test apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `git diff --check`: passed.
