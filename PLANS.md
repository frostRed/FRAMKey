# Keychain Helper Authorization

Status: active

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

Status: active

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
