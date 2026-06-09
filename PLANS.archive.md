# Project Skeleton

Status: completed

## Goal

Create the first reviewable FRAMKey project skeleton: a buildable Rust workspace that reflects the cartridge vault architecture, plus minimal docs and browser-extension placeholders.

## Scope

- Establish top-level workspace metadata and development commands.
- Add early-phase crates for core types, vault layout, device abstraction, GBxCart boundary, recovery policy, IPC, EVM boundary, signer helper, native host, and CLI.
- Keep implementations intentionally thin, focused on stable interfaces and safety boundaries.
- Document the product/security assumptions from the prior design discussion.

## Invariants

- Browser extension, native host, desktop/UI, and signer helper must remain separate trust boundaries.
- Extension/native host must not handle wallet secrets.
- Device layer must not know wallet semantics.
- Vault/recovery data structures must avoid plaintext seed/private key fields.
- Cloud backups are durability only; cloud shares alone must not be sufficient for recovery.

## Likely Files

- `Cargo.toml`
- `README.md`
- `crates/*`
- `extension/chrome/*`
- `docs/threat-model.md`
- `docs/vault-format.md`
- `docs/recovery-policy.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- Narrow crate tests if any are added.

## Risks

- It is easy to overbuild too many crates too early; keep each crate as an ownership boundary only.
- Crypto, Keychain, EVM, and GBxCart protocol code should stay mostly unimplemented until those APIs are verified in small slices.
- Current skeleton should not imply production wallet safety.

# Phase 0 Save Image Baseline

Status: completed

## Goal

Create the first hardware-adjacent development slice: CLI commands and file-backed device support for reading, writing, hashing, and verifying save images before implementing the GBxCart serial protocol.

## Scope

- Add a file-backed `VaultDevice` implementation for save-image fixtures.
- Add stable BLAKE3 save-image fingerprints.
- Add CLI commands for `probe`, `read-save`, `write-save`, and `verify-save`.
- Add sample-directory and GBxCart notes docs.
- Keep real GBxCart reads/writes unimplemented until hardware protocol behavior is verified.

## Invariants

- Device code still does not know wallet semantics.
- CLI save-image commands must operate on opaque bytes.
- Verification output must include a stable hash users can paste back into a check.
- Missing GBxCart protocol implementation should fail explicitly, not silently.

## Likely Files

- `Cargo.toml`
- `crates/framkey-device/src/lib.rs`
- `crates/framkey-cli/src/main.rs`
- `README.md`
- `docs/gbxcart-notes.md`
- `save_image_samples/README.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- CLI smoke test with a temporary save-image fixture.

## Risks

- File-backed device is only a fixture path; it must not be mistaken for hardware confirmation.
- Hash checks prove byte identity, not semantic vault validity.

# Native GBxCart Save Transport

Status: completed

## Goal

Implement the first native Rust GBxCart transport slice so FRAMKey can probe the attached cartridge and read/write the current GBA save image without invoking FlashGBX/Python.

## Scope

- Add serial-port access to `framkey-gbxcart`.
- Support auto-detecting the CH340/CH341 GBxCart serial port, with an explicit `--port` override.
- Probe GBxCart firmware and GBA cartridge header.
- Support the minimal save-type path needed for the current cartridge: explicit GBA EEPROM 64K / 8 KiB save read and write.
- Preserve the existing `VaultDevice` trait boundary so callers still handle opaque save images.
- Update CLI/docs only where needed to expose save type selection and verification workflow.

## Invariants

- Device transport code must not know wallet/vault semantics.
- Writes must fail if the input size does not match the selected save type.
- Hardware write verification remains mandatory at the workflow level: read before write, write, read back, compare hash.
- No firmware update behavior belongs in this slice.

## Likely Files

- `Cargo.toml`
- `crates/framkey-gbxcart/Cargo.toml`
- `crates/framkey-gbxcart/src/lib.rs`
- `crates/framkey-cli/src/main.rs`
- `README.md`
- `docs/gbxcart-notes.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `framkey-cli device probe --device gbx-cart --port /dev/cu.usbserial-210 --save-type gba-eeprom-64k`
- Native read twice -> write same image -> native readback -> BLAKE3/SHA-256 match.
- Hardware artifact: `save_image_samples/20260531-144205-A88J-native-gbxcart/`.

## Risks

- The first native path is intentionally narrow and should not claim broad GB/GBC/GBA cartridge support.
- EEPROM command timing is hardware-sensitive; keep timeouts conservative and validate with repeated reads.
- Existing FlashGBX output identifies this card as A88J with EEPROM64K, but the native CLI should require explicit save type until database matching exists.

# Native GBxCart SRAM/FRAM Save Transport

Status: completed

## Goal

Extend the native Rust GBxCart transport from the verified EEPROM64K path to the standard GBA 256K SRAM/FRAM save class used by many GBA cartridges and likely FRAMKey-style storage targets.

## Scope

- Add an explicit `gba-sram-fram-256k` save type with aliases for common naming.
- Implement native GBA SRAM/FRAM read/write using the existing AGB RAM commands.
- Keep writes size-checked and readback-verified.
- Update docs and tests for the new save type.

## Invariants

- No auto-detection yet; callers must still choose the save type explicitly.
- Do not attempt SRAM/FRAM writes unless the cartridge is known to present a SRAM/FRAM save bus.
- Do not add FLASH save, ROM flashing, or unlicensed 64/128 KiB SRAM bank-switching in this slice.

## Likely Files

- `crates/framkey-gbxcart/src/lib.rs`
- `crates/framkey-cli/src/main.rs`
- `README.md`
- `docs/gbxcart-notes.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- CLI parsing/probe for `--save-type gba-sram-fram-256k` against the attached adapter passed.
- Follow-up hardware validation on the modified A88J cartridge is recorded in the next section.

## Risks

- Stock ROM/database metadata may disagree with the save bus exposed by a modified cartridge; explicit save type selection remains required.

# Native SRAM/FRAM Validation On Modified A88J

Status: completed

## Goal

Validate the native `gba-sram-fram-256k` path against the currently inserted A88J cartridge now that the cartridge is known to be a FRAM-modded card that presents as SRAM/FRAM at the save bus.

## Scope

- Read the 32 KiB SRAM/FRAM image twice with the native Rust transport.
- Check byte stability and record hashes/artifacts.
- If reads are stable, write the same 32 KiB image back and perform native readback verification.
- Keep the previous EEPROM64K artifacts for comparison; do not reinterpret them as authoritative for this modified card.

## Invariants

- Write only a byte-for-byte image that was just read from the same SRAM/FRAM path.
- Preserve all hardware artifacts under `save_image_samples/`.
- Do not introduce save-type auto-detection in this validation slice.

## Likely Files

- `save_image_samples/*/notes.md`
- `PLANS.md`

## Verification

- Native SRAM/FRAM read twice -> compare.
- Native write same image -> internal readback.
- Separate native readback -> compare SHA-256/BLAKE3.
- Hardware artifact: `save_image_samples/20260531-145112-A88J-native-sram-fram/`.

## Risks

- The card may still respond to EEPROM protocol due to ROM/header heritage or board-specific behavior; treat EEPROM and SRAM/FRAM dumps as separate observations until the physical save design is documented.

# FRAM Save Image Vault Smoke Test

Status: completed

## Goal

Write and verify the first FRAMKey-owned 32 KiB save image on the modified A88J SRAM/FRAM cartridge using a minimal two-slot binary test-vault layout.

## Scope

- Add a deterministic binary save-image layout in `framkey-vault`.
- Add CLI commands to build and inspect a non-secret test vault save image.
- Back up the current 32 KiB SRAM/FRAM image before writing.
- Write the test image to the cartridge with native GBxCart transport and verify readback.

## Invariants

- Do not store real wallet secrets in this smoke test.
- The built image must fit exactly in the 32 KiB SRAM/FRAM save area.
- The image must include hashes sufficient to detect payload or active-slot corruption.
- The card write must be byte-for-byte verified after write.

## Likely Files

- `crates/framkey-vault/Cargo.toml`
- `crates/framkey-vault/src/lib.rs`
- `crates/framkey-cli/Cargo.toml`
- `crates/framkey-cli/src/main.rs`
- `docs/vault-format.md`
- `README.md`
- `save_image_samples/*/notes.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- Build test vault image -> inspect -> write to SRAM/FRAM -> read back -> compare -> inspect readback.
- Hardware artifact: `save_image_samples/20260531-145804-framkey-vault-smoke/`.

## Risks

- This intentionally overwrites the cartridge's current SRAM/FRAM contents, so the pre-write backup artifact must be preserved.
- The layout is a prototype and should not be treated as the final encrypted wallet vault format.

# Native 1Mbit SRAM/FRAM Transport

Status: completed

## Goal

Support and validate the full 1 Mbit / 128 KiB FRAM capacity of the modified GBA cartridge instead of only the 256K / 32 KiB SRAM-compatible window.

## Scope

- Add an explicit `gba-sram-fram-1mbit` save type.
- Implement the 128 KiB path as two 64 KiB SRAM banks with AGB bank switching.
- Read the full 128 KiB image twice and compare.
- Compare the native 128 KiB read against FlashGBX `sram1m`.
- Add a write safety guard for mirrored-bank 1 Mbit reads.

## Invariants

- Do not infer 1 Mbit from ROM metadata; require explicit save type.
- Refuse non-mirrored 128 KiB writes when the current cartridge read shows mirrored 64 KiB banks.
- Preserve the current on-card contents under `save_image_samples/`.
- Do not change the vault smoke-test format in this slice.

## Likely Files

- `crates/framkey-gbxcart/src/lib.rs`
- `crates/framkey-cli/src/main.rs`
- `README.md`
- `docs/gbxcart-notes.md`
- `save_image_samples/*/notes.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- Native 128 KiB read twice -> compare.
- FlashGBX v3.37 `sram1m` read -> compare against native read.
- Bank1-only marker write -> observe mirror behavior -> restore original 128 KiB image.
- Guarded bank1-only marker write -> confirm refusal before modification.

## Risks

- The current A88J FRAM mod reads as a stable 128 KiB image, but both native and FlashGBX paths expose mirrored 64 KiB banks. The standard unlicensed 1M SRAM bank-select method does not prove independent 128 KiB capacity on this card.
- A future unsafe or mapper-specific path may be needed if this physical card really has 1 Mbit wired behind a different bank-select scheme.

# Native 512Kbit SRAM/FRAM Target

Status: completed

## Goal

Make the modified A88J cartridge's stable 512 Kbit / 64 KiB SRAM/FRAM window a first-class native save type so FRAMKey can target enough capacity without relying on unverified 1 Mbit bank switching.

## Scope

- Add an explicit `gba-sram-fram-512kbit` save type.
- Implement it as linear 64 KiB SRAM/FRAM read/write.
- Keep the existing 1 Mbit path available but documented as experimental for this cartridge.
- Update docs and sample notes to treat 512 Kbit as the recommended target for the current card.

## Invariants

- Do not infer save type from A88J stock metadata.
- Do not change the vault smoke-test layout in this slice.
- Preserve existing 1 Mbit safety guard behavior.
- Writes still require exact-size images and native readback verification.

## Likely Files

- `crates/framkey-gbxcart/src/lib.rs`
- `crates/framkey-cli/src/main.rs`
- `README.md`
- `docs/gbxcart-notes.md`
- `save_image_samples/*/notes.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-gbxcart`
- `cargo check --workspace`
- `cargo nextest run --workspace`
- Native 64 KiB read from the current card -> compare against first 64 KiB of the earlier 1 Mbit read.
- Native 64 KiB read twice -> write same image -> readback compare.

## Risks

- The card may still physically contain more FRAM, but this slice intentionally targets the proven accessible window rather than capacity discovery.

# Default 64KiB Vault Test Image

Status: completed

## Goal

Move the default FRAMKey hardware smoke-test vault image from 32 KiB to the validated 512 Kbit / 64 KiB SRAM/FRAM window.

## Scope

- Change the vault default save image size constant to 64 KiB.
- Keep the existing `--image-size` override for explicit fixture sizes.
- Update README and vault-format documentation examples.
- Verify the generated default image is 65536 bytes and still inspects correctly.

## Invariants

- Do not change the save image wire format version in this slice.
- Preserve the two-slot layout and payload/header validation behavior.
- Do not write a new default image to hardware unless separately needed.

## Likely Files

- `crates/framkey-vault/src/lib.rs`
- `README.md`
- `docs/vault-format.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-vault`
- `cargo check --workspace`
- `cargo nextest run --workspace`
- CLI build default test image -> inspect -> confirm 65536-byte size.

## Risks

- Existing 32 KiB sample artifacts remain valid historical evidence, but new default smoke-test images will no longer match those old hashes.

# 64KiB Vault Hardware Smoke Test

Status: completed

## Goal

Write and verify the default 64 KiB FRAMKey test vault image on the modified A88J cartridge using the validated `gba-sram-fram-512kbit` native path.

## Scope

- Back up the current 64 KiB SRAM/FRAM window before writing.
- Build the default 64 KiB non-secret test vault image.
- Inspect the generated image before writing.
- Write the image through native GBxCart 512Kbit transport.
- Read back and compare byte-for-byte.
- Record sample artifacts and notes under `save_image_samples/`.

## Invariants

- Do not store real wallet secrets in this smoke test.
- Do not use the unverified 1 Mbit bank-switching path.
- Preserve the pre-write 64 KiB backup artifact.
- The write command's native readback verification must pass, and a separate post-write read must match.

## Likely Files

- `save_image_samples/*/notes.md`
- `PLANS.md`

## Verification

- Build default 64 KiB test image -> inspect.
- Native read backup with `gba-sram-fram-512kbit`.
- Native write default test image -> internal readback.
- Separate native readback -> compare SHA-256/BLAKE3 and inspect.

## Risks

- This intentionally overwrites the current 64 KiB SRAM/FRAM window, so the pre-write backup must be kept.

# Dev/Test Encrypted Vault

Status: completed

## Goal

Build a dev/test encrypted vault path that can generate a test wallet secret, encrypt it into the 64 KiB save image, write/read it through the FRAM cartridge, decrypt it, and verify metadata without using real wallet secrets.

## Scope

- Implement real XChaCha20-Poly1305 AEAD boxes in `framkey-crypto`.
- Add a dev/test encrypted vault payload in `framkey-vault`.
- Wrap the generated data-encryption key with a caller-provided dev KEK.
- Add CLI commands to build and open dev encrypted vault save images.
- Keep the existing non-secret test image commands for hardware smoke tests.
- Run the full file-backed and GBxCart-backed round trip.

## Invariants

- Do not store or print plaintext wallet secrets.
- Mark dev/test wrappers clearly so they cannot be mistaken for production Keychain protection.
- Do not introduce macOS Keychain or signer-helper behavior in this slice.
- Preserve the existing two-slot save image wire format and 64 KiB default.
- Use exact-size write and readback verification through the existing device path.

## Likely Files

- `Cargo.toml`
- `crates/framkey-crypto/Cargo.toml`
- `crates/framkey-crypto/src/lib.rs`
- `crates/framkey-vault/Cargo.toml`
- `crates/framkey-vault/src/lib.rs`
- `crates/framkey-cli/src/main.rs`
- `README.md`
- `docs/vault-format.md`
- `save_image_samples/*/notes.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-crypto`
- `cargo check -p framkey-vault`
- `cargo check --workspace`
- `cargo nextest run --workspace`
- CLI build dev encrypted vault -> open/decrypt -> verify metadata.
- Write dev encrypted vault to `gba-sram-fram-512kbit` -> read back -> open/decrypt -> compare metadata.

## Risks

- The dev KEK is intentionally test-only; this path must not be presented as production Keychain/Touch ID protection.

# macOS Keychain Touch ID KEK Wrapper

Status: completed

## Goal

Replace the default dev KEK vault workflow with a native macOS Keychain-backed KEK wrapper so the test wallet secret can be encrypted, written to the FRAM card, read back, decrypted, and metadata-checked through local machine protection.

## Scope

- Implement the `framkey-keychain-macos` crate using macOS Security.framework generic password items plus LocalAuthentication.
- Generate or load a 32-byte KEK stored in the local Keychain and gated by Touch ID biometry before each load.
- Store a hash of the Touch ID enrollment domain state in the Keychain blob to enforce current-set invalidation.
- Add `mac_keychain` encrypted vault build/open helpers in `framkey-vault`.
- Add CLI commands to initialize, build, and open Keychain-protected encrypted vault save images.
- Keep the dev KEK commands available only as explicit test tooling.
- Run file-backed and, if the OS prompt succeeds, GBxCart-backed write/read/decrypt verification.

## Invariants

- Do not print or log the KEK, DEK, or plaintext wallet secret.
- Device transport must still treat save images as opaque bytes.
- The vault wrapper AAD must bind the DEK wrapper to wallet id, generation, device binding id, and Keychain item id.
- Do not silently weaken the Keychain policy to an unprotected software file if Touch ID/biometry authorization is unavailable.
- Keep the dev/test wrapper visibly marked as dev-only.

## Likely Files

- `Cargo.toml`
- `crates/framkey-keychain-macos/Cargo.toml`
- `crates/framkey-keychain-macos/src/lib.rs`
- `crates/framkey-vault/src/lib.rs`
- `crates/framkey-cli/Cargo.toml`
- `crates/framkey-cli/src/main.rs`
- `README.md`
- `docs/vault-format.md`
- `save_image_samples/*/notes.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-keychain-macos`
- `cargo check -p framkey-vault`
- `cargo check --workspace`
- `cargo nextest run --workspace`
- CLI initialize Keychain KEK -> build Keychain encrypted image -> open/decrypt metadata.
- Write Keychain encrypted image to `gba-sram-fram-512kbit` -> read back -> open/decrypt -> compare metadata.

## Risks

- Plain CLI processes may not be entitled to create `SecAccessControl`-protected generic password items, so this slice enforces Touch ID through LocalAuthentication before Keychain access instead.
- Existing Keychain blobs with a changed Touch ID enrollment hash must be deleted and recreated before they can wrap new vaults.

# Short-Lived EVM Signer Helper MVP

Status: completed

## Goal

Move the responsibility for touching plaintext wallet secret bytes out of the CLI and into the short-lived signer helper, then prove a Keychain-protected FRAM vault can produce a verifiable EVM `personal_sign` signature.

## Scope

- Add minimal EVM secp256k1 address derivation, Ethereum personal-sign hashing, signing, and recovery verification in `framkey-evm`.
- Ensure newly generated EVM vault test secrets are valid secp256k1 private keys.
- Add a helper request/response shape for opening Keychain vault metadata and signing one personal message.
- Implement `framkey-signer-helper` as a one-request process that loads the Keychain KEK, decrypts the wallet secret, signs, zeroizes via existing secret containers, writes JSON, and exits.
- Change the CLI Keychain build path to delegate wallet-secret generation and encryption to the helper.
- Change the CLI Keychain open path to delegate plaintext-secret work to the helper.
- Add a CLI `signer personal-sign` smoke command that reads the card/save image and asks the helper to sign.

## Invariants

- The CLI must not directly decrypt or receive the plaintext wallet secret for Keychain-protected vaults.
- The signer helper must not read or write the cartridge; it receives an encrypted save image and returns only public metadata plus a signature.
- Do not add transaction parsing, transaction broadcasting, typed data, Permit, or extension/native-host wiring in this slice.
- Do not print or log private keys, DEKs, KEKs, or plaintext wallet secrets.
- The helper must handle exactly one request and terminate.

## Likely Files

- `Cargo.toml`
- `crates/framkey-evm/Cargo.toml`
- `crates/framkey-evm/src/lib.rs`
- `crates/framkey-vault/Cargo.toml`
- `crates/framkey-vault/src/lib.rs`
- `crates/framkey-ipc/src/lib.rs`
- `crates/framkey-signer-helper/Cargo.toml`
- `crates/framkey-signer-helper/src/main.rs`
- `crates/framkey-cli/Cargo.toml`
- `crates/framkey-cli/src/main.rs`
- `README.md`
- `docs/threat-model.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-evm`
- `cargo check -p framkey-vault`
- `cargo check -p framkey-signer-helper`
- `cargo check -p framkey-cli`
- `cargo check --workspace`
- `cargo nextest run --workspace`
- Helper open Keychain vault metadata through Touch ID.
- CLI signer smoke against the current `gba-sram-fram-512kbit` card image; verify recovered address matches helper output.

## Risks

- Existing pre-MVP vault images used random 32-byte secrets that were not explicitly validated as secp256k1 scalars, though invalid values are extremely unlikely.
- This creates a working software signer path, but it still lacks transaction parsing, signer code signing, sandboxing, and UI confirmation required before real funds.
- Live macOS Touch ID smoke completed on retry: helper-backed generation 2 vault was written to the `gba-sram-fram-512kbit` card, read back byte-identical, and signed a `personal_sign` message with recovered address matching helper output.

# Signer Helper Local Hardening

Status: completed

## Goal

Close the remaining local signer-helper MVP gaps that can be enforced in this repository before a GUI or packaged app exists: restrict helper capabilities, bound helper inputs, and make helper binary identity visible and optionally pinned.

## Scope

- Run `framkey-signer-helper` under a macOS `sandbox-exec` profile that denies network access by default when launched by the CLI.
- Add an explicit opt-out for unsandboxed local development instead of silently weakening the helper boundary.
- Hash the helper binary before launch, include that identity in CLI output, and allow callers to pin the expected helper hash.
- Bound signer-helper stdin, save-image size, generated image size, and `personal_sign` message size.
- Document the local-hardening behavior and its limits.

## Invariants

- Do not claim this replaces real code signing, notarization, hardened runtime, or a GUI confirmation flow.
- Do not let the helper read/write the cartridge or access network by default.
- Do not print or log private keys, DEKs, KEKs, or plaintext wallet secrets.
- Keep the helper as a one-request process.

## Likely Files

- `crates/framkey-cli/Cargo.toml`
- `crates/framkey-cli/src/main.rs`
- `crates/framkey-ipc/src/lib.rs`
- `crates/framkey-signer-helper/src/main.rs`
- `README.md`
- `docs/threat-model.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-ipc`
- `cargo check -p framkey-signer-helper`
- `cargo check -p framkey-cli`
- `cargo check --workspace`
- `cargo nextest run --workspace`
- `cargo run -p framkey-cli -- signer personal-sign --help`
- Run a helper-backed open/sign smoke with default sandbox if Touch ID authorization is available.

## Risks

- `sandbox-exec` is a local macOS development mechanism, not the final app sandbox story.
- Hash pinning detects replacement only when the expected hash is supplied by the caller or environment.
- Non-interactive checks proved sandboxed helper launch, network denial, and hash mismatch rejection; live sandboxed Keychain signing still needs a Touch ID retry because two authorization attempts timed out.

# Browser Bridge Read-Only MVP

Status: completed

## Goal

Turn the Chrome extension and native host from placeholders into a read-only browser bridge: dApps can discover FRAMKey, request accounts, and receive the FRAMKey EVM address from the card-backed Keychain vault without exposing signing methods to web pages.

## Scope

- Implement an EIP-1193-shaped injected provider with EIP-6963 announcement and provider error handling.
- Relay content-script messages through the extension service worker to Chrome Native Messaging.
- Implement `framkey-native-host` methods for `eth_chainId`, `eth_accounts`, `eth_requestAccounts`, and `framkey_getStatus`.
- Read the configured save image from GBxCart or a file fixture, then delegate Keychain vault opening/address derivation to `framkey-signer-helper`.
- Store per-origin account authorization in extension storage.
- Reject signing and transaction methods explicitly in the extension/native-host path.
- Document dev installation, native host manifest setup, and configuration.

## Invariants

- The browser extension must not touch wallet secrets, Keychain KEKs, DEKs, or decrypted wallet material.
- The native host remains a relay/orchestrator; it must not sign and must not directly decrypt the wallet secret.
- The signer helper remains the only process that may touch decrypted EOA wallet material.
- No transaction signing, typed data signing, or transaction simulation in this slice.
- Native messaging must stay within Chrome's JSON length-prefixed protocol and write diagnostics only to stderr.

## Likely Files

- `crates/framkey-ipc/src/lib.rs`
- `crates/framkey-signer-helper/src/main.rs`
- `crates/framkey-native-host/Cargo.toml`
- `crates/framkey-native-host/src/main.rs`
- `extension/chrome/manifest.json`
- `extension/chrome/src/provider.js`
- `extension/chrome/src/content-script.js`
- `extension/chrome/src/service-worker.js`
- `README.md`
- `docs/browser-bridge.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-ipc`
- `cargo check -p framkey-signer-helper`
- `cargo check -p framkey-native-host`
- `cargo check --workspace`
- `cargo nextest run --workspace`
- `node --check extension/chrome/src/provider.js`
- `node --check extension/chrome/src/content-script.js`
- `node --check extension/chrome/src/service-worker.js`
- Direct native-message smoke for `eth_chainId` and unsupported signing rejection.
- If Touch ID authorization is available, direct native-message smoke for `eth_requestAccounts` against the current GBxCart card.

## Risks

- Chrome extension installation still needs manual Load unpacked and native host manifest registration until packaging work exists.
- `eth_requestAccounts` may trigger Touch ID through the helper because current vault images do not yet rely solely on stored public metadata.
- Direct native messaging smoke completed for `eth_chainId`, `framkey_getStatus`, signing rejection, and `framkey_getAccount` against the current GBxCart card through the sandboxed helper.

# Archived Completed Plan Sections - 2026-05-31

# Tauri-First Product Route

Status: completed

## Goal

Record the product decision to prioritize a Tauri DeFi Browser before expanding the Chrome extension into a signing wallet, while keeping shared provider/signing/simulation core as the long-term architecture.

## Scope

- Document why FRAMKey should not fork Rabby.
- Document the Tauri DeFi Browser as the next product surface.
- Keep the Chrome/Brave extension as the long-term daily-browser frontend.
- Record that both frontends should eventually share provider, permission, simulation, and signing core.
- Identify the next large implementation task after the read-only browser bridge.

## Invariants

- Remote dApp content is untrusted in both Chrome and Tauri.
- No signing without a trusted local approval broker.
- No transaction signing without simulation/transaction summary work.
- Browser extension and dApp WebView must remain secret-free.
- The signer helper remains the only process that may touch decrypted EOA wallet material.

## Likely Files

- `README.md`
- `docs/product-roadmap.md`
- `docs/browser-bridge.md`
- `docs/threat-model.md`
- `PLANS.md`

## Verification

- Documentation review.
- Check that README points to the roadmap and no longer describes Chrome extension work as the immediate product path.

## Risks

- Tauri WebView compatibility may be weaker than Chrome extension compatibility for some dApps.
- Building a DeFi Browser can drift into recreating browser features; v0.2 should stay limited to a short explicit dApp target list and provider/signing path validation.

# Tauri DeFi Browser Foundation

Status: completed

## Goal

Create the first runnable Tauri desktop foundation for FRAMKey: a trusted wallet UI plus an untrusted dApp WebView test surface with injected EIP-1193/EIP-6963 provider, wired only to read-only account/chain/status methods through the existing card, Keychain, and signer-helper stack.

## Scope

- Add a Tauri v2 app crate under `apps/framkey-desktop/src-tauri`.
- Add static trusted UI and local dApp test page without introducing a JS build tool.
- Create two app surfaces:
  - trusted local wallet UI
  - dApp WebView test surface with injected provider
- Implement Tauri commands for `eth_chainId`, `eth_accounts`, `eth_requestAccounts`, `framkey_getStatus`, and `wallet_getCapabilities`.
- Reuse the existing GBxCart/file save-image device path and signer-helper `OpenKeychainVault` request for public address derivation.
- Explicitly reject signing and transaction methods.
- Document how to run the Tauri foundation and the current trust-boundary limits.

## Invariants

- Do not expose signing, transaction submission, typed data, or simulation in this slice.
- The Tauri dApp WebView remains untrusted and must not receive direct Keychain, GBxCart, or signer-helper access.
- The signer helper remains the only process that may touch decrypted EOA wallet material.
- Keep the static UI simple and operational; do not build a full browser or routing system.
- Avoid introducing npm/Vite until the app shell behavior is proven.

## Likely Files

- `Cargo.toml`
- `apps/framkey-desktop/src-tauri/Cargo.toml`
- `apps/framkey-desktop/src-tauri/build.rs`
- `apps/framkey-desktop/src-tauri/tauri.conf.json`
- `apps/framkey-desktop/src-tauri/capabilities/*.json`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/ui/*`
- `README.md`
- `docs/product-roadmap.md`
- `docs/threat-model.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo check --workspace`
- `cargo nextest run --workspace`
- `node --check apps/framkey-desktop/ui/*.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `cargo build -p framkey-desktop`
- `cargo run -p framkey-desktop` startup smoke; stopped with Ctrl-C after the Tauri event loop launched.

## Risks

- Tauri/WebKit runtime behavior can only be fully validated by launching the app, not by Cargo checks.
- The first dApp surface is a local test page; remote dApp compatibility remains a later task.

# Tauri Request Review Pipeline

Status: completed

## Goal

Add the next Tauri DeFi Browser slice: capture dApp signing and transaction requests into a trusted wallet UI review queue, show structured request intent, and continue rejecting those methods until approval and simulation policy are implemented.

## Scope

- Add an in-memory review queue to `framkey-desktop`.
- Route dangerous provider methods through backend capture instead of client-side pre-blocking.
- Keep `personal_sign`, `eth_sendTransaction`, `eth_sign`, and `eth_signTypedData*` blocked after capture.
- Add trusted UI commands to list and dismiss captured review requests.
- Extend the local dApp test page with transaction and typed-data request buttons.
- Document the request-review slice and its no-signing invariant.

## Invariants

- No signing request may reach `framkey-signer-helper` in this slice.
- No transaction request may be simulated, broadcast, or signed in this slice.
- The dApp WebView remains untrusted and only receives provider error/results.
- The review queue is local process memory only; do not add persistence yet.
- Request params are bounded and summarized before display.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/dapp.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/dapp.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/product-roadmap.md`
- `docs/tauri-defi-browser.md`
- `docs/threat-model.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo check --workspace`
- `cargo nextest run --workspace`
- `cargo build -p framkey-desktop`
- `cargo run -p framkey-desktop` startup smoke; stopped with Ctrl-C after the Tauri event loop launched.

## Risks

- This is an approval-pipeline precursor, not approval itself; UI wording must make blocked status obvious.
- Captured params may contain large calldata or typed-data payloads, so display must stay summarized and bounded.

# Tauri Local Approval Broker Policy

Status: completed

## Goal

Add a dry-run local approval broker to the Tauri DeFi Browser: captured dangerous requests gain explicit trusted-UI decisions, TTL expiry, origin/session binding metadata, and one-time decision tokens, while signing, simulation, and broadcasting remain disabled.

## Scope

- Extend review requests with lifecycle status: pending, approved, rejected, expired.
- Add expiry timestamps and automatic expiry during queue reads/decisions.
- Generate per-request decision tokens and consume them on approve/reject to model replay protection.
- Add trusted-window-only Tauri commands for approve/reject/expire decisions.
- Keep provider responses blocked even when a request is later approved in the trusted UI.
- Update trusted UI controls to approve/reject/dismiss requests and show expiry/token state.
- Update docs to describe dry-run approval semantics and no-signing boundary.

## Invariants

- Approval must not invoke `framkey-signer-helper`.
- Approval must not return a signature, simulation, or transaction hash to dApps.
- Only the trusted `main` window may change review decisions.
- Decision tokens are one-time local broker tokens and are not persisted.
- Expired requests cannot be approved later.
- Origin metadata remains bound to each captured request.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/product-roadmap.md`
- `docs/tauri-defi-browser.md`
- `docs/threat-model.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo check --workspace`
- `cargo nextest run --workspace`
- `cargo build -p framkey-desktop`
- `cargo run -p framkey-desktop` startup smoke; stopped with Ctrl-C after the Tauri event loop launched.

## Risks

- A dry-run approval UI can be mistaken for real approval; keep status and docs explicit.
- Tauri command access needs a trusted-window guard because dApp content is untrusted.

# Controlled Tauri Personal Sign

Status: completed

## Goal

Allow the Tauri DeFi Browser to complete exactly one signing path: `personal_sign` requests from the untrusted dApp WebView are captured, shown in the trusted wallet UI, approved with the local broker, signed by `framkey-signer-helper`, and returned to the dApp as an EIP-1193-compatible signature.

## Scope

- Keep `eth_sign`, `eth_signTypedData*`, `eth_sendTransaction`, and `eth_signTransaction` captured and blocked.
- Add a controlled approval wait path for pending `personal_sign` requests.
- Keep the provider request command from blocking trusted-window review queue commands while it waits for approval.
- Parse and bound `personal_sign` params before helper invocation.
- Pass the requested account into the signer helper as an expected address and refuse mismatches before signing.
- Update review queue lifecycle so the trusted UI can distinguish approved, signed, failed, rejected, and expired requests.
- Preserve the signer-helper boundary: desktop and WebView never receive plaintext wallet secret material.

## Invariants

- No signing occurs before a trusted-main-window approval decision.
- Approval tokens remain one-time, process-local, and omitted from provider-visible error data.
- The signer helper is still the only process that can touch decrypted EOA private material.
- A dApp-provided account must match the vault-derived EVM address before the helper signs.
- Transaction signing and typed-data signing remain disabled until simulation and policy checks exist.

## Likely Files

- `crates/framkey-ipc/src/lib.rs`
- `crates/framkey-signer-helper/src/main.rs`
- `crates/framkey-cli/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/threat-model.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo check --workspace`
- `cargo nextest run --workspace`
- If the app builds, launch `cargo run -p framkey-desktop` for a startup smoke.

## Risks

- Waiting for approval inside a provider request can block that specific dApp call until approval, rejection, or expiry; keep the config lock out of the wait path.
- Account mismatch checks must happen in the helper before signing, not only in the desktop broker.
- Returning only the signature keeps dApp compatibility, but UI/debug metadata must stay in the trusted review queue.

# Keychain Local Unlock Binding

Status: completed

## Goal

Fix the macOS Keychain KEK behavior from first principles: the local Keychain KEK is a device-local unlock wrapper, not a recovery root. A Touch ID enrollment change should not appear as an indefinite signer hang, and the default local policy should not brick the current vault before recovery flows exist.

## Scope

- Add an explicit device-owner-authentication Keychain policy that survives Touch ID enrollment changes while still requiring local user authentication.
- Keep the old biometry-current-set policy readable and diagnosable for existing vaults.
- Add an explicit rebind command that migrates an existing local KEK blob to the new policy without changing the KEK, wallet secret, or card vault.
- Classify old biometry-current-set drift as recovery/rebind-required instead of generic Touch ID failure.
- Update docs to describe the security trade-off clearly.

## Invariants

- Rebinding must not decrypt or touch the wallet secret.
- Rebinding must not change the KEK bytes, `kek_id`, Keychain service/account, or vault wrapper binding.
- The signer helper remains the only process that may decrypt wallet secret material.
- No automatic rebind on signing; policy changes must be explicit.

## Likely Files

- `crates/framkey-keychain-macos/src/lib.rs`
- `crates/framkey-signer-helper/src/main.rs`
- `crates/framkey-cli/src/main.rs`
- `README.md`
- `docs/vault-format.md`
- `docs/threat-model.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-keychain-macos`
- `cargo check -p framkey-cli`
- `cargo check -p framkey-signer-helper`
- `cargo nextest run -p framkey-keychain-macos -p framkey-signer-helper -p framkey-cli`
- Manual: run `vault rebind-keychain-kek`, then retry controlled `personal_sign`.

## Risks

- Device-owner authentication is less strict than biometry-current-set invalidation because a changed fingerprint set no longer revokes the local KEK automatically.
- This is acceptable only because recovery/rotation is not yet production-ready; docs must not claim hardware-wallet-grade biometric binding.
- After recovery/rotation is implemented and verified, revisit the default Keychain KEK policy and switch back to strict biometric-enrollment binding, or an equivalent tighter local unlock policy that can be safely recovered from.

# Tauri Transaction Simulation Review

Status: completed

## Goal

Build the first transaction review foundation for the Tauri DeFi Browser: captured `eth_sendTransaction` requests should receive a normalized simulation/decoded-review report in the trusted UI, while transaction signing and broadcasting remain disabled.

## Scope

- Add a reusable Rust simulation/decoding boundary for normalized EVM transaction requests.
- Parse `eth_sendTransaction` params into chain/from/to/value/data/gas fields with bounded, structured output.
- Decode common ERC-20 and ERC-721/1155 approval/transfer selectors conservatively.
- Attach simulation warnings for unknown calldata, malformed calldata, contract calls without a known decode, account/chain mismatches, and native value movement.
- Surface the report in the trusted request-review summary and provider-visible blocked review data.
- Keep third-party simulation API integration behind the model boundary; do not require live API credentials in this slice.
- Update docs to describe the new review layer and its remaining no-signing invariant.

## Invariants

- `eth_sendTransaction` must still return a blocked provider error.
- No transaction may reach the signer helper, be signed, or be broadcast.
- dApp WebViews must not get direct filesystem, Keychain, GBxCart, signer-helper, or simulation-provider credentials.
- The simulation report is advisory until a live third-party simulation client and policy checks exist.
- Unknown or malformed calldata must be explicit in the trusted UI.

## Likely Files

- `Cargo.toml`
- `crates/framkey-simulation/*`
- `apps/framkey-desktop/src-tauri/Cargo.toml`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/dapp.js`
- `README.md`
- `docs/product-roadmap.md`
- `docs/tauri-defi-browser.md`
- `docs/threat-model.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-simulation`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-simulation -p framkey-desktop`
- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- Manual Tauri smoke if the app builds: click dApp `eth_sendTransaction` and confirm the trusted UI shows decoded/simulation data while the dApp receives a blocked error.

## Risks

- Local decoding can be mistaken for full simulation; UI and docs must label it as conservative/advisory.
- Selector-based decoding only covers common methods and must explicitly flag unknown calldata.
- Third-party API response shape is still unchosen, so the model should avoid provider-specific fields except optional raw audit data.

# Simulation Client and Policy Gate

Status: completed

## Goal

Turn the local transaction decoder into a real simulation-client boundary with a fail-closed policy gate, so the Tauri review flow can later swap in third-party simulation APIs without changing dApp/provider semantics.

## Scope

- Add a `framkey-simulation` request/client abstraction around `eth_sendTransaction` simulation.
- Keep the current local decoder as the default offline client.
- Add a transaction policy evaluation object that always fails closed until live simulation and explicit policy checks exist.
- Surface policy blockers separately from decoded warnings in the trusted UI.
- Keep dApp behavior unchanged: transactions are captured for review and return a blocked provider error.
- Update docs to distinguish local decoder, simulation client, and policy gate.

## Invariants

- No transaction signing, transaction serialization, or broadcasting in this slice.
- Policy evaluation must be machine-readable and default-deny.
- Unknown/malformed calldata and missing live simulation must remain visible in trusted UI.
- No simulation-provider credentials or network calls in this slice.
- The dApp WebView receives only provider-compatible blocked errors and bounded review data.

## Likely Files

- `crates/framkey-simulation/src/lib.rs`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/product-roadmap.md`
- `docs/tauri-defi-browser.md`
- `docs/threat-model.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-simulation`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-simulation -p framkey-desktop`
- `node --check apps/framkey-desktop/ui/main.js`
- Manual Tauri smoke: click dApp `eth_sendTransaction`, confirm blocked error remains and trusted UI shows both decoded simulation and policy blockers.

## Risks

- A policy object can look like approval readiness; UI wording must make `blocked` and `canSign: false` obvious.
- The first adapter is offline only, so docs must not imply live risk simulation is complete.

# Alchemy Simulation RPC Adapter

Status: completed

## Goal

Add the first real third-party simulation adapter for the Tauri DeFi Browser using Alchemy Simulation RPC, while keeping local decoding as the default and preserving fail-closed transaction policy.

## Scope

- Add an Alchemy RPC simulation client behind the existing `framkey-simulation` trait.
- Build JSON-RPC `alchemy_simulateAssetChanges` requests from captured `eth_sendTransaction` params.
- Preserve the local decoded transaction report and attach raw provider response for audit.
- Add desktop config and environment variables for enabling Alchemy RPC without hardcoding secrets.
- Redact simulation-provider secrets from status/debug output.
- Keep transaction signing and broadcasting disabled regardless of simulation success.
- Add fixture HTTP tests so adapter behavior is validated without live credentials.
- Update docs with the supported config shape and security boundary.

## Invariants

- No Alchemy URL or access key is hardcoded or logged.
- Default desktop behavior remains offline local decoding only.
- Provider failure, timeout, malformed response, or missing live simulation must remain fail-closed.
- Successful live simulation may remove the `live_simulation_required` blocker but must not enable signing.
- dApp behavior remains a blocked provider error for `eth_sendTransaction`.

## Likely Files

- `Cargo.toml`
- `Cargo.lock`
- `crates/framkey-simulation/Cargo.toml`
- `crates/framkey-simulation/src/lib.rs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `README.md`
- `docs/product-roadmap.md`
- `docs/tauri-defi-browser.md`
- `docs/threat-model.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-simulation`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-simulation -p framkey-desktop`
- `node --check apps/framkey-desktop/ui/main.js`
- Manual Tauri smoke without Alchemy URL: verify local decoder path still blocks transactions.

## Risks

- Alchemy RPC request/response shapes can evolve; keep raw provider response and cite docs.
- HTTP dependencies increase build surface; keep the adapter isolated to the simulation crate.
- A successful simulation is not a signing approval; policy must still include a signing-disabled blocker.

# Recovery Backup Pack Generation

Status: completed

## Goal

Implement the first real recovery-backup generation path for new Keychain vaults: when a wallet vault is generated, the signer helper can add a recovery DEK wrapper and return grouped backup-share files that match the documented 2-of-3 recovery policy.

## Scope

- Extend `framkey-recovery` beyond policy modeling with a serializable backup-pack format.
- Split a random recovery root key into grouped shares:
  - cloud group: 2-of-2 member files
  - local physical group: 1-of-2 member files
  - remote physical group: 1-of-2 member files
- Wrap the vault DEK with the recovery root key using the existing `DekWrapper::Recovery` slot.
- Add optional recovery backup generation to signer-helper keychain vault build IPC.
- Add CLI flags so `vault build-keychain-encrypted-image` can write backup files into a user-selected directory.
- Print only paths and hashes in CLI reports; never print share bytes or recovery key material.
- Update docs to describe where users should place generated files.

## Invariants

- The wallet secret and DEK are not printed or returned to the CLI.
- Cloud backup files alone must not recover the recovery root key.
- Any valid policy combination should reconstruct the recovery root key in tests.
- Existing keychain vault build behavior remains available when recovery backups are not requested.
- Backup files are user-movable artifacts; cloud upload and local/remote physical storage are user actions.

## Likely Files

- `crates/framkey-recovery/src/lib.rs`
- `crates/framkey-recovery/Cargo.toml`
- `crates/framkey-vault/src/lib.rs`
- `crates/framkey-vault/Cargo.toml`
- `crates/framkey-ipc/src/lib.rs`
- `crates/framkey-signer-helper/src/main.rs`
- `crates/framkey-cli/src/main.rs`
- `README.md`
- `docs/recovery-policy.md`
- `docs/vault-format.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-recovery`
- `cargo check -p framkey-vault`
- `cargo check -p framkey-ipc`
- `cargo check -p framkey-signer-helper`
- `cargo check -p framkey-cli`
- `cargo nextest run -p framkey-recovery -p framkey-vault -p framkey-ipc -p framkey-signer-helper -p framkey-cli`
- CLI smoke building a keychain vault with `--recovery-out-dir`, verifying backup files are created and reports contain only paths/hashes.

## Risks

- This is backup generation, not the full recovery/import UX.
- Share files are sensitive recovery material; product copy must tell users where to store each file.
- The grouped sharing implementation is security-sensitive and needs later review before real funds.

# Archived Tauri Desktop Plan Sections

# Tauri DeFi Browser Mock Wallet Slice

Status: completed

## Goal

Move the Tauri app from a local-only dApp test surface toward a usable DeFi browser by adding remote dApp navigation, Uniswap/Aave quick links, read-only EVM RPC proxying through the configured Alchemy endpoint, and a clearly marked in-memory mock wallet mode for UI/debug flows without Touch ID.

## Scope

- Let the trusted UI open the dApp WebView at local test, Uniswap, or Aave URLs.
- Add a compact browser launcher surface with URL entry and curated DeFi shortcuts.
- Add a runtime wallet mode:
  - Keychain vault mode remains the default.
  - Mock in-memory mode is opt-in via config/env and exposes a generated EVM address for app debugging.
- Support common provider read methods needed by DeFi dApps:
  - `net_version`
  - `eth_blockNumber`
  - `eth_getBalance`
  - `eth_call`
  - `eth_estimateGas`
  - `eth_gasPrice`
  - `eth_feeHistory`
  - `eth_getTransactionCount`
- Proxy read-only RPCs to Alchemy without exposing the RPC URL/token to the dApp or trusted UI.
- Keep transaction signing and broadcasting disabled in non-mock production paths.

## Invariants

- Remote dApp content remains untrusted and receives only the injected provider.
- RPC proxy must allowlist read methods; no arbitrary JSON-RPC forwarding.
- Mock wallet mode must be visibly labeled and must not reuse real vault secrets.
- No Alchemy token or URL is returned in `framkey_status`.
- Real transaction signing remains blocked until explicit policy/signing gates are implemented.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- Tauri smoke: open local dApp test, trigger `eth_chainId`, `eth_requestAccounts`, and a read RPC; open Uniswap/Aave shortcut far enough to verify navigation starts and provider injection does not crash.

Completed verification:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 13 tests.
- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `.env` `ALCHEMY_TOKEN` live RPC smoke with `eth_blockNumber`: passed without printing token.
- Tauri app launch smoke: debug `.app` created FRAMKey CGWindows for trusted UI and dApp windows; UI screenshot/click automation was blocked by macOS screen/accessibility capture limits in this session.

## Risks

- Real DeFi sites may have CSP or WebKit compatibility issues that need separate site-specific testing.
- Read-only RPC support improves compatibility but does not make transaction signing production-ready.
- Mock wallet mode is for development only and must remain visually distinct.

# Mock Transaction Signing And Broadcast Slice

Status: completed

## Goal

Move from read-only DeFi compatibility toward normal wallet behavior by letting the Tauri app approve, sign, and broadcast `eth_sendTransaction` in opt-in mock wallet mode, while keeping real Keychain-vault transaction signing blocked until signer-helper transaction support exists.

## Scope

- Add EVM transaction signing primitives for legacy/EIP-155 and basic EIP-1559 raw transactions.
- In desktop mock wallet mode, handle `eth_sendTransaction` through the existing trusted review queue:
  - validate `from`/`chainId` against the configured mock account
  - fill missing nonce, gas limit, and fee fields through the configured Alchemy RPC
  - sign the transaction in memory after trusted UI approval
  - broadcast via `eth_sendRawTransaction`
- Keep `eth_signTransaction`, typed data, raw `eth_sign`, and real Keychain-vault transaction signing blocked.
- Surface signed/broadcast transaction metadata in review execution state without logging secrets or RPC credentials.

## Invariants

- Real vault transaction signing remains disabled in this slice.
- Mock mode must stay visibly marked and must not read card/Keychain secrets.
- dApps cannot call arbitrary RPC or receive the Alchemy URL/token.
- Transaction signing requires a non-expired trusted-UI approval.
- Broadcast errors are returned as provider errors; raw private key material is never logged.

## Likely Files

- `crates/framkey-evm/src/lib.rs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-evm`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-evm`
- `cargo nextest run -p framkey-desktop`
- `node --check apps/framkey-desktop/ui/main.js`
- Live `.env` Alchemy smoke for fee/nonce/broadcast error behavior without exposing token.

Completed verification:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-evm`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-evm`: passed, 5 tests.
- `cargo nextest run -p framkey-desktop`: passed, 15 tests.
- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `.env` Alchemy live smoke: `eth_getTransactionCount`, `eth_gasPrice`, and `eth_estimateGas` returned results; deliberately invalid `eth_sendRawTransaction` returned a provider decode error without exposing token/URL.

## Risks

- Mainnet mock accounts are normally unfunded, so broadcast smoke may correctly fail with insufficient funds after signing.
- Full production support still needs signer-helper transaction signing, stricter transaction policy, and richer UI simulation before real funds.

# Keychain Signer Helper Transaction Signing Slice

Status: completed

## Goal

Enable real Keychain-vault `eth_sendTransaction` signing through the short-lived signer helper so the desktop app can use the same trusted approval and Alchemy broadcast path without touching plaintext wallet secrets in the Tauri process.

## Scope

- Add IPC request/response types for EVM transaction signing.
- Extend `framkey-signer-helper` to decrypt the Keychain vault, validate wallet type/account, sign a prepared EVM transaction, and exit.
- Update desktop `eth_sendTransaction` to share preparation/review/broadcast for mock and Keychain modes:
  - mock mode signs in memory
  - Keychain mode signs through signer-helper
- Keep `eth_signTransaction`, typed data, raw `eth_sign`, and arbitrary RPC forwarding blocked.
- Update docs/status text so transaction signing is no longer described as mock-only.

## Invariants

- Plaintext real wallet secret only appears inside signer-helper.
- Trusted UI approval is required before signer-helper transaction signing.
- Signer helper remains network-denied; desktop process handles RPC completion and broadcast.
- Alchemy token/RPC URL remains hidden from dApp/status.
- Transaction request `from` must match the vault/mock account.

## Likely Files

- `crates/framkey-ipc/src/lib.rs`
- `crates/framkey-signer-helper/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-ipc`
- `cargo check -p framkey-signer-helper`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-ipc`
- `cargo nextest run -p framkey-signer-helper`
- `cargo nextest run -p framkey-desktop`
- Live `.env` Alchemy smoke for nonce/fee/broadcast error behavior without exposing token.

Completed verification:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-ipc`: passed.
- `cargo check -p framkey-signer-helper`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-ipc`: passed, 2 tests.
- `cargo nextest run -p framkey-signer-helper`: passed, 6 tests.
- `cargo nextest run -p framkey-desktop`: passed, 16 tests.
- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `.env` Alchemy live smoke: `eth_getTransactionCount`, `eth_gasPrice`, and `eth_estimateGas` returned results; deliberately invalid `eth_sendRawTransaction` returned a provider decode error without exposing token/URL.

## Risks

- Live real-vault signing requires user Touch ID/password and a real funded account for final chain success; automated tests should not require funds.
- Transaction policy is still conservative review-first plumbing and needs stricter product policy before real funds.

# Tauri Vault Creation Recovery Backup UX

Status: completed

## Goal

Let the trusted desktop app create a new Keychain-encrypted FRAMKey vault, write it to the configured device, and write the required recovery backup pack to a user-selected local directory without exposing wallet secret, DEK, KEK, recovery root key, or recovery share bytes to dApp content or logs.

## Scope

- Add a trusted-main-window Tauri command for vault creation.
- Reuse signer-helper `BuildKeychainVault` with recovery backups enabled.
- Size the new vault image from the configured device target, defaulting to the validated 64 KiB `gba-sram-fram-512kbit` path.
- Write the returned encrypted save image through the existing `VaultDevice` abstraction.
- Write one recovery manifest and six share JSON files using the same file names and no-overwrite behavior as the CLI.
- Return only public metadata, file paths, file hashes, save-image hash, and helper report to the trusted UI.
- Add a compact desktop UI control for generation, recovery output directory, and explicit overwrite confirmation.
- Update docs to make the desktop creation path and backup handling clear.

## Invariants

- Plaintext wallet material remains confined to `framkey-signer-helper`.
- The dApp WebView cannot call the vault creation command.
- Recovery share file contents are never returned in command output.
- Existing CLI recovery pack layout remains the canonical file format.
- Existing vault files and recovery files must not be overwritten silently.
- EEPROM 8 KiB targets are not valid for the current vault format minimum.

## Likely Files

- `apps/framkey-desktop/src-tauri/Cargo.toml`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- `node --check apps/framkey-desktop/ui/main.js`
- Where safe, run a file-device smoke for vault creation into a temporary save image and recovery directory.

Completed verification:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 19 tests.
- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- Real file-device creation smoke was not run because the new path intentionally invokes macOS Keychain/Touch ID and writes a real vault image.

## Risks

- This command can overwrite a configured card save image after explicit confirmation, so the UI and command boundary need a deliberate overwrite gate.
- Recovery backup files are sensitive artifacts even though encrypted/grouped; command output must stay metadata-only.
- Automated tests should not require Touch ID or a real GBxCart device.

# Recovery Keychain Rewrap And Strict Local Policy

Status: completed

## Goal

Complete the first recovery path after backup generation: given enough recovery share files and an existing vault image, bind that vault to the current macOS Keychain item without touching the plaintext wallet secret, then switch new local KEK creation back to the stricter biometry-current-set policy.

## Scope

- Add a vault-core recovery rewrap function that reconstructs the recovery root key, decrypts only the recovery DEK wrapper, and writes a new macOS Keychain DEK wrapper into the save image.
- Add signer-helper IPC for recovery rewrap so DEK handling and Keychain KEK access stay in the short-lived helper.
- Add CLI recovery smoke command using recovery share JSON files.
- Add trusted desktop UI command/control using recovery share file paths and explicit device overwrite confirmation.
- Update docs to explain recovery rewrap, share requirements, and the stricter default Keychain policy.

## Invariants

- Recovery rewrap must not decrypt or expose the wallet secret.
- Recovery share bytes/root key/DEK must not be logged or returned in UI/CLI output.
- Cloud-only shares must still fail recovery.
- The dApp WebView cannot trigger recovery rewrap.
- Existing recovery share file format remains unchanged.
- New local KEKs should use strict biometry-current-set once recovery rewrap is available.

## Likely Files

- `crates/framkey-vault/src/lib.rs`
- `crates/framkey-ipc/src/lib.rs`
- `crates/framkey-signer-helper/src/main.rs`
- `crates/framkey-cli/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/threat-model.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-vault`
- `cargo check -p framkey-ipc`
- `cargo check -p framkey-signer-helper`
- `cargo check -p framkey-cli`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-vault -p framkey-ipc -p framkey-signer-helper -p framkey-cli -p framkey-desktop`
- JS syntax checks for desktop UI/provider files.

Completed verification:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-vault`: passed.
- `cargo check -p framkey-ipc`: passed.
- `cargo check -p framkey-signer-helper`: passed.
- `cargo check -p framkey-cli`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo check -p framkey-ipc -p framkey-signer-helper -p framkey-cli -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-vault`: passed, 7 tests.
- `cargo nextest run -p framkey-vault -p framkey-ipc -p framkey-signer-helper -p framkey-cli -p framkey-desktop`: passed, 36 tests.
- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.

## Risks

- A rewrapped vault changes the encrypted save image; UI and CLI must require explicit output/write targets.
- Strict biometry-current-set is safer but can require recovery after Touch ID enrollment drift, so docs must make recovery path prominent.

# Transaction Policy And Risk Approval

Status: completed

## Goal

Make desktop transaction signing obey the review policy instead of treating every trusted UI approval as signable. Low-risk simulated transactions may proceed with ordinary approval; transactions with overrideable risks require an explicit high-risk approval; non-overrideable failures must not reach signer-helper.

## Scope

- Replace the always-blocked transaction policy result with `allowed`, `requires_user_override`, and `blocked`.
- Mark policy blockers as overrideable or non-overrideable.
- Enforce transaction policy after trusted approval and before any signer-helper/mock signing.
- Add a trusted UI high-risk approval action for overrideable transactions.
- Keep malformed requests and simulation provider failures non-overrideable.
- Update docs/tests to reflect the real policy gate.

## Invariants

- `eth_sendTransaction` must not sign if policy says blocked.
- An ordinary approval must not sign a transaction that requires high-risk override.
- A high-risk override must be visible in review state and still require the one-time decision token.
- Typed data, raw `eth_sign`, and `eth_signTransaction` remain blocked without signing.
- Alchemy token/RPC URL must remain hidden from dApp/status.

## Likely Files

- `crates/framkey-simulation/src/lib.rs`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/threat-model.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-simulation`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-simulation -p framkey-desktop`
- `node --check apps/framkey-desktop/ui/main.js`

Completed verification:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-simulation`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-simulation -p framkey-desktop`: passed, 30 tests.
- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- Nonprinting `.env` Alchemy `eth_chainId` smoke: passed after retrying with a direct HTTPS request.

## Risks

- Too-strict default rules can break real DeFi router transactions; the high-risk override exists for current Uniswap/Aave compatibility while richer decoding is still incomplete.
- Too-loose override rules can normalize dangerous approvals; non-overrideable failures must stay blocked.

# Trusted Review And Account UX

Status: completed

## Goal

Make the trusted desktop UI usable as a wallet confirmation surface instead of requiring the user to inspect raw JSON. The account panel should show a balance snapshot when RPC is configured, and request review cards should surface transaction intent, amount, counterparty, policy state, warnings, and approval action clearly.

## Scope

- Add an account balance row populated through the existing provider/RPC path without exposing RPC credentials.
- Replace the always-visible raw review summary with a structured review synopsis plus collapsible raw details.
- Render transaction policy state, blockers, approvals, transfers, gas, nonce, and calldata in compact wallet-oriented rows.
- Keep the existing raw JSON preview available for debugging.
- Preserve trusted-window-only approval commands and current signer-helper boundaries.

## Invariants

- UI changes must not expose Alchemy token/RPC URL.
- UI changes must not authorize any new signing path.
- Blocked, high-risk, and ordinary transaction approvals must remain visually distinct.
- Raw request data remains inspectable but not the primary user surface.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 23 tests.
- Static `file://` Browser preview was attempted and blocked by Browser URL policy; no workaround was used.

## Risks

- Frontend-only formatting can misrepresent on-chain amounts if unit conversion is wrong; keep raw values visible.
- Real dApp runtime behavior still needs a Tauri visual smoke after code-level checks.

# Tauri Mock Runtime Smoke

Status: completed

## Goal

Run the desktop wallet in mock mode and verify the real Tauri windows exercise the expected user flow: trusted UI status/account/balance, local dApp provider injection, request capture, approval, and transaction review policy display.

## Scope

- Launch `framkey-desktop` with `FRAMKEY_WALLET_MODE=mock_in_memory` and Alchemy-backed RPC/simulation from `.env`.
- Use the local dApp WebView buttons to trigger read-only RPC, `personal_sign`, typed-data capture, and `eth_sendTransaction` capture.
- Use the trusted UI to approve/reject where appropriate.
- Inspect terminal logs and visible UI state for failures.
- Fix runtime bugs found during the smoke.

## Invariants

- Mock mode must not touch the GBxCart card or Keychain vault.
- Alchemy token/RPC URL must not be printed.
- Transaction signing still requires policy authorization; blocked requests must not sign.
- Real UI review cards must remain usable at the configured desktop window size.

## Likely Files

- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/dapp.js`
- `apps/framkey-desktop/ui/styles.css`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- Runtime Tauri mock smoke via local dApp and trusted UI.
- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`

Completed verification:

- `cargo run -p framkey-desktop` and bundled `.app` launch in mock mode both start the app process; Tauri internal window state reports visible `main` and `dapp` windows after explicit main-window creation.
- Computer Use, AppleScript, and `screencapture` could not inspect/capture the Tauri windows in this host session even though Tauri reports both windows visible.
- Added env-gated runtime smoke instrumentation:
  - `FRAMKEY_DESKTOP_WINDOW_SMOKE=1` logs main/dapp window visibility, size, and position.
  - `FRAMKEY_DESKTOP_AUTOSMOKE=1` lets the dApp WebView drive provider calls while the trusted UI WebView auto-approves mock-mode review requests and logs smoke stages.
- `FRAMKEY_WALLET_MODE=mock_in_memory FRAMKEY_SIMULATION_PROVIDER=alchemy_asset_changes FRAMKEY_DESKTOP_AUTOSMOKE=1 cargo run -p framkey-desktop`: passed through visible window creation, `eth_chainId`, pre-connect empty `eth_accounts`, trusted approval for `eth_requestAccounts`, connected `eth_accounts`, and approved `personal_sign`; `eth_sendTransaction` was captured but correctly blocked by policy when live Alchemy simulation did not produce an overrideable/allowed decision.
- `FRAMKEY_WALLET_MODE=mock_in_memory FRAMKEY_SIMULATION_PROVIDER=local_decoder_only FRAMKEY_DESKTOP_AUTOSMOKE=1 cargo run -p framkey-desktop`: passed through visible window creation, account connection approval, typed-data dry-run capture/block, approved `personal_sign`, and high-risk-approved `eth_sendTransaction` signing path. Final broadcast returned expected mock-account insufficient-funds provider error after signing attempt.
- Added provider-flow regression tests covering mock `personal_sign` review approval and mock `eth_sendTransaction` high-risk approval, signing, and local RPC broadcast.
- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed after autosmoke instrumentation.
- `cargo nextest run -p framkey-desktop`: passed, 30 tests.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

# Alchemy Asset Changes Normalization

Status: completed

Archived from `PLANS.md` during Uniswap/Aave intent decoder planning.

## Goal

Make live Alchemy simulation useful in the trusted confirmation UI by normalizing `alchemy_simulateAssetChanges` `result.changes` into the existing `assetTransfers` and `approvals` report fields instead of leaving provider asset changes only in raw JSON.

## Scope

- Parse successful Alchemy simulation responses with `changes`, `gasUsed`, and `error`.
- Convert transfer and approval changes into existing normalized report fields.
- Fail closed when a provider-simulated response is missing or malforms `result.changes`.
- Preserve raw provider response for audit while using normalized fields for UI display.

## Invariants

- Do not expose Alchemy token/RPC URL, signatures, raw transaction bytes, wallet secret, recovery share bytes, KEK, DEK, or RRK.
- Do not change the policy rule that only live provider-simulated low-risk requests get ordinary approval.
- Do not trust UI metadata to change policy decisions.
- Keep local decoder behavior available for deterministic mock smoke and high-risk override flows.

## Verification

- Focused `cargo nextest run -p framkey-simulation alchemy`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Mock Tauri autosmoke to verify existing transaction review/signing still works.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 9 tests.
- `echo $RUSTC_WRAPPER` confirmed `sccache`; `sccache --show-stats` reported healthy cache stats.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-simulation alchemy`: passed, 3 tests.
- `cargo nextest run -p framkey-desktop`: passed, 43 tests.
- Mock Tauri autosmoke with `.env`-derived Alchemy read RPC passed through account connection, Permit typed-data signing, `personal_sign`, transaction review/signing, expected insufficient-funds broadcast failure, and portfolio smoke with `ok=true`, `rpc=true`, `errors=0`.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- External GUI automation remains unavailable in this host session, so the durable runtime evidence is from Tauri's own window state plus WebView-driven autosmoke logs.
- Mainnet mock accounts are unfunded, so transaction broadcast can fail after successful review/signing; that is acceptable when the review/signing path is proven.

# Provider Compatibility Hardening

Status: completed

## Goal

Make the injected Tauri dApp provider closer to a normal EIP-1193/EIP-6963 wallet provider so common DeFi frontends and connector libraries can discover FRAMKey, subscribe to account/chain events, and use legacy compatibility methods where required.

## Scope

- Add provider state for `selectedAddress`, `chainId`, `networkVersion`, and connection status.
- Emit `connect`, `accountsChanged`, and `chainChanged` events from successful provider requests.
- Add common event aliases and legacy methods: `addListener`, `off`, `once`, `listenerCount`, `listeners`, `isConnected`, `enable`, `send`, and `sendAsync`.
- Give EIP-6963 provider info a non-empty data URI icon.
- Extend the local dApp test page to display provider events.
- Add a Node-based regression test for provider injection behavior without requiring a Tauri WebView.

## Invariants

- The provider remains a relay; it must not expose secrets, RPC URLs, or direct Tauri commands beyond `framkey_provider_request`.
- Unsupported or blocked signing methods must still be decided by the Rust provider path, not by frontend compatibility shims.
- EIP-6963 announcement must not overwrite an existing `window.ethereum` provider unless none exists.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/ui/dapp.html`
- `apps/framkey-desktop/ui/dapp.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`

Completed verification:

- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 6 tests.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 25 tests.
- Nonprinting `.env` Alchemy `eth_chainId` smoke: passed with result `0x1`.

## Risks

- Legacy provider methods vary across wallets; implement the smallest compatibility surface that maps cleanly onto EIP-1193.
- Some dApps fingerprint MetaMask-specific behavior; FRAMKey should not claim to be MetaMask.

# Tauri dApp Account Permissions

Status: completed

## Goal

Move account exposure from "any dApp can ask and get the address" to normal wallet connection behavior: each dApp origin must request account access, the trusted FRAMKey UI must approve or reject it, and approved origins can later query `eth_accounts` during the current app session.

## Scope

- Add an in-memory per-origin account permission store for the Tauri process.
- Make `eth_accounts` return an empty array until the origin is approved.
- Make `eth_requestAccounts` and `wallet_requestPermissions` create trusted review requests and grant only after approval.
- Support `wallet_getPermissions` and `wallet_revokePermissions` for common connector compatibility.
- Add trusted UI connected-site listing and a disconnect action.
- Extend the local dApp test page with permission request/query/revoke buttons.
- Add regression tests for approval, rejection, granted lookup, and revocation behavior.

## Invariants

- Trusted UI origin remains allowed for local account-management commands.
- dApp account access grants are process-local for now; no persistent permission database yet.
- Approval grants only `eth_accounts`, never signing or transaction permission.
- Signatures and transactions must still require their own review and policy gates.
- No wallet secret, Alchemy token, RPC URL, KEK, DEK, or recovery share bytes are exposed through permission outputs.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/dapp.html`
- `apps/framkey-desktop/ui/dapp.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- JS syntax checks for provider and UI files.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`

Completed verification:

- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 7 tests.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 28 tests.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- Some dApps assume persistent account permissions; session-only grants are acceptable for the current Tauri MVP but should become a small encrypted/local permission store later.
- If a real Keychain vault is configured, first approval can still trigger Touch ID after the user approves account connection.

# Signing Requires Connected Origin

Status: completed

## Goal

Require a dApp origin to have an approved `eth_accounts` session grant before it can ask FRAMKey to review signatures or transactions. This makes the Tauri wallet behavior match normal wallet expectations: connect first, then sign or transact.

## Scope

- Check account permission before `personal_sign`, `eth_sendTransaction`, `eth_sign`, `eth_signTransaction`, and typed-data request capture.
- Return an EIP-1193-style unauthorized provider error for unconnected dApps instead of adding a review item.
- Keep trusted UI account-management calls exempt.
- Preserve existing signing, transaction policy, and signer-helper boundaries after the origin is connected.
- Add regression tests for unconnected rejection and connected success paths.

## Invariants

- A connection grant only authorizes account exposure and eligibility to request review; it does not pre-approve any signature or transaction.
- Unconnected signing requests must not enter the review queue.
- Trusted UI commands must remain able to read local account metadata.
- No secrets or RPC credentials are exposed through the new error path.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- JS syntax checks to ensure frontend files remain valid.

Completed verification:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 30 tests.
- JS syntax checks for provider and UI files: passed.

## Risks

- Some legacy dApps may attempt signing before connecting; they will now receive `4100` and must call `eth_requestAccounts` first.

# Recovery Backup Guidance UX

Status: completed

## Goal

Make the desktop recovery flow usable without reading raw JSON: after creating a vault, the trusted UI should show the generated manifest/share files as a concrete backup checklist with destination labels, hashes, and the recovery rule. After recovery rewrap, the UI should show which share files were used and confirm that wallet-secret bytes were not touched.

## Scope

- Add a trusted UI recovery panel for the latest vault creation or recovery operation.
- Render recovery manifest and six share files from `recoveryBackups.files`.
- Group share files by destination:
  - iCloud
  - Google Drive
  - local physical
  - remote physical
- Show backup set id, wallet id, generation, share count, and `cloudAloneRecovers=false`.
- Keep raw JSON available for audit, but make the backup checklist the primary surface.
- Render recovery rewrap results with file count, used paths, rewritten save hash, and `walletSecretTouched=false`.
- Do not display recovery share bytes or secrets.

## Invariants

- UI must not expose `shareHex`, recovery root key, DEK, KEK, wallet secret, Alchemy token, or RPC URL.
- Recovery files remain written by the desktop process with no-overwrite semantics.
- Cloud-only recovery must remain described as insufficient.
- Existing CLI recovery pack layout remains unchanged.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 30 tests.

## Risks

- The UI can guide file placement but cannot verify that the user actually uploaded or copied files to the intended destinations.

# Remote dApp Provider Telemetry

Status: completed

## Goal

Make Uniswap/Aave compatibility work observable inside the trusted desktop UI: when the untrusted dApp WebView loads a remote site, FRAMKey should show provider injection lifecycle events and provider request outcomes without requiring WebKit devtools or terminal log inspection.

## Scope

- Add a bounded in-memory provider event log in the desktop state.
- Record provider request outcomes with method, origin, status, duration, result shape, and sanitized error metadata.
- Let the injected provider report non-secret lifecycle events such as provider injection, EIP-6963 announcement, and EIP-6963 provider request.
- Add trusted UI controls to view, refresh, and clear the event log.
- Update docs so Uniswap/Aave smoke testing has a concrete evidence surface.

## Invariants

- Do not store raw provider params, raw calldata, signatures, RPC URLs, Alchemy token, KEK, DEK, wallet secret, or recovery share bytes in telemetry.
- Event reads and clears must be restricted to the trusted main window.
- The untrusted dApp WebView may only append bounded telemetry events and provider requests; it must not gain filesystem, Keychain, GBxCart, signer-helper, or token access.
- The event log is process-local debug/diagnostic state, not persistence.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Nonprinting `.env` Alchemy smoke for runtime configuration.

Completed verification:

- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 8 tests.
- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 31 tests.
- Nonprinting `.env` Alchemy `eth_blockNumber` smoke: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.
- `FRAMKEY_WALLET_MODE=mock_in_memory FRAMKEY_SIMULATION_PROVIDER=alchemy_asset_changes FRAMKEY_DESKTOP_AUTOSMOKE=1 cargo run -p framkey-desktop`: passed through visible window creation, `.env`-derived Alchemy simulation config, account approval, blocked typed-data capture, and approved `personal_sign`; the test transaction was captured and correctly remained blocked by current policy.

## Risks

- WebKit compatibility issues on real remote sites may still require site-specific follow-up after telemetry identifies the failing method or lifecycle gap.

# Remote dApp Startup Smoke

Status: completed

## Goal

Make remote Uniswap/Aave WebView compatibility testable without relying on manual clicking or macOS accessibility capture: a development run should be able to start the dApp WebView at a chosen remote target and stream sanitized provider telemetry to terminal logs.

## Scope

- Add a startup dApp target env override for local/Uniswap/Aave/custom `http`/`https` URLs.
- Add an opt-in provider telemetry stderr stream for development smoke evidence.
- Add explicit Tauri app-command ACL permissions so approved remote dApp origins can call only the provider bridge/telemetry command surface.
- Add an opt-in read-only remote provider smoke for `eth_chainId`, `eth_accounts`, and `eth_blockNumber`.
- Keep existing trusted UI buttons and event panel as the normal product surface.
- Document the remote dApp smoke workflow with `.env` Alchemy configuration.

## Invariants

- Startup target must use the same URL validation as the trusted UI open command.
- Telemetry output must not print raw params, calldata, signatures, RPC URLs, Alchemy token, KEK, DEK, wallet secret, or recovery share bytes.
- The dApp WebView remains untrusted and receives only the injected provider.
- Remote IPC is allowlisted to `https://app.uniswap.org` and `https://app.aave.com`; trusted-only commands still enforce the main-window boundary.
- This is a debug/smoke workflow, not persistent telemetry.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/build.rs`
- `apps/framkey-desktop/src-tauri/capabilities/dapp.json`
- `apps/framkey-desktop/src-tauri/capabilities/default.json`
- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Start Uniswap and Aave WebViews with mock wallet + Alchemy and verify provider injection telemetry reaches stderr without printing credentials.

Completed verification:

- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 8 tests.
- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 31 tests.
- Nonprinting `.env` Alchemy `eth_blockNumber` smoke: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.
- Uniswap remote WebView smoke with mock wallet, Alchemy, startup URL, remote provider smoke, and stderr telemetry: passed; window loaded `https://app.uniswap.org/`, provider injected, EIP-6963 request/announce events fired, the site called `eth_accounts`, and smoke `eth_chainId`, `eth_accounts`, and `eth_blockNumber` completed through the provider bridge.
- Aave remote WebView smoke with mock wallet, Alchemy, startup URL, remote provider smoke, and stderr telemetry: passed; window loaded `https://app.aave.com/`, provider injected, repeated EIP-6963 request/announce events fired, and smoke `eth_chainId`, `eth_accounts`, and `eth_blockNumber` completed through the provider bridge.

## Risks

- Real dApp behavior can change independently; this smoke proves WebView loading and provider discovery signals, not complete swap/borrow execution.

# Remote dApp Interactive Smoke

Status: completed

## Goal

Extend the remote Uniswap/Aave smoke from read-only provider discovery into a repeatable development flow that exercises account connection, `personal_sign`, and `eth_sendTransaction` review/signing from a real remote WebView without exposing the Alchemy token or using a real vault.

## Scope

- Add a mode value for `FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE`:
  - `read` / `1` keeps the current read-only smoke.
  - `interactive` also requests accounts, signs a fixed smoke message, and submits a minimal transaction request.
- Add a separate trusted UI auto-approval gate for remote smoke so the dApp WebView can wait on the real review broker.
- Keep auto-approval limited to mock wallet mode.
- Record sanitized telemetry for each smoke request, including errors, without logging raw params, calldata, signatures, RPC URLs, tokens, wallet secrets, or recovery material.
- Update docs for the Alchemy-backed remote interactive smoke workflow.

## Invariants

- Remote dApp content remains untrusted and only reaches the provider bridge command surface.
- `FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE` must not auto-approve in Keychain-vault mode.
- A transaction broadcast failure from an unfunded mock account is acceptable after the review/signing path is proven.
- Read-only smoke remains available for compatibility checks that should not open approval prompts.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- JS syntax checks for provider and desktop UI files.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Nonprinting `.env` Alchemy RPC smoke.
- Uniswap/Aave remote interactive smoke with mock wallet, Alchemy, trusted auto-approval, and stderr telemetry.

Completed verification:

- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 9 tests.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: healthy, no cache errors.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 31 tests.
- Nonprinting `.env` Alchemy `eth_blockNumber` smoke: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.
- Uniswap remote interactive smoke with `FRAMKEY_SIMULATION_PROVIDER=alchemy_asset_changes`: provider injection, EIP-6963, `eth_requestAccounts`, and `personal_sign` passed; `eth_sendTransaction` reached trusted review and was blocked by current Alchemy simulation policy before signing.
- Uniswap remote interactive smoke with `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only` and `.env` Alchemy RPC: passed through account approval, `personal_sign`, transaction high-risk approval, signing path, and Alchemy broadcast attempt; final provider error was expected mock-account insufficient funds.
- Aave remote interactive smoke with `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only` and `.env` Alchemy RPC: passed through provider injection, repeated EIP-6963 discovery, account approval, `personal_sign`, transaction high-risk approval, signing path, and Alchemy broadcast attempt; final provider error was expected mock-account insufficient funds.

## Risks

- Live DeFi app loading and connector behavior can drift outside the repo.
- Alchemy simulation or mainnet gas/nonce/broadcast calls can fail transiently; the important boundary is whether FRAMKey captures, reviews, signs, and reports the result without leaking credentials.

# Trusted Wallet Product Readiness UX

Status: completed

Archived from `PLANS.md` while starting `Alchemy Preferred RPC Defaults`.

## Goal

Make the Tauri wallet feel less like a debug console by giving the trusted UI a compact product-oriented view of wallet readiness, dApp session state, pending approvals, and recovery backup placement.

## Scope

- Add a trusted UI DeFi session panel that summarizes:
  - current wallet/account/RPC readiness
  - selected dApp target
  - connected origins
  - latest provider discovery/request status
  - latest signature and transaction review outcomes
  - the next product action
- Keep provider event raw details available but make the top-level state easier to scan.
- Make the recovery backup plan more actionable after vault creation by surfacing destination groups and per-file placement checks.
- Do not add new signing capability or loosen transaction policy.

## Invariants

- No Alchemy token, RPC URL, calldata, signatures, wallet secret, recovery share bytes, KEK, or DEK should be printed in the new UI surfaces.
- dApp content remains untrusted; this slice only changes trusted UI rendering and local state derived from existing trusted commands.
- The recovery UI may track local checklist state only; it must not imply cloud upload happened automatically.
- The product surface should stay dense and operational, not become a marketing page.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Runtime mock smoke if UI changes affect request/recovery/session rendering.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 9 tests.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: healthy, no cache errors.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 31 tests.
- `FRAMKEY_WALLET_MODE=mock_in_memory FRAMKEY_SIMULATION_PROVIDER=local_decoder_only FRAMKEY_RPC_TIMEOUT_MS=30000 FRAMKEY_DESKTOP_AUTOSMOKE=1 FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1 cargo run -p framkey-desktop`: passed through visible trusted/dApp windows, provider injection, account approval, typed-data blocked review, `personal_sign`, transaction high-risk approval, signing path, and expected insufficient-funds broadcast error.
- Browser static `file://` preview was attempted and blocked by Browser URL policy; no workaround was used.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- A purely UI-derived readiness summary can be misleading if it overstates chain success; use conservative wording and derive status from existing event/review evidence only.

# Alchemy Preferred RPC Defaults

Status: completed

Archived from `PLANS.md` while starting `Recovery Health Summary`.

## Goal

Make the Alchemy-first RPC path explicit and regression-tested: a debug `.env` with only `ALCHEMY_TOKEN` should configure the trusted read-RPC provider as Alchemy mainnet without exposing the token or full URL in wallet status.

## Scope

- Add focused configuration tests for token-derived Alchemy RPC.
- Verify the configured RPC is reported only as sanitized metadata.
- Keep transaction simulation local by default unless `FRAMKEY_SIMULATION_PROVIDER=alchemy_asset_changes` is explicitly selected.
- Use the existing `.env` token for a nonprinting runtime smoke.

## Invariants

- Do not print or return the Alchemy token, full RPC URL, calldata, signatures, wallet secret, recovery share bytes, KEK, DEK, or RRK.
- Do not expand the untrusted dApp RPC method allowlist.
- Do not make live simulation the default just because an Alchemy token is present.
- Keep `FRAMKEY_RPC_URL` and explicit Alchemy RPC URL overrides higher priority than token-derived defaults.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- Focused `cargo nextest run -p framkey-desktop` tests for Alchemy defaults.
- Mock runtime smoke with `.env`-derived Alchemy RPC and sanitized provider telemetry.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs` passed 9 tests.
- `echo $RUSTC_WRAPPER` showed `sccache`; `sccache --show-stats` was healthy.
- `cargo fmt --all -- --check`: passed after formatting the new Rust code.
- `cargo check -p framkey-desktop`: passed.
- Focused `cargo nextest run -p framkey-desktop alchemy_token_configures_read_rpc_without_enabling_live_simulation explicit_rpc_url_takes_priority_over_alchemy_token live_simulation_requires_explicit_provider_selection`: passed, 3 tests.
- `cargo nextest run -p framkey-desktop`: passed, 43 tests.
- Mock Tauri autosmoke with `.env`-derived Alchemy RPC passed: trusted/dApp windows opened, provider injection worked, account approval, Permit typed-data signing, `personal_sign`, transaction review/signing, expected insufficient-funds broadcast failure, and portfolio smoke reported `ok=true`, `rpc=true`, `errors=0`.

## Risks

- Alchemy can fail transiently or rate-limit live smoke; the regression test should cover config behavior without network.

# Recovery Health Summary

Status: completed

Archived from `PLANS.md` while starting `Transaction Risk Summary UX`.

## Goal

Keep the desktop recovery flow product-complete after vault creation by preserving the generated backup plan while users run read-only recovery drills or recovery rewrap, and by showing one health summary that tells the user the next safe recovery action.

## Scope

- Split trusted UI recovery state into backup-pack, drill, and recover result slots.
- Re-render the Recovery Backup Plan from all remembered recovery states instead of letting later operations replace earlier backup file cards.
- Add a Recovery Health summary for created/placed/drilled/recovered state and next action.
- Keep placement checklist, Finder reveal, and recovery-file prefill available after `Check Recovery Set`.

## Invariants

- Do not display recovery share bytes, wallet secret, KEK, DEK, RRK, Alchemy token, RPC URL, calldata, or signatures.
- Do not change recovery file format, recovery policy, signer-helper trust boundary, or destructive recovery confirmation.
- Checklist state remains local UI state and must not imply cloud upload or physical copy happened automatically.
- The untrusted dApp WebView must not gain access to local paths or recovery commands.

## Likely Files

- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Mock runtime smoke if the UI initialization path changes.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs` passed 9 tests.
- `echo $RUSTC_WRAPPER` showed `sccache`; `sccache --show-stats` was healthy.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 43 tests.
- Mock Tauri autosmoke with `.env`-derived Alchemy RPC passed: trusted/dApp windows opened, provider injection worked, account approval, Permit typed-data signing, `personal_sign`, transaction review/signing, expected insufficient-funds broadcast failure, and portfolio smoke reported `ok=true`, `rpc=true`, `errors=0`.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- The health summary can only prove local generated/drill state and operator checklist state; it cannot verify cloud uploads or physical storage.

# Transaction Risk Summary UX

Status: completed

Archived from `PLANS.md` while starting `Alchemy Asset Changes Normalization`.

## Goal

Make DeFi transaction confirmation easier to evaluate by moving the policy decision, approval path, and concrete blocker/warning reasons into the top-level transaction review card instead of leaving them mostly inside raw simulation details.

## Scope

- Add a compact transaction risk detail section to trusted UI review cards.
- Translate policy blocker codes into user-readable labels without hiding exact codes/messages.
- Preserve the existing raw simulation and policy gate details for audit.
- Do not change signing policy, policy evaluation, simulation semantics, or approval decisions.

## Invariants

- Do not expose raw calldata beyond existing byte counts/collapsible debug previews.
- Do not display Alchemy token/RPC URL, signatures, wallet secret, recovery share bytes, KEK, DEK, or RRK.
- Ordinary approval, high-risk override, and blocked states must continue to come from policy flags, not UI-only interpretation.
- dApp WebView permissions and method allowlist remain unchanged.

## Likely Files

- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Mock Tauri autosmoke to verify transaction review/signing still reaches the expected broadcast error.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs` passed 9 tests.
- `echo $RUSTC_WRAPPER` showed `sccache`; `sccache --show-stats` was healthy.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 43 tests.
- Mock Tauri autosmoke with `.env`-derived Alchemy RPC passed: trusted/dApp windows opened, provider injection worked, account approval, Permit typed-data signing, `personal_sign`, high-risk transaction approval/signing, expected insufficient-funds broadcast failure, and portfolio smoke reported `ok=true`, `rpc=true`, `errors=0`.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- UI labels can oversimplify security context; keep exact blocker codes/messages visible alongside friendlier labels.

# Remote Permit Compatibility Evidence

Status: completed

## Goal

Make the newly enabled Permit/Permit2 typed-data signing visible in the same Uniswap/Aave development evidence path as account connection, `personal_sign`, and transaction signing, so the trusted UI can show whether a remote DeFi run exercised Permit signing.

## Scope

- Extend remote interactive provider smoke to request a deterministic ERC-20 Permit `eth_signTypedData_v4` after account connection.
- Keep smoke telemetry sanitized: record signature shape only, never raw typed-data params or signature bytes.
- Add a Permit/typed-data step to the trusted dApp Compatibility panel for Local Test, Uniswap, and Aave.
- Make the trusted UI autosmoke window configurable for slow remote pages without enabling real-wallet auto-approval.
- Update docs and roadmap so remote smoke evidence includes Permit typed-data signing.

## Invariants

- Do not enable arbitrary typed-data signing; broker restrictions from Controlled Typed Data Signing remain authoritative.
- Do not log raw typed-data params, signatures, Alchemy token/RPC URL, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Remote smoke stays mock/autosmoke-only for approvals and must not auto-approve real Keychain-vault signing.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `node --check apps/framkey-desktop/ui/main.js`
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- Mock Tauri autosmoke showing Permit typed-data signing remains successful.

Completed verification:

- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 9 tests.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 38 tests.
- Local WebView remote-provider interactive smoke with `.env`-derived Alchemy RPC passed: provider smoke ran read checks, account connection, `personal_sign`, ERC-20 Permit `eth_signTypedData_v4` with `typedIntent=erc20_permit`, `controlled_typed_data_signing`, transaction review/signing, expected insufficient-funds broadcast failure, and `provider_smoke_completed`; terminal telemetry did not print raw params or signature bytes.
- Uniswap remote-provider interactive smoke with `.env`-derived Alchemy RPC passed: provider injection/EIP-6963, read checks, account connection, `personal_sign`, ERC-20 Permit `eth_signTypedData_v4` with `typedIntent=erc20_permit`, `controlled_typed_data_signing`, transaction review/signing, expected insufficient-funds broadcast failure, and `provider_smoke_completed`.
- Added `FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE_DURATION_MS` for slow remote-page smoke runs; `node --check apps/framkey-desktop/ui/main.js`, `cargo fmt --all -- --check`, `cargo check -p framkey-desktop`, and `cargo nextest run -p framkey-desktop` passed after the change.
- Aave remote-provider interactive smoke passed with `FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE_DURATION_MS=90000`: provider injection/EIP-6963, read checks, account connection, `personal_sign`, ERC-20 Permit `eth_signTypedData_v4` with `typedIntent=erc20_permit`, `controlled_typed_data_signing`, transaction review/signing, expected insufficient-funds broadcast failure, and `provider_smoke_completed`.

## Risks

- Synthetic Permit smoke proves the wallet/provider path, not that every embedded remote dApp flow will choose FRAMKey or present the same typed-data schema.
- Remote sites can change load timing or provider discovery behavior; keep the configurable autosmoke duration as a development harness, not a production approval policy.

# Recovery Backup Placement Guide

Status: completed

## Goal

Make desktop-created recovery packs more durable and operator-friendly by writing a human-readable placement guide beside the manifest/share files, and avoid leaving partial new backup files behind if pack writing fails before the vault image is written.

## Scope

- Add a non-secret placement guide artifact to the desktop recovery output directory.
- Include destination rules, filenames, public identifiers, and hashes already shown in the trusted UI.
- Surface the guide path/hash in the Recovery Backup Plan panel.
- Clean up only newly created files if backup pack writing fails partway through.

## Invariants

- Do not include recovery share bytes, wallet secret, DEK, KEK, RRK, Alchemy token, RPC URL, or raw signing data in the guide or UI summary.
- Keep no-overwrite semantics for manifest, share files, and the new guide.
- Do not change the recovery share JSON format or recovery threshold policy.

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop recovery_backup_pack`: passed, 2 tests.
- `cargo nextest run -p framkey-desktop`: passed, 39 tests.

# Recovery Set Drill

Status: completed

## Goal

Add a read-only recovery drill path to the Tauri wallet so users can check that selected recovery backup files satisfy the policy before running the destructive recovery rewrap that overwrites the configured vault device.

## Scope

- Add signer-helper IPC for validating recovery-share sets without Keychain access and without save-image writes.
- Keep RRK reconstruction inside the signer helper; return only public metadata, satisfied group labels, file count, and pass/fail status.
- Add a trusted UI `Check Recovery Set` action that uses the same recovery file list as recovery rewrap.
- Show drill results in the Recovery Backup Plan panel without implying cloud upload/copy actually happened.

## Invariants

- Do not print or return recovery share bytes, RRK, DEK, KEK, wallet secret, Alchemy token, or RPC URL.
- Do not write the configured vault device during a drill.
- Do not relax the documented policy: cloud alone remains insufficient.

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-ipc -p framkey-signer-helper -p framkey-desktop`
- `cargo nextest run -p framkey-ipc -p framkey-signer-helper -p framkey-desktop`

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-ipc -p framkey-signer-helper -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-ipc -p framkey-signer-helper -p framkey-desktop`: passed, 53 tests.
- Mock Tauri autosmoke with `.env`-derived Alchemy RPC passed after the UI change: trusted and dApp windows opened, provider injection worked, account connection, Permit typed-data signing, `personal_sign`, transaction review/signing, expected insufficient-funds broadcast failure, and portfolio smoke reported `ok=true`, `rpc=true`, `errors=0`.

# Recovery Backup Finder Reveal

Status: completed

## Goal

Make recovery backup placement more ergonomic by letting the trusted UI reveal generated backup files in Finder, so the user can place/upload the iCloud, Google Drive, local physical, and remote physical files without manually copying long paths.

## Scope

- Add a trusted-only Tauri command for revealing an existing local path.
- Expose the command only to the `main` trusted wallet UI capability.
- Add `Reveal` actions to recovery manifest, placement guide, and share cards.
- Keep dApp WebView and provider bridge unable to open arbitrary local paths.

## Invariants

- Do not expose recovery share bytes, wallet secret, KEK, DEK, RRK, Alchemy token, or RPC URL.
- Reject empty, malformed, or missing paths before launching Finder.
- Use argument-based process invocation only; no shell interpolation.

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop reveal_path_request`: passed, 1 test.
- `cargo nextest run -p framkey-desktop`: passed, 40 tests.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

# Trusted Transaction Activity

Status: completed

Archived from `PLANS.md` during trusted workspace UX planning.

## Goal

Make `eth_sendTransaction` feel like a normal wallet flow after signing by showing a trusted activity/history panel with review status, approval status, broadcast hash or error, and optional receipt status refresh.

## Scope

- Add a trusted-only in-memory transaction activity log in the desktop process.
- Record transaction review capture, local approval/rejection, signing/broadcast failure, and successful broadcast.
- Add a trusted UI command to read activity and optionally refresh receipts through the configured RPC.
- Render recent transaction activity in the trusted wallet UI with tx hash, origin, method, policy, receipt state, and errors.
- Keep the existing review queue as the confirmation surface; activity is post-review wallet state.

## Invariants

- Do not store or display raw calldata, raw signed transactions, signatures, Alchemy token/RPC URL, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Activity is trusted UI only; the dApp provider must not get a new activity/history method.
- Receipt refresh failure must not break account connection, signing, transaction review, recovery, or portfolio.
- Keep activity process-local in this slice; persistence can be added after the model is stable.

## Verification

- JS syntax/provider checks.
- Focused Rust tests for transaction activity recording and receipt refresh shaping.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Mock Tauri autosmoke to verify review/sign/broadcast behavior still works and activity renders without blocking.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 9 tests.
- `echo $RUSTC_WRAPPER` confirmed `sccache`; `sccache --show-stats` returned healthy stats.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop transaction_activity mock_send_transaction_provider_flow_uses_high_risk_review_override`: passed, 1 focused test with transaction activity/receipt assertions.
- `cargo nextest run -p framkey-desktop`: passed, 43 tests.
- Mock Tauri autosmoke with `.env`-derived Alchemy read RPC passed through account connection, Permit typed-data signing, `personal_sign`, transaction review/signing, expected insufficient-funds broadcast failure, portfolio smoke, and `trusted_ui_activity_smoke` with transaction activity status `failed`.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

# DeFi Compatibility Run Status

Status: completed

Archived from `PLANS.md` during Uniswap/Aave intent decoder planning.

## Goal

Give the trusted wallet UI a per-dApp compatibility summary for common targets so Uniswap/Aave support is visible as product state rather than buried in raw provider events.

## Scope

- Add a trusted UI compatibility panel for Local Test, Uniswap, and Aave.
- Summarize each target's provider injection, read RPC, account connection, `personal_sign`, and `eth_sendTransaction` status from existing provider events and review queue state.
- Provide quick open buttons for each target.
- Treat expected mock-account broadcast failures as a transaction path that reached review/sign/broadcast, not as a full chain success.
- Keep raw provider events and review cards as the detailed audit surface.

## Invariants

- Do not store or display raw params, calldata, signatures, RPC URLs, Alchemy token, wallet secret, or recovery share bytes.
- Do not add new dApp permissions, automatic approvals, or signing paths.
- Compatibility status is process-local evidence, not a certification that future dApp behavior will remain compatible.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- JS syntax checks for desktop UI and provider files.
- Provider injection regression test.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Runtime smoke where practical.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 9 tests.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: healthy, no cache errors.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 31 tests.
- `FRAMKEY_WALLET_MODE=mock_in_memory FRAMKEY_SIMULATION_PROVIDER=local_decoder_only FRAMKEY_RPC_TIMEOUT_MS=30000 FRAMKEY_DESKTOP_AUTOSMOKE=1 FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1 cargo run -p framkey-desktop`: passed through visible trusted/dApp windows, provider injection, account approval, typed-data blocked review, `personal_sign`, transaction high-risk approval, signing path, and expected insufficient-funds broadcast error.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- The summary can only report evidence currently in the process event log and review queue; after restart it starts empty by design.

# Recovery Placement Readiness Checklist

Status: completed

Archived from `PLANS.md` during structured transaction risk summary planning.

## Goal

Make the desktop recovery flow guide the user through backup placement after vault generation and show whether the checked placements satisfy the documented recovery policy.

## Scope

- Track per-file placement checks in trusted UI state.
- Summarize iCloud, Google Drive, local physical, and remote physical placement state.
- Compute whether the checked files satisfy either:
  - both cloud shares plus at least one physical share, or
  - at least one local physical plus at least one remote physical share.
- Keep cloud-only explicitly insufficient.
- Use the readiness summary to guide the recovery file prefill and next action, without uploading or copying files automatically.

## Invariants

- Checklist state is local UI state only; it must not imply files were uploaded or copied by the app.
- Do not display recovery share bytes, wallet secret, DEK, KEK, RRK, or Alchemy credentials.
- Do not change the recovery file format or recovery rewrap policy.
- Existing no-overwrite semantics for generated backup files remain unchanged.

## Likely Files

- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- JS syntax checks.
- Provider injection regression test.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Runtime smoke if the UI initialization path changes.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs` passed 9 tests.
- `echo $RUSTC_WRAPPER` showed `sccache`; `sccache --show-stats` was healthy.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop` passed 31 tests.
- Mock Tauri autosmoke passed through trusted and dApp windows, provider injection, account approval, typed-data blocking, `personal_sign`, high-risk tx approval/signing path, and expected insufficient-funds broadcast failure.
- `cargo tauri build --debug --bundles app --no-sign` produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- The app cannot verify external cloud uploads or physical copies; the checklist is an operator aid, not proof of backup durability.

# Uniswap and Aave Intent Decoder

Status: completed

Archived from `PLANS.md` during transaction impact summary planning.

## Goal

Make transaction confirmation more useful for common DeFi use by recognizing top-level Uniswap and Aave calldata in the local decoder, so the trusted UI can show protocol intent instead of treating those calls as unknown selectors.

## Scope

- Add local decoder entries for common Uniswap V2/V3/Universal Router, generic multicall, and Aave V3 Pool selectors.
- Decode fixed top-level ABI words where practical, such as amounts, recipient, asset, deadline, and Aave rate mode.
- For dynamic payloads such as paths, bytes arrays, commands, and multicalls, show bounded metadata like payload byte count or offset rather than raw calldata.
- Surface protocol labels in trusted transaction review without adding new permissions, automatic approvals, or signing paths.
- Preserve the existing policy split: live Alchemy simulation is still required for ordinary transaction approval; local-only protocol recognition only removes the unknown-selector blocker.

## Invariants

- Do not display or store raw params, full calldata, signatures, Alchemy token/RPC URL, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Unknown selectors must remain warning/override gated.
- Malformed calldata must remain blocked.
- Decoding is a product review aid, not a guarantee that the DeFi transaction is safe.

## Verification

Completed verification:

- Selector mapping was checked locally with Keccak-256 before implementation.
- `cargo check -p framkey-simulation`: passed.
- `cargo nextest run -p framkey-simulation uniswap`: passed, 4 tests.
- `cargo nextest run -p framkey-simulation aave`: passed, 1 test.
- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 10 tests.
- `cargo fmt --all -- --check`: passed.
- `cargo nextest run -p framkey-simulation`: passed, 13 tests.
- `cargo check -p framkey-desktop`: passed.
- `FRAMKEY_WALLET_MODE=mock_in_memory FRAMKEY_SIMULATION_PROVIDER=local_decoder_only FRAMKEY_RPC_TIMEOUT_MS=30000 FRAMKEY_DESKTOP_AUTOSMOKE=1 FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1 cargo run -p framkey-desktop`: passed through provider injection, account approval, Permit typed-data signing, `personal_sign`, transaction high-risk review/signing, portfolio smoke, transaction activity smoke, and the expected mock-account insufficient-funds broadcast error.
- `cargo nextest run -p framkey-desktop`: passed, 46 tests.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- The decoder recognizes top-level function intent only; it does not execute nested router commands, multicall payloads, or slippage/math semantics.
- Protocol-recognized local-only transactions still require the explicit high-risk path unless live simulation and policy evaluation allow ordinary approval.

# Trusted Workspace UX

Status: completed

Archived from `PLANS.md` during multi-chain protocol registry planning.

## Goal

Make the trusted desktop wallet feel less like a debug console by organizing the existing wallet, DeFi, recovery, and diagnostics surfaces into workflow tabs while preserving the current commands, smoke flows, and audit details.

## Scope

- Add a trusted workspace tab control for Wallet, DeFi, Recovery, and Diagnostics.
- Assign existing panels to the workflow where users naturally expect them.
- Keep all current controls and debug/audit surfaces available.
- Persist the selected workspace locally so refreshes keep the same context.
- Update docs/plan with the new workspace behavior.

## Invariants

- Do not add dApp permissions, automatic approvals, signing paths, or network access.
- Do not expose raw provider params, calldata, signatures, Alchemy token/RPC URL, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Existing autosmoke and remote smoke must still work without needing manual tab selection.
- The dApp WebView remains untrusted and does not gain access to trusted workspace state.

## Verification

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs && node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 10 provider-injection tests.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- Mock Tauri autosmoke with `.env`-derived Alchemy read RPC passed through hidden-tab workspace UI, provider injection, account approval, Permit typed-data signing, `personal_sign`, transaction review/signing, expected insufficient-funds broadcast failure, portfolio smoke, and transaction activity smoke.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- Hiding panels can hide important pending approvals; the tab bar and Request Review surface must remain obvious enough for active workflows.

# Transaction Counterparty Trust Summary

Status: completed

Archived from `PLANS.md` during remote multi-chain dApp smoke planning.

## Goal

Make Uni/Aave transaction confirmation safer and easier to review by adding a backend-generated counterparty trust summary for the transaction `to` address and approval spenders/operators.

## Scope

- Add a small chain-aware known-counterparty registry for first-target Ethereum mainnet DeFi contracts.
- Label known Uniswap, Permit2, and Aave counterparties in transaction review output.
- Surface unknown transaction recipients and approval spenders/operators as review items.
- Treat unknown approval spender/operator authority as an explicit high-risk approval condition without changing hard-block behavior.
- Render the trust summary near the top of the trusted transaction review card.
- Document that the registry is a conservative review/policy input, not a full contract-verification system.

## Invariants

- Known labels must not bypass live simulation, signer-helper account checks, approval broker expiry, or decision-token validation.
- Unknown spender/operator approval should be safer than before, not silently allowed as ordinary approval.
- Do not display raw params, full calldata, signatures, Alchemy token/RPC URL, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Keep the initial registry narrow and explicit; do not imply broad protocol coverage.

## Likely Files

- `crates/framkey-simulation/src/lib.rs`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused simulation tests for known Uniswap/Aave/Permit2 addresses and unknown approval spender policy.
- Desktop review summary tests for trust JSON.
- JS syntax/provider tests.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-simulation`
- `cargo check -p framkey-desktop`
- Mock Tauri autosmoke and debug bundle build.

Completed verification:

- JS syntax checks and provider-injection tests: passed, 10 provider tests.
- `cargo fmt --all -- --check`: passed.
- Focused simulation trust tests: passed, 5 tests for known Uniswap/Aave/Permit2 and unknown approval authority.
- `cargo check -p framkey-simulation` and `cargo nextest run -p framkey-simulation`: passed, 21 tests.
- `cargo check -p framkey-desktop` and `cargo nextest run -p framkey-desktop`: passed, 46 tests.
- Mock Tauri autosmoke with `.env` Alchemy read RPC: passed through account, Permit, `personal_sign`, transaction review/signing, Portfolio, Activity, and expected mock-account insufficient-funds broadcast error.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- The registry is intentionally narrow and Ethereum-mainnet-only; more chains and protocols still need explicit source-backed entries before broad real-fund use.
- Known counterparty labels are review and policy context, not contract verification or a dApp allow guarantee.

# Transaction Impact Summary

Status: completed

Archived from `PLANS.md` during remote multi-chain dApp smoke planning.

## Goal

Make transaction confirmation easier to review by adding a backend-generated impact summary for native value movement, asset transfers, and approval changes, then rendering that impact directly in the trusted UI.

## Scope

- Add a serializable `TransactionImpactSummary` to transaction review reports.
- Summarize native value movement, approvals, transfers, and whether live provider asset changes are present.
- Highlight unlimited token approval and operator approval as impact items without changing signing policy.
- Render the impact summary near the top of the trusted transaction review card.
- Keep detailed approvals/transfers and raw debug JSON available behind existing collapsible sections.

## Invariants

- Impact summary is review context only; it must not authorize signing or broadcasting.
- Do not display raw params, full calldata, signatures, Alchemy token/RPC URL, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Do not depend on token metadata availability for policy or impact correctness.
- Existing policy, risk, and high-risk approval behavior must remain unchanged.

## Likely Files

- `crates/framkey-simulation/src/lib.rs`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused impact-summary tests for native value, approvals, provider transfers, and empty movement.
- JS syntax/provider tests.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-simulation`
- `cargo check -p framkey-desktop`
- Mock Tauri autosmoke and debug bundle build.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 10 tests.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-simulation`: passed.
- `cargo nextest run -p framkey-simulation impact`: passed, 2 tests.
- `cargo nextest run -p framkey-simulation risk impact`: passed, 4 tests.
- `cargo nextest run -p framkey-simulation`: passed, 17 tests.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop summarizes_transaction_without_raw_calldata transaction_summary_includes_display_only_asset_context`: passed, 2 tests.
- `cargo nextest run -p framkey-desktop`: passed, 46 tests.
- `FRAMKEY_WALLET_MODE=mock_in_memory FRAMKEY_SIMULATION_PROVIDER=local_decoder_only FRAMKEY_RPC_TIMEOUT_MS=30000 FRAMKEY_DESKTOP_AUTOSMOKE=1 FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1 cargo run -p framkey-desktop`: passed through Alchemy-backed RPC, provider injection, account approval, Permit typed-data signing, `personal_sign`, transaction review/signing, portfolio smoke, transaction activity smoke, and expected mock-account insufficient-funds broadcast error.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- Impact summary is display-only; policy and signer authorization still come from existing review policy and decision-token checks.
- Provider asset changes are only present when live Alchemy simulation is explicitly enabled; the deterministic autosmoke kept simulation local while using `.env` Alchemy for read RPC and broadcast.

# Trusted Network Selector

Status: completed

Archived from `PLANS.md` during recovery runtime smoke planning.

## Goal

Make the trusted wallet UI behave like a normal multi-chain wallet by letting the user switch the active session network from the wallet surface, instead of relying only on dApps to request `wallet_switchEthereumChain`.

## Scope

- Add a trusted-window-only command for switching to one of the supported Alchemy-backed chains.
- Reuse the existing supported-chain allowlist and pre-mutation Alchemy `eth_chainId` probe.
- Add a Wallet workspace network selector and switch action populated from `framkey_status.supportedChains`.
- Refresh status, portfolio, session readiness, and activity after a trusted network switch.
- Document the manual network switch path.

## Invariants

- The dApp WebView must not gain direct access to the trusted switch command.
- Do not expose Alchemy token/RPC URL, raw provider params, signatures, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Unsupported chains, missing Alchemy token, and unavailable Alchemy app networks must fail before mutating session state.
- Manual switching must remain session-local; config files and `.env` are not rewritten.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/capabilities/default.json`
- `apps/framkey-desktop/src-tauri/permissions/autogenerated/framkey_switch_session_chain.toml`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- Focused Rust tests for trusted manual switch result shape and existing switch/probe behavior.
- JS syntax checks for trusted UI.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`, `apps/framkey-desktop/ui/dapp.js`, and provider injection JS: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- Focused desktop tests for trusted UI switching, dApp switching, and chain RPC probe behavior: passed, 5 tests.
- `cargo nextest run -p framkey-desktop`: passed, 49 tests.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.
- Mock Tauri autosmoke passed through main/dApp window creation, account approval, Permit typed-data signing, `personal_sign`, transaction review/signing, portfolio smoke, activity smoke, and the expected unfunded mock-account broadcast error.

## Risks

- Manual switching still depends on the Alchemy app token enabling the target network. The selector is intentionally limited to the current supported-chain allowlist and fails closed when the RPC probe cannot prove the target chain.

# Archived Plan Tail - 2026-06-01

# Remote Uni/Aave Smoke Hardening

Status: completed

Archived from `PLANS.md` during Alchemy RPC health planning.

## Goal

Turn the current Uniswap/Aave support from a documented capability into current-build evidence by running remote dApp smoke flows through the Tauri WebView with mock wallet signing, Alchemy-backed read RPC, trusted approval, and sanitized telemetry.

## Scope

- Run repeatable remote smoke against Uniswap and Aave using `.env`-derived Alchemy RPC and `mock_in_memory` wallet mode.
- Verify provider injection/EIP-6963 discovery, read RPC, account connection, `personal_sign`, Permit typed-data signing, and `eth_sendTransaction` review/signing behavior where the remote page allows the scripted smoke.
- Fix provider, trusted UI, telemetry, or documentation gaps exposed by real remote pages.
- Record current-build evidence in this plan and docs without exposing Alchemy credentials or signed payloads.

## Invariants

- Do not print or commit `.env`, Alchemy token/RPC URL, raw calldata, raw signed transactions, signatures, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Remote dApp smoke must use `mock_in_memory`; it must not touch the real GBxCart card, Keychain vault, or signer-helper secrets.
- A final mock-account insufficient-funds broadcast error is acceptable only after the request reaches trusted review/signing; provider/simulation failures before review must be investigated.
- Keep any compatibility fix scoped to the provider/app boundary; do not fork or vendor dApp code.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Remote Uniswap smoke with `FRAMKEY_DESKTOP_START_URL=uniswap`.
- Remote Aave smoke with `FRAMKEY_DESKTOP_START_URL=aave`.
- JS syntax/provider-injection tests after any frontend/provider changes.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- Focused or full `cargo nextest run -p framkey-desktop` if Rust behavior changes.

Completed verification:

- `FRAMKEY_WALLET_MODE=mock_in_memory FRAMKEY_SIMULATION_PROVIDER=local_decoder_only FRAMKEY_RPC_TIMEOUT_MS=30000 FRAMKEY_DESKTOP_START_URL=uniswap FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE=interactive FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE=1 FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE_DURATION_MS=90000 FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1 cargo run -p framkey-desktop`: passed. The Uniswap WebView received provider injection/EIP-6963 events, Alchemy-backed `eth_blockNumber`, trusted `eth_requestAccounts`, approved `personal_sign`, approved controlled ERC-20 Permit `eth_signTypedData_v4`, and `eth_sendTransaction` review/signing before the expected mock-account insufficient-funds broadcast error.
- `FRAMKEY_WALLET_MODE=mock_in_memory FRAMKEY_SIMULATION_PROVIDER=local_decoder_only FRAMKEY_RPC_TIMEOUT_MS=30000 FRAMKEY_DESKTOP_START_URL=aave FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE=interactive FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE=1 FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE_DURATION_MS=90000 FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1 cargo run -p framkey-desktop`: passed. The Aave WebView received provider injection/EIP-6963 events, Alchemy-backed `eth_blockNumber`, trusted `eth_requestAccounts`, approved `personal_sign`, approved controlled ERC-20 Permit `eth_signTypedData_v4`, and `eth_sendTransaction` review/signing before the expected mock-account insufficient-funds broadcast error.
- No provider or UI compatibility fix was required by these two current-build smoke runs.
- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.

## Risks

- Remote dApps can change deployment, wallet discovery behavior, CSP, or WebView support at any time; record exact current smoke evidence but keep the implementation resilient and fail-closed.

# Trusted Session Network Switching

Status: completed

Archived from `PLANS.md` during Alchemy RPC health planning.

## Goal

Make Uniswap/Aave-style network switching usable without letting an untrusted dApp silently mutate the wallet network: support trusted approval for known Alchemy EVM networks, then update the session chain/RPC config in memory.

## Scope

- Add a trusted review kind for `wallet_switchEthereumChain`.
- Support a small known chain map for Alchemy-backed EVM networks used by common DeFi apps: Ethereum, Sepolia, Base, Optimism, Arbitrum, and Polygon.
- Use `.env`/environment Alchemy token to derive the target read-RPC endpoint when switching networks; explicit single RPC URLs remain fixed and should fail closed for other chains.
- Update trusted status/UI so the active chain label and supported chains are visible.
- Keep the switch session-local; config files and `.env` are not rewritten.

## Invariants

- No silent chain switching from the dApp WebView.
- Do not expose Alchemy token/RPC URL, raw provider params, signatures, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- `eth_sendTransaction` must keep validating transaction `chainId` against the active session chain after a switch.
- Unsupported chains or missing Alchemy token must return a provider error before mutating session config.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests for accepted, rejected, unsupported, and missing-token chain switch paths.
- Provider-injection JS tests for successful and failed chain switch state updates.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Mock Tauri autosmoke to ensure existing DeFi flows still work after the new review kind.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js && node --check apps/framkey-desktop/ui/dapp.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.js && node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs && node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 10 provider-injection tests including rejected switch state.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop switch_chain switch_session wallet_switch`: passed, 3 focused network-switch tests.
- `cargo nextest run -p framkey-desktop`: passed, 46 tests.
- Mock Tauri autosmoke with `.env`-derived Alchemy read RPC passed through provider injection, account approval, Permit typed-data signing, `personal_sign`, transaction review/signing, expected insufficient-funds broadcast failure, portfolio smoke, and transaction activity smoke.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- dApps can request many chain switches; keep the flow explicit, bounded by the existing review TTL, and visible in trusted UI.

# Remote Multi-Chain dApp Smoke

Status: completed

## Goal

Turn the current Uni/Aave remote smoke from default-mainnet evidence into multi-chain evidence by letting the injected provider smoke request a trusted `wallet_switchEthereumChain` to a configured supported chain before the account/sign/transaction path.

## Scope

- Add a sanitized development env setting for the remote smoke target chain.
- Inject that target chain into the untrusted dApp WebView without exposing RPC URLs or Alchemy credentials.
- Have remote provider smoke request `wallet_switchEthereumChain`, verify the provider reports the target chain after approval, then continue the existing interactive account, `personal_sign`, Permit typed-data, and transaction path.
- Keep all network switching behind the existing trusted review broker and supported Alchemy chain allowlist.
- Update focused provider tests and developer docs, then run a remote Uniswap or Aave smoke with a non-default chain.

## Invariants

- No silent chain switching: the dApp request must still be captured and approved by trusted UI before session state mutates.
- Do not print or commit `.env`, Alchemy token/RPC URL, raw calldata, signatures, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Unsupported or malformed target chain settings should not cause a partial session mutation.
- The remote smoke feature remains development-only and safe to run with `mock_in_memory`.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`
- `PLANS.archive.md`

## Verification

- Provider-injection test for configured smoke chain switch and sanitized telemetry.
- `node --check` for changed JS files.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- Focused remote smoke with `mock_in_memory`, `.env` Alchemy, trusted autosmoke, and a non-default supported chain.

Completed verification:

- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`, provider-injection test harness, and unchanged UI JS syntax checks: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop switch_chain switch_session wallet_switch`: passed, 3 tests.
- `cargo nextest run -p framkey-desktop chain_switch_rpc_probe`: passed, 2 tests.
- `cargo nextest run -p framkey-desktop`: passed, 48 tests.
- Base/Arbitrum/OP/Polygon probe check against the local `.env` Alchemy app showed those networks are not enabled for that app; the new switch RPC probe now catches that class before session mutation.
- Uniswap remote multi-chain smoke with `FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE_CHAIN_ID=0xaa36a7` passed through provider injection/EIP-6963, trusted Sepolia switch, switched read RPC, account approval, `personal_sign`, controlled Permit2 typed-data signing, transaction review/signing, and the expected unfunded mock-account broadcast error.

## Risks

- Multi-chain smoke depends on which networks the Alchemy app token enables. Sepolia is currently enabled for the local `.env`; Base/Arbitrum/OP/Polygon need to be enabled in Alchemy before they can be used as live smoke targets.

# Multi-Chain Protocol Registry

Status: completed

## Goal

Make Uni/Aave use less noisy across the chains the Tauri wallet already supports by expanding the counterparty trust registry beyond Ethereum mainnet to the current Alchemy-backed session chains.

## Scope

- Extend known-counterparty labels for Ethereum, Sepolia, Base, OP Mainnet, Arbitrum One, and Polygon.
- Add source-backed Uniswap V2 Router02, V3 SwapRouter/SwapRouter02, Universal Router, Permit2, and Aave V3 Pool entries where available.
- Keep unknown active approval spenders/operators on the explicit high-risk approval path.
- Add focused tests that prove known multi-chain Uni/Aave/Permit2 addresses are recognized and unknown approvals remain high-risk.
- Update docs to describe the registry's chain coverage and source boundary.

## Invariants

- Known labels must not bypass live simulation, signer-helper account checks, approval broker expiry, or decision-token validation.
- Registry entries must be explicit and chain-scoped; do not treat an address as globally trusted unless the protocol publishes it as same-address on those chains.
- Do not display raw params, full calldata, signatures, Alchemy token/RPC URL, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Do not add broad contract verification claims; this remains a conservative review/policy aid.

## Likely Files

- `crates/framkey-simulation/src/lib.rs`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`
- `PLANS.archive.md`

## Verification

- Focused simulation tests for each supported chain's known Uni/Aave entries.
- Focused policy regression test for unknown active approval authority.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-simulation`
- `cargo nextest run -p framkey-simulation`
- `cargo check -p framkey-desktop`
- Mock Tauri autosmoke and debug bundle build.

Completed verification:

- Source lookup used official Uniswap v2/v3/v4 deployment docs and the BGD Labs Aave address book for the supported chains.
- `cargo fmt --all -- --check`: passed.
- Focused simulation registry tests: passed, 4 tests for switchable Uniswap chains, Permit2, Aave pools, known Permit2 approval, and unknown active approval authority.
- `cargo check -p framkey-simulation` and `cargo nextest run -p framkey-simulation`: passed, 23 tests.
- `cargo check -p framkey-desktop` and `cargo nextest run -p framkey-desktop`: passed, 46 tests.
- Mock Tauri autosmoke with `.env` Alchemy read RPC: passed through account, Permit, `personal_sign`, transaction review/signing, Portfolio, Activity, and expected mock-account insufficient-funds broadcast error.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- This registry only covers the current switchable chains and selected Uni/Aave entrypoints; it is not a general contract-verification database.
- Remote dApps can route through aggregators, proxy contracts, or new deployments that will still appear unknown until explicitly reviewed and added.

# Structured Transaction Risk Summary

Status: completed

## Goal

Make transaction review feel less like a raw policy dump by adding a structured risk summary that combines local intent decoding, live simulation state, warnings, blockers, and approval path into one trusted UI surface.

## Scope

- Add a serializable risk summary to `framkey-simulation` transaction review reports.
- Classify blocked, high-risk, caution, and low-risk transaction states without changing signing policy.
- Include concise reason/action fields so the UI can explain ordinary approval, explicit high-risk approval, or hard blocking.
- Render the backend-provided risk summary in the trusted Tauri review card.
- Keep existing policy blockers and raw simulation details available for audit.

## Invariants

- Risk summary is display/review context only; it must not authorize signing or broadcasting.
- Do not expose raw params, full calldata, signatures, Alchemy token/RPC URL, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Unknown selectors, malformed calldata, and provider failures must keep their current policy behavior.
- Local-only protocol decoding must remain high-risk unless live simulation is available.

## Likely Files

- `crates/framkey-simulation/src/lib.rs`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused risk-summary tests for allowed live simulation, local-only DeFi, high-risk approval, malformed calldata, and provider failure.
- JS syntax/provider tests.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-simulation`
- `cargo check -p framkey-desktop`
- Mock Tauri autosmoke and debug bundle build.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed.
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 10 tests.
- `cargo fmt --all -- --check`: passed.
- `cargo nextest run -p framkey-simulation risk`: passed, 2 tests.
- `cargo nextest run -p framkey-simulation uniswap aave`: passed, 5 tests.
- `cargo nextest run -p framkey-simulation`: passed, 15 tests.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 46 tests.
- `FRAMKEY_WALLET_MODE=mock_in_memory FRAMKEY_SIMULATION_PROVIDER=local_decoder_only FRAMKEY_RPC_TIMEOUT_MS=30000 FRAMKEY_DESKTOP_AUTOSMOKE=1 FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1 cargo run -p framkey-desktop`: passed through provider injection, account approval, Permit typed-data signing, `personal_sign`, transaction high-risk review/signing, portfolio smoke, transaction activity smoke, and expected mock-account insufficient-funds broadcast error.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- The summary is only as good as the current local decoder and Alchemy simulation adapter; deeper protocol semantics and allowlists are still future policy work.
- The UI treats backend risk as display context only; signer access remains controlled by policy fields and decision tokens.

# Trusted Portfolio Snapshot

Status: completed

## Goal

Make the trusted Tauri wallet UI behave more like a normal wallet by showing a refreshable portfolio snapshot for the active account: native ETH balance, latest block, and ERC-20 balances discovered through the configured Alchemy RPC.

## Scope

- Add a trusted desktop command for wallet asset snapshots.
- Query native balance and block number through the existing RPC boundary.
- Query ERC-20 balances with Alchemy token APIs when an Alchemy RPC is configured, then enrich nonzero balances with token metadata.
- Render a Portfolio panel in the trusted UI with loading/error/empty states and manual refresh.
- Update docs to describe the portfolio query path and its privacy boundary.

## Invariants

- Do not expose Alchemy token, RPC URL, raw provider params, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Keep the token-balance API trusted-UI only; do not expand the untrusted dApp RPC allowlist with Alchemy-specific methods.
- Portfolio failures must not break account connection, dApp provider injection, signing review, transaction review, or recovery flows.
- Limit metadata fan-out so a wallet with many historical token contracts cannot make the UI hang.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- JS syntax checks.
- Focused Rust tests for portfolio response shaping.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Runtime smoke with `.env`-derived Alchemy RPC and mock wallet.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs` passed 9 tests.
- `echo $RUSTC_WRAPPER` showed `sccache`; `sccache --show-stats` was healthy.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop` passed 33 tests, including portfolio missing-RPC and Alchemy token metadata coverage.
- Mock Tauri autosmoke with `.env`-derived Alchemy RPC passed through trusted/dApp windows, provider injection, account approval, typed-data blocking, `personal_sign`, high-risk tx approval/signing path, expected insufficient-funds broadcast failure, and `trusted_ui_portfolio_smoke` with `ok=true`, `rpc=true`, `errors=0`.
- `cargo tauri build --debug --bundles app --no-sign` produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- Alchemy token-balance responses are provider-specific; keep this behind the trusted Alchemy RPC configuration instead of presenting it as generic EVM RPC behavior.
- Some tokens have missing or malformed metadata; the UI should still show contract address and raw balance.

# Transaction Asset Metadata Review

Status: completed

## Goal

Make transaction confirmations easier to understand for normal DeFi use by enriching decoded ERC-20 approval/transfer reviews with trusted token metadata from the configured Alchemy RPC.

## Scope

- Add best-effort asset context to transaction review summaries for decoded token contracts.
- Query token metadata through the trusted desktop process only, using the existing Alchemy RPC configuration.
- Render approval/transfer amounts with token symbol and decimals when metadata is available.
- Surface partial metadata failure without blocking connection, review capture, signing policy, recovery, or dApp provider flows.
- Keep policy decisions based on simulation/policy evaluation, not on metadata availability.

## Invariants

- Do not expose Alchemy token, RPC URL, raw provider params, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- Do not add Alchemy token APIs to the untrusted dApp provider allowlist.
- Metadata enrichment must be fail-open for display only and fail-closed remains controlled by transaction policy.
- Unknown calldata and high-risk approval warnings must keep their current policy behavior.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests for metadata-enriched transaction summary.
- JS syntax checks and provider-injection tests.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Runtime smoke with `.env`-derived Alchemy RPC and mock wallet.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs` passed 9 tests.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop` passed 34 tests, including display-only `assetContext` policy coverage and transaction metadata enrichment in the provider flow.
- Mock Tauri autosmoke with `.env`-derived Alchemy RPC passed through trusted/dApp windows, provider injection, account approval, typed-data blocking, `personal_sign`, ERC-20 approval transaction review with `assetStatus=ok` and `assetTokenCount=1`, high-risk tx approval/signing path, expected insufficient-funds broadcast failure, and `trusted_ui_portfolio_smoke` with `ok=true`, `rpc=true`, `errors=0`.
- `cargo tauri build --debug --bundles app --no-sign` produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- Token metadata can be missing or misleading; the UI should still include the contract address and not treat metadata as a security proof.

# Typed Data Permit Review

Status: completed

## Goal

Make typed-data requests from DeFi apps safer to inspect by recognizing common ERC-20 Permit and Uniswap Permit2 shapes in the trusted review UI while keeping typed-data signing blocked until a dedicated signer-helper path and stricter policy exist.

## Scope

- Parse EIP-712 typed-data review payloads enough to identify ERC-20 `Permit`, Permit2 `PermitSingle`, `PermitBatch`, `PermitTransferFrom`, and `PermitBatchTransferFrom`.
- Surface owner/spender/token/amount/nonce/deadline fields in the trusted UI.
- Keep raw/truncated typed-data previews available for debugging.
- Keep `eth_signTypedData*` blocked before signing; this task improves review visibility only.
- Update docs and autosmoke evidence for the blocked-but-structured typed-data path.

## Invariants

- Do not add typed-data signing in this slice.
- Do not touch signer-helper secret handling or expand plaintext wallet-secret responsibility.
- Do not expose wallet secret, KEK, DEK, RRK, recovery share bytes, Alchemy token, or RPC URL.
- Unrecognized typed-data must remain captured and blocked with a safe preview.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/dapp.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests for ERC-20 Permit and Permit2 typed-data summaries.
- JS syntax checks and provider-injection tests.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Runtime smoke proving typed-data is still blocked while review summary is structured.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 9 tests.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 36 tests including ERC-20 Permit and Permit2 typed-data summary coverage.
- Mock Tauri autosmoke with `.env`-derived Alchemy RPC passed: provider injection/account approval worked, `eth_signTypedData_v4` was blocked with 4200 while trusted UI showed `typedIntent=erc20_permit`, `personal_sign` succeeded, ERC-20 approval review showed `assetStatus=ok` and `assetTokenCount=1`, high-risk transaction signing reached the expected insufficient-funds broadcast failure, and portfolio smoke reported `ok=true`, `rpc=true`, `errors=0`.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- EIP-712 schemas vary widely; this should remain best-effort review context, not a security proof or signing policy.

# Controlled Typed Data Signing

Status: completed

## Goal

Move DeFi typed-data support from display-only review to real controlled signing for the common Permit path: allow approved ERC-20 Permit and Uniswap Permit2 `eth_signTypedData_v4` requests to sign through the same trusted-review and signer-helper boundary as `personal_sign`, while keeping unknown typed data blocked.

## Scope

- Add an EIP-712 v4 hashing/signing path in `framkey-evm` for structs, nested structs, dynamic strings/bytes, addresses, booleans, unsigned integers, and one-dimensional arrays needed by Permit2.
- Extend signer-helper IPC with a typed-data signing request/response that returns signature and typed-data hash metadata only.
- Add signer-helper and mock-wallet typed-data signing paths without moving plaintext wallet secret handling into the desktop process.
- Change the desktop provider broker so recognized Permit/Permit2 typed-data requests wait for trusted approval and then sign; unrecognized typed data remains captured and blocked.
- Update trusted UI/autosmoke/docs/status capabilities so signed typed-data is visible as product behavior instead of a dry-run.

## Invariants

- Only `eth_signTypedData_v4` with recognized `typedData.intent` may sign in this slice.
- Unknown typed data, raw `eth_sign`, `eth_signTransaction`, malformed payloads, and account mismatches remain blocked.
- The desktop process must not log raw typed-data params, signatures, Alchemy token/RPC URL, wallet secret, KEK, DEK, RRK, or recovery share bytes.
- The Keychain-vault path must keep plaintext wallet secret confined to `framkey-signer-helper`.

## Likely Files

- `crates/framkey-evm/src/lib.rs`
- `crates/framkey-evm/Cargo.toml`
- `crates/framkey-ipc/src/lib.rs`
- `crates/framkey-signer-helper/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/dapp.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo check -p framkey-evm`
- `cargo check -p framkey-signer-helper`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-evm -p framkey-ipc -p framkey-signer-helper -p framkey-desktop`
- JS syntax/provider-injection checks after UI/autosmoke updates.
- Mock Tauri autosmoke proving Permit typed-data signs after approval while unknown typed-data remains blocked.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `node apps/framkey-desktop/src-tauri/src/provider-injection.test.mjs`: passed, 9 tests.
- `echo $RUSTC_WRAPPER`: `sccache`; `sccache --show-stats` reported zero cache errors.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-evm -p framkey-ipc -p framkey-signer-helper -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-evm -p framkey-ipc -p framkey-signer-helper -p framkey-desktop`: passed, 58 tests including EIP-712 signing/recovery, Permit2 arrays, IPC serialization, helper typed-data size limits, review broker authorization, and mock provider typed-data signing.
- Mock Tauri autosmoke with `.env`-derived Alchemy RPC passed: `eth_signTypedData_v4` captured `typedIntent=erc20_permit`, trusted UI approved it, broker mode was `controlled_typed_data_signing`, provider received a signature string, `personal_sign` still succeeded, ERC-20 approval review retained `assetStatus=ok` and `assetTokenCount=1`, high-risk transaction signing reached the expected insufficient-funds broadcast failure, and portfolio smoke reported `ok=true`, `rpc=true`, `errors=0`.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- EIP-712 is easy to get subtly wrong; keep the first signing surface intentionally limited to recognized Permit/Permit2 shapes and add focused hashing/signing tests.

# Transaction Failure Recovery Guidance

Status: completed

## Goal

Make failed or blocked DeFi transactions tell the user what to do next from the trusted Session and Transaction Activity surfaces, not only from raw provider errors.

## Scope

- Add sanitized transaction activity guidance for pending, approved, broadcast, confirmed, reverted, and failed transaction states.
- Classify common broadcast/signing failures such as insufficient gas funds, nonce conflicts, wrong-network/chain mismatches, and reverted execution into actionable next steps.
- Reuse backend transaction review guidance for policy-blocked requests, especially live simulation provider failures.
- Render activity guidance in the trusted UI and let the DeFi Session next action prefer blocked/failed transaction recovery instructions.
- Update docs to describe the user-facing recovery guidance.

## Invariants

- Guidance is display-only and must not authorize signing, retry transactions automatically, or alter policy decisions.
- Do not expose Alchemy token/RPC URL, raw calldata, raw transaction bytes, signatures, wallet secret, KEK, DEK, RRK, recovery root key, or recovery share bytes.
- Keep raw provider error text available in the activity log, but keep guidance copy sanitized and short.
- Preserve deterministic local-simulation smoke and default Alchemy live-simulation behavior.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- Focused Rust tests for blocked live-simulation guidance and insufficient-funds activity guidance.
- JS syntax check for trusted UI.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop activity guidance`
- `cargo nextest run -p framkey-desktop`
- Mock runtime smoke with default Alchemy simulation and deterministic local override.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed after running `cargo fmt --all`.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop activity guidance`: passed, 4 focused tests covering transaction review guidance plus blocked simulation and insufficient-funds activity guidance.
- `cargo nextest run -p framkey-desktop`: passed, 57 tests.
- Default mock runtime smoke with `.env` Alchemy token and no `FRAMKEY_SIMULATION_PROVIDER`: passed account, Permit, `personal_sign`, Portfolio, and RPC Health; status reported `simulation=alchemy_asset_changes`; the unfunded mock transaction was captured and left blocked by live simulation policy rather than auto-signed.
- Deterministic mock runtime smoke with `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`: passed account approval, Permit typed-data signing, `personal_sign`, transaction high-risk review/signing, Portfolio, RPC Health, Transaction Activity smoke, and the expected unfunded mock-account broadcast failure.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- Failure classification is heuristic because RPC providers phrase errors differently; unknown failures should still fall back to a conservative retry/check-RPC instruction.
- This improves recovery UX but does not yet add automated transaction retry, quote refresh, or deeper dApp-specific repair flows.

# Transaction Review Guidance UX

Status: completed

## Goal

Make transaction review clearer for normal DeFi use by showing a trusted, product-level signing guidance summary before raw simulation/policy details.

## Scope

- Add a backend `guidance` object to transaction review summaries derived from policy/risk/simulation state.
- Distinguish ordinary approval, explicit high-risk approval, and blocked/no-signing states with concise titles, messages, and action labels.
- Render the guidance at the top of transaction review cards in the trusted UI.
- Explain disabled transaction approval states without exposing raw calldata, RPC URL/token, signatures, or wallet/recovery secrets.
- Add focused tests for allowed, high-risk override, and blocked simulation-provider failure guidance.

## Invariants

- Guidance is display-only and must not authorize signing or change policy decisions.
- Signer access still depends on existing policy fields, decision tokens, and review broker authorization.
- Do not expose Alchemy token/RPC URL, raw calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, or recovery share bytes.
- Keep raw review summary/debug details available for diagnostics.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- Focused Rust tests for transaction review guidance.
- JS syntax check for trusted UI.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop guidance transaction`
- `cargo nextest run -p framkey-desktop`
- Mock runtime smoke with default Alchemy simulation and deterministic local override.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop guidance transaction`: passed, 12 focused tests including ready, high-risk, and blocked guidance states.
- `cargo nextest run -p framkey-desktop`: passed, 55 tests.
- Default mock runtime smoke with `.env` Alchemy token and no `FRAMKEY_SIMULATION_PROVIDER`: passed account, Permit, and `personal_sign`; status reported `simulation=alchemy_asset_changes`; RPC health reported healthy without token/RPC URL exposure; the unfunded mock transaction was captured and left blocked for manual review instead of auto-signed.
- Deterministic mock runtime smoke with `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`: passed account approval, Permit typed-data signing, `personal_sign`, transaction high-risk review/signing, portfolio smoke, activity smoke, and the expected unfunded mock-account broadcast failure.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- Guidance is display-only; it improves review UX but does not replace deeper protocol-specific semantics or production allowlists.
- The UI copy is intentionally conservative. More detailed remediation can be added once recovery/rotation and production packaging are stronger.

# Alchemy Live Simulation Default

Status: completed

## Goal

Make the Tauri wallet's normal transaction review path use Alchemy live asset-change simulation by default when an Alchemy RPC is configured, so common Uni/Aave transactions can reach ordinary approval only after live simulation succeeds instead of defaulting to high-risk override.

## Scope

- Change desktop simulation config precedence so explicit `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only` still forces local mode, while an available Alchemy RPC/token enables `alchemy_asset_changes` by default.
- Preserve explicit `FRAMKEY_SIMULATION_PROVIDER=alchemy_asset_changes` and JSON config behavior.
- Keep read RPC endpoint/token hidden from status, review, logs, and dApp JavaScript.
- Update docs to describe the new default and the local-only developer override.
- Update focused tests for default live simulation, explicit local override, and status redaction.

## Invariants

- Live simulation failures remain non-overrideable blockers.
- Local-only mode remains available for deterministic mock/development smoke.
- Do not expose Alchemy token/RPC URL, raw calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, or recovery share bytes.
- This change must not grant dApps new permissions or bypass trusted review.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests for token-derived default live simulation, explicit local override, and status redaction.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop simulation alchemy_token`
- `cargo nextest run -p framkey-desktop`
- Mock runtime smoke with `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only` to preserve deterministic UI smoke.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop simulation alchemy_token`: passed, 4 focused tests covering token-derived live simulation default, explicit local override, explicit live provider selection, and status redaction.
- `cargo nextest run -p framkey-desktop`: passed, 53 tests.
- Default mock runtime smoke with `.env` Alchemy token and no `FRAMKEY_SIMULATION_PROVIDER`: passed account, Permit, and `personal_sign` paths; status reported `simulation=alchemy_asset_changes`; RPC health reported healthy without token/RPC URL exposure; the unfunded mock transaction was captured but not auto-approved because live policy blocked it before signing.
- Deterministic mock runtime smoke with `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`: passed account approval, Permit typed-data signing, `personal_sign`, transaction high-risk review/signing, portfolio smoke, activity smoke, and the expected unfunded mock-account broadcast failure.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- Alchemy provider or app/network availability can now block normal transaction signing earlier; this is intentional for the safer default but developers should use `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only` for deterministic smoke and offline UI debugging.
- Explicit generic `FRAMKEY_RPC_URL` endpoints are not assumed to support Alchemy simulation unless an Alchemy token or Alchemy endpoint is available.

# Alchemy RPC Health Surface

Status: completed

## Goal

Make Alchemy's role as the preferred RPC provider visible and testable from the trusted wallet UI, without exposing the token or endpoint to dApp JavaScript or logs.

## Scope

- Add a trusted-window-only RPC health command for the currently configured read RPC.
- Probe `eth_chainId` and `eth_blockNumber`, measure latency, and verify the endpoint matches the active session chain.
- Render a compact RPC health panel in the trusted UI with provider, network, chain match, latest block, latency, and sanitized errors.
- Keep the existing token-derived Alchemy default and explicit RPC override behavior unchanged.
- Add focused tests for missing RPC, healthy RPC, and wrong-chain RPC responses.

## Invariants

- Do not expose or log Alchemy token, full RPC URL, raw provider params, signatures, wallet secret, KEK, DEK, RRK, recovery root key, or recovery share bytes.
- The health check is observational only; it must not change session chain, simulation provider, wallet state, permissions, or transaction policy.
- A wrong-chain or failed health check must be shown as unhealthy, not silently accepted.
- dApp WebViews must not receive direct access to the trusted health command.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/capabilities/default.json`
- `apps/framkey-desktop/src-tauri/permissions/autogenerated/framkey_rpc_health.toml`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`
- `PLANS.archive.md`

## Verification

- Focused Rust tests for RPC missing, healthy chain match, and wrong-chain health result.
- JS syntax checks for trusted UI changes.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Mock runtime smoke using `.env`-derived Alchemy token.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop rpc_health`: passed, 3 focused tests covering missing RPC, healthy chain/block probe, and wrong-chain health result.
- `cargo nextest run -p framkey-desktop`: passed, 53 tests.
- `FRAMKEY_WALLET_MODE=mock_in_memory FRAMKEY_SIMULATION_PROVIDER=local_decoder_only FRAMKEY_RPC_TIMEOUT_MS=30000 FRAMKEY_DESKTOP_AUTOSMOKE=1 FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1 cargo run -p framkey-desktop`: passed with `.env`-derived Alchemy token. The trusted UI emitted `trusted_ui_rpc_health_smoke` with `healthy=true`, `chainMatches=true`, `latestBlock=true`, `tokenExposed=false`, and `rpcUrlExposed=false`; the normal provider smoke reached account approval, Permit typed-data signing, `personal_sign`, transaction review/signing, portfolio smoke, activity smoke, and the expected unfunded mock-account broadcast failure.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- RPC Health is observational only; it does not make live simulation mandatory for ordinary transaction approval and does not replace transaction policy.
- It verifies the currently configured endpoint at check time, so the result can go stale after network/provider changes.

# Recovery Pack Runtime Smoke

Status: completed

## Goal

Add a mock-safe recovery runtime smoke so the desktop app can prove backup-file generation and recovery policy validation without requiring Touch ID, Keychain mutation, or a real GBxCart write.

## Scope

- Add a trusted-window-only development command that creates a disposable standard recovery backup pack in a temp or requested directory.
- Write the same manifest, six recovery share files, and placement guide through the desktop backup writer used by vault creation.
- Validate that cloud-only shares fail and a recommended recovery set passes through the signer helper's read-only recovery drill.
- Let trusted UI autosmoke render the generated recovery pack and drill outcome when explicitly enabled.
- Document the development smoke path and keep it separate from real vault creation.

## Invariants

- This smoke must not touch Keychain, Touch ID, GBxCart, configured vault device contents, or plaintext wallet secrets.
- Do not expose wallet secret, KEK, DEK, RRK, recovery root key, recovery share bytes, Alchemy token/RPC URL, signatures, or raw calldata.
- The real `Create Vault + Recovery` path remains the production path for generating a vault and recovery pack together.
- Generated smoke files must be clearly development artifacts and safe to delete.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/capabilities/default.json`
- `apps/framkey-desktop/src-tauri/permissions/autogenerated/framkey_recovery_smoke_pack.toml`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- Focused Rust test for recovery smoke output, cloud-only failure, recommended set success, and no secret bytes in JSON output.
- JS syntax checks for trusted UI autosmoke changes.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Mock runtime smoke with recovery autosmoke enabled.

Completed verification:

- `cargo build -p framkey-signer-helper`: passed.
- `FRAMKEY_WALLET_MODE=mock_in_memory FRAMKEY_SIMULATION_PROVIDER=local_decoder_only FRAMKEY_RPC_TIMEOUT_MS=30000 FRAMKEY_DESKTOP_AUTOSMOKE=1 FRAMKEY_DESKTOP_RECOVERY_AUTOSMOKE=1 FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR=1 cargo run -p framkey-desktop`: passed with `.env`-derived Alchemy token loaded from the shell. Recovery autosmoke wrote six share files, reported `cloudOnlyCanRecover=false`, `recommendedCanRecover=true`, `walletSecretTouched=false`, and `recoveryShareBytesPrinted=false`; the normal provider smoke reached account approval, Permit typed-data signing, `personal_sign`, transaction review/signing, portfolio smoke, activity smoke, and the expected unfunded mock-account broadcast failure.
- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 50 tests.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- This runtime smoke uses synthetic development recovery material and is not a substitute for the real `Create Vault + Recovery` path against Keychain and the configured card.
- The recovery drill depends on the signer helper binary being available locally.

# Packaged Signer Helper Readiness

Status: completed

## Goal

Make the Tauri wallet app's packaged path explicit and verifiable for the short-lived signer helper, so real Keychain-vault signing does not silently depend on a development-only `target/debug` helper path after bundling.

## Scope

- Prepare a Tauri sidecar artifact for `framkey-signer-helper` during desktop builds when the helper binary has been built.
- Teach the desktop runtime to find the helper next to the app executable or in the bundled app resources, while still supporting explicit config/env overrides.
- Surface sanitized helper readiness in trusted UI/status so the operator can distinguish packaged helper, dev helper, missing helper, and hash-pinned helper states.
- Keep helper invocation, sandboxing, hash pinning, and signer-helper protocol unchanged.
- Update docs with the packaged helper build and verification workflow.

## Invariants

- The untrusted dApp WebView must not gain filesystem, Keychain, GBxCart, or signer-helper access.
- Helper readiness must not print wallet secrets, KEK, DEK, RRK, recovery root key, recovery share bytes, Alchemy token, RPC URL, raw calldata, or signatures.
- Missing helper must fail closed for real Keychain-vault signing and stay explicit in trusted UI status.
- Existing mock-in-memory development flows must keep working without touching Keychain or the card.

## Likely Files

- `apps/framkey-desktop/src-tauri/build.rs`
- `apps/framkey-desktop/src-tauri/tauri.conf.json`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- `cargo build -p framkey-signer-helper`
- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- focused and broad `cargo nextest run -p framkey-desktop`
- runtime smoke in mock mode with helper readiness visible and no secret leakage
- `cargo tauri build --debug --bundles app --no-sign` plus bundle inspection for the helper sidecar

Completed verification:

- `cargo build -p framkey-signer-helper`: passed.
- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed after running `cargo fmt --all`.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop signer_helper`: passed, 3 focused tests.
- `cargo nextest run -p framkey-desktop`: passed, 67 tests.
- Development runtime smoke with `FRAMKEY_WALLET_MODE=mock_in_memory`, `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`, and temporary Activity/Recovery state paths: passed account approval, Permit typed-data signing, `personal_sign`, read-only dApp compatibility check, high-risk transaction review/signing, Transaction Activity smoke, Portfolio, and RPC Health. `trusted_ui_helper_status_smoke` reported `ready=true`, `location=cargo_target`, and `sandbox=macos_sandbox_exec_no_network`.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.
- Bundle inspection found `target/debug/bundle/macos/FRAMKey.app/Contents/MacOS/framkey-signer-helper`; it was executable and byte-identical to `target/debug/framkey-signer-helper`.
- Packaged app binary runtime smoke from `target/debug/bundle/macos/FRAMKey.app/Contents/MacOS/framkey-desktop`: passed the same mock wallet flow, and `trusted_ui_helper_status_smoke` reported `ready=true`, `location=bundled_app`, and `sandbox=macos_sandbox_exec_no_network`.
- Temporary Activity JSON from packaged-app smoke was owner-only `0600`; sensitive keyword scans found no raw transaction fields, Alchemy endpoint/token, wallet secret, recovery root key, or `shareHex`.

## Risks

- Tauri sidecar naming is target-triple-sensitive; runtime discovery should avoid assuming only one bundle location.
- Packaged helper inclusion is not a substitute for production code signing, notarization, hardened runtime, and final entitlements.

# Persistent Recovery Backup Plan UX

Status: completed

## Goal

Keep the generated Recovery backup plan visible after a desktop app restart so a user can continue uploading cloud shares, copying physical shares, revealing generated files, and running recovery drills without needing to recreate the vault or inspect raw command output.

## Scope

- Persist only sanitized trusted-UI recovery operation summaries: public metadata, generated file paths, BLAKE3 hashes, placement roles, drill result status, and recovery rewrap status.
- Restore the last backup plan on startup before the user performs a new recovery operation.
- Keep the existing per-file placement checklist behavior and combine it with restored backup file summaries.
- Do not persist or expose recovery share bytes, wallet secret material, KEK, DEK, RRK, or recovery root key bytes.
- Update docs to describe restart behavior for recovery backup planning.

## Invariants

- Persistence must not authorize recovery rewrap, overwrite a vault device, upload cloud files, copy physical files, or run signer-helper commands.
- Recovery pack writes remain no-overwrite and owner-only; this task only restores the sanitized plan view.
- A corrupt or unavailable restored UI state must fall back to the baseline recovery policy guide.
- The untrusted dApp WebView must not receive the persisted recovery plan or new filesystem abilities.

## Likely Files

- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- focused or broad `cargo nextest run -p framkey-desktop`
- mock runtime smoke with recovery autosmoke and `.env` Alchemy RPC
- debug Tauri bundle build if frontend/runtime changes warrant it

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed after running `cargo fmt --all`.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop recovery_ui_state`: passed, 1 focused test.
- `cargo nextest run -p framkey-desktop`: passed, 65 tests.
- Deterministic mock runtime smoke with `FRAMKEY_DESKTOP_RECOVERY_AUTOSMOKE=1`, `FRAMKEY_DESKTOP_RECOVERY_STATE_PATH=/tmp/framkey-recovery-state-smoke.json`, and `FRAMKEY_DESKTOP_ACTIVITY_PATH=/tmp/framkey-activity-recovery-state-smoke.json`: passed recovery pack generation, dApp account approval, Permit typed-data signing, `personal_sign`, trusted UI read-only compatibility-check smoke, high-risk transaction review/signing, Transaction Activity smoke, Portfolio, RPC Health, and the expected unfunded mock-account broadcast failure. Recovery state and Activity JSON were written as owner-only `0600`, the recovery output directory was `0700`, all generated recovery files were `0600`, and sensitive keyword scans found no raw transaction fields, Alchemy endpoint/token, recovery root key, wallet secret, share bytes, or `shareHex`.
- Restart restore runtime smoke using the same temporary recovery-state path and no recovery autosmoke: passed with `trusted_ui_recovery_state_restored` and `trusted_ui_recovery_state_smoke` reporting restored backup/drill state, `shareFileCount=6`, and no recovered rewrap result.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- Persisted file paths can become stale if the user moves files outside FRAMKey; reveal and drill actions must continue to surface normal missing-file errors.
- UI local state is convenience state, not backup material; the generated recovery files remain the source of truth.

# Local File Permission Hardening

Status: completed

## Goal

Tighten desktop-local file permissions for FRAMKey's privacy- and recovery-relevant disk writes so the Tauri wallet app no longer depends on the user's umask for Activity state or generated recovery packs.

## Scope

- Make the desktop Transaction Activity JSON and its containing directory owner-only on Unix/macOS.
- Make generated Recovery pack files and the recovery output directory owner-only on Unix/macOS.
- Preserve no-overwrite behavior for recovery artifacts and atomic replacement for Activity persistence.
- Keep the placement guide content unchanged, but write it with the same private file mode as the pack it describes.
- Add regression tests for file and directory mode behavior on Unix.
- Update docs to describe local file permission behavior.

## Invariants

- Permission hardening must not expose raw transaction bytes, raw calldata, signatures, Alchemy token/RPC URL, wallet secret, KEK, DEK, RRK, recovery root key, recovery share bytes, or decision tokens.
- Permission changes must not authorize signing, retry transactions, alter recovery policy, or change backup threshold semantics.
- Recovery pack writes must remain no-overwrite and clean up partial files on failure.
- Non-Unix platforms should continue using the existing create/write behavior.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests for Activity persistence permissions and Recovery pack permissions.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Mock runtime smoke with deterministic local simulation and `.env` Alchemy RPC.
- Debug Tauri bundle build.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop file_and_directory_modes`: passed, 2 focused tests.
- `cargo nextest run -p framkey-desktop transaction_activity_persistence`: passed, 4 focused tests.
- `cargo nextest run -p framkey-desktop`: passed, 64 tests.
- Default mock runtime smoke with `.env` Alchemy token and `FRAMKEY_DESKTOP_ACTIVITY_PATH=/tmp/framkey-activity-smoke-default.json`: passed provider injection, account approval, Permit typed-data signing, `personal_sign`, RPC Health, Portfolio, trusted UI read-only compatibility-check smoke, and live-simulation blocking for the unfunded mock transaction. The Activity JSON was written as owner-only `0600` and did not contain raw transaction fields, the Alchemy endpoint/token, or the test calldata marker.
- Deterministic mock runtime smoke with `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`, `FRAMKEY_DESKTOP_RECOVERY_AUTOSMOKE=1`, and `FRAMKEY_DESKTOP_ACTIVITY_PATH=/tmp/framkey-activity-smoke-local.json`: passed account approval, recovery autosmoke, Permit typed-data signing, `personal_sign`, trusted UI read-only compatibility-check smoke, transaction high-risk review/signing, Transaction Activity smoke, Portfolio, RPC Health, and the expected unfunded mock-account broadcast failure. The Activity JSON was written as owner-only `0600`, the recovery output directory was `0700`, all generated recovery files were `0600`, and the Activity JSON did not contain raw transaction fields, the Alchemy endpoint/token, or the test calldata marker.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- macOS cloud folders may preserve POSIX modes differently across sync providers; the local generated source files should still start private before the user places them.
- Pack files remain recoverable backup material and still need deliberate user placement after generation.

# Persistent Transaction Activity

Status: completed

## Goal

Make Transaction Activity survive app restarts so normal wallet users can keep the latest broadcast hash, receipt status, and recovery guidance visible after reopening FRAMKey.

## Scope

- Persist only sanitized `TransactionActivityEntry` records already safe for the trusted UI.
- Load persisted activity on desktop startup before the trusted UI asks for `framkey_transaction_activity`.
- Save activity after review capture, approval/failure, broadcast hash, and receipt refresh updates.
- Include a small persistence status in the activity snapshot and trusted UI so the user can tell whether activity is restored/local-only.
- Keep process-local provider events, review queue contents, dApp grants, and raw debug output non-persistent.
- Update docs to describe the persistence boundary.

## Invariants

- Persisted activity must not include raw transaction bytes, raw calldata, signatures, Alchemy token/RPC URL, wallet secret, KEK, DEK, RRK, recovery root key, recovery share bytes, or decision tokens.
- Persistence is trusted desktop state only; untrusted dApp JavaScript must not get a history API.
- Corrupt or unreadable activity files should not prevent the wallet UI from starting; start with an empty log and surface a sanitized persistence warning.
- Storage writes should be bounded to the existing activity limit.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests for persistence roundtrip, corrupt-file fallback, transient-review restore handling, and secret-string redaction boundaries.
- JS syntax check for trusted UI.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Mock runtime smoke with deterministic local simulation and `.env` Alchemy RPC.
- Debug Tauri bundle build.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed after running `cargo fmt --all`.
- `cargo nextest run -p framkey-desktop transaction_activity_persistence`: passed, 3 focused tests.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 62 tests.
- Default mock runtime smoke with `.env` Alchemy token and `FRAMKEY_DESKTOP_ACTIVITY_PATH=/tmp/framkey-activity-smoke-default.json`: passed provider injection, account approval, Permit typed-data signing, `personal_sign`, RPC Health, Portfolio, trusted UI read-only compatibility-check smoke, and live-simulation blocking for the unfunded mock transaction. The temporary Activity JSON was written and did not contain raw transaction fields, the Alchemy endpoint/token, or the test calldata marker.
- Deterministic mock runtime smoke with `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only` and `FRAMKEY_DESKTOP_ACTIVITY_PATH=/tmp/framkey-activity-smoke-local.json`: passed account approval, Permit typed-data signing, `personal_sign`, trusted UI read-only compatibility-check smoke, transaction high-risk review/signing, Transaction Activity smoke, Portfolio, RPC Health, and the expected unfunded mock-account broadcast failure. The temporary Activity JSON was written with failed Activity status and did not contain raw transaction fields, the Alchemy endpoint/token, or the test calldata marker.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- This is local wallet UX persistence, not a complete transaction indexer; it only tracks activity seen by this FRAMKey desktop app.
- File permissions and app-data placement still need production packaging review alongside macOS signing/sandbox work.

# Transaction Receipt Auto-Tracking

Status: completed

## Goal

Make normal DeFi transaction use feel less manual by letting the trusted Transaction Activity panel automatically poll receipts for recently broadcast transactions instead of requiring the user to know when to press `Refresh Receipts`.

## Scope

- Add trusted UI state that detects broadcast transactions with pending receipts.
- Poll `framkey_transaction_activity` with receipt refresh enabled on a bounded interval while refreshable transaction hashes exist.
- Render a short receipt-tracking status in the Transaction Activity panel so the user can see whether FRAMKey is checking, waiting, confirmed, reverted, failed, or idle.
- Keep the existing manual `Refresh` and `Refresh Receipts` controls.
- Update docs to describe automatic receipt tracking as process-local wallet UX.

## Invariants

- Auto-refresh must not authorize signing, retry transactions, rebroadcast transactions, mutate dApp permissions, or alter policy.
- Receipt polling must stay bounded and must use the existing trusted Alchemy RPC boundary; the dApp must not receive the Alchemy token, RPC URL, raw transaction bytes, signatures, wallet secret, KEK, DEK, RRK, recovery root key, or recovery share bytes.
- Confirmed/reverted/failed/rejected/expired activities should stop receipt polling.
- Manual refresh behavior remains available for diagnostics and recovery from transient UI state.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- JS syntax check for trusted UI.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- Focused or broad `cargo nextest run -p framkey-desktop`
- Mock runtime smoke with deterministic local simulation and `.env` Alchemy RPC.
- Debug Tauri bundle build.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 59 tests.
- Default mock runtime smoke with `.env` Alchemy token and no `FRAMKEY_SIMULATION_PROVIDER`: passed provider injection, account approval, Permit typed-data signing, `personal_sign`, RPC Health, Portfolio, trusted UI read-only compatibility-check smoke, and live-simulation blocking for the unfunded mock transaction.
- Deterministic mock runtime smoke with `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`: passed account approval, Permit typed-data signing, `personal_sign`, trusted UI read-only compatibility-check smoke, transaction high-risk review/signing, Transaction Activity smoke, Portfolio, RPC Health, and the expected unfunded mock-account broadcast failure.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- Receipt polling is still process-local and stops when the app exits.
- A stuck or delayed transaction can remain pending until Alchemy returns a receipt or the user retries from the dApp.

# dApp Compatibility Guidance

Status: completed

## Goal

Turn the Local Test, Uniswap, and Aave compatibility cards from raw run-status grids into product guidance that tells the user what the current evidence means and what to do next.

## Scope

- Add a per-target compatibility guidance summary derived from provider, read RPC, connection, signing, Permit, and transaction evidence.
- Distinguish not checked, checking, provider/read-ready, wallet-connected, needs attention, and usable/complete states.
- Render concise status/action copy on each compatibility card without exposing raw params, calldata, signatures, RPC URL, Alchemy token, or wallet/recovery secrets.
- Keep the existing step grid and raw provider events for diagnostics.
- Update docs to describe the guidance layer.

## Invariants

- Guidance is display-only and must not grant dApp permissions, authorize signing, switch networks, retry transactions, or alter policy.
- Remote dApp content remains untrusted.
- Do not expose Alchemy token/RPC URL, raw params, calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, or recovery share bytes.
- Existing compatibility evidence and telemetry remain process-local.

## Likely Files

- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- JS syntax check for trusted UI.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`
- Mock runtime smoke with default Alchemy simulation and deterministic local override.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 59 tests.
- Default mock runtime smoke with `.env` Alchemy token and no `FRAMKEY_SIMULATION_PROVIDER`: passed account, Permit, `personal_sign`, Portfolio, RPC Health, trusted UI read-only compatibility-check smoke, and live-simulation blocking for the unfunded mock transaction.
- Deterministic mock runtime smoke with `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`: passed account approval, Permit typed-data signing, `personal_sign`, trusted UI read-only compatibility-check smoke, transaction high-risk review/signing, Portfolio, RPC Health, Transaction Activity smoke, and the expected unfunded mock-account broadcast failure.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- Guidance is derived from bounded process-local evidence, so it can become stale after a dApp reload or app restart.
- A read-only check cannot prove signing/transaction compatibility; it only proves provider injection and read RPC.

# Trusted dApp Compatibility Check

Status: completed

## Goal

Make Uni/Aave compatibility checks available from the trusted Tauri UI instead of requiring startup environment variables and stderr telemetry.

## Scope

- Add a trusted-window-only command that asks the current untrusted dApp WebView to run a read-only FRAMKey provider probe.
- Expose a safe provider-injection entry point for read-only compatibility probes after page load.
- Add `Check` actions to the dApp Compatibility cards so the user can open Local Test, Uniswap, or Aave and collect provider/RPC evidence from the UI.
- Keep the check read-only by default: provider injection, `eth_chainId`, `eth_accounts`, and `eth_blockNumber`.
- Continue rendering results through the existing Provider Events and Compatibility panels.

## Invariants

- The UI-triggered check must not sign, request account approval, send transactions, mutate network, or expose secrets.
- The command must be restricted to the trusted main window.
- Remote dApp content remains untrusted and receives no direct Keychain, filesystem, GBxCart, recovery, or signer-helper access.
- Do not expose Alchemy token/RPC URL, raw params, calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, or recovery share bytes.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/provider-injection.js`
- `apps/framkey-desktop/src-tauri/capabilities/default.json`
- `apps/framkey-desktop/src-tauri/permissions/autogenerated/framkey_run_dapp_compatibility_check.toml`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- Focused Rust tests for compatibility-check request normalization.
- JS syntax checks for trusted UI and provider injection.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop compatibility check`
- `cargo nextest run -p framkey-desktop`
- Mock runtime smoke to ensure existing local and Alchemy paths still work.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/src-tauri/src/provider-injection.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop compatibility check`: passed, 3 focused tests including read-only default and interactive-mode rejection.
- `cargo nextest run -p framkey-desktop`: passed, 59 tests.
- Default mock runtime smoke with `.env` Alchemy token and no `FRAMKEY_SIMULATION_PROVIDER`: passed account, Permit, `personal_sign`, Portfolio, RPC Health, trusted UI read-only compatibility-check smoke, and live-simulation blocking for the unfunded mock transaction.
- Deterministic mock runtime smoke with `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`: passed account approval, Permit typed-data signing, `personal_sign`, trusted UI read-only compatibility-check smoke, transaction high-risk review/signing, Portfolio, RPC Health, Transaction Activity smoke, and the expected unfunded mock-account broadcast failure.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- The trusted UI command can only start the probe; evidence still arrives asynchronously through provider telemetry.
- Some remote dApps may navigate or delay page initialization, so the UI check may need a retry after the page settles.

# Trusted dApp Navigation Controls

Status: completed

## Goal

Make the Tauri DeFi Browser feel like a usable wallet browser instead of a launch-only panel by surfacing trusted dApp navigation state and adding basic reload/back/forward/home controls.

## Scope

- Track the current dApp WebView target, sanitized current URL, origin, load status, and update time in trusted process state.
- Add trusted-main-window-only commands for reading the dApp session state and issuing bounded navigation actions.
- Render current dApp state in the DeFi Browser panel and keep the DeFi Session panel aligned with the actual WebView state.
- Keep Local Test, Uniswap, Aave, and custom `http`/`https` open flows on the existing URL validation path.
- Update docs to describe the trusted navigation layer.

## Invariants

- Remote dApp content remains untrusted and must not gain filesystem, Keychain, GBxCart, recovery, signer-helper, token, or raw RPC URL access.
- Navigation state must be sanitized: no query string, no fragment, no Alchemy token/RPC URL, no raw calldata, no signatures, and no wallet/recovery secrets.
- Navigation controls must not grant account permission, sign, send transactions, switch chains, or approve requests.
- Existing compatibility checks and provider-event telemetry remain process-local and bounded.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/capabilities/default.json`
- `apps/framkey-desktop/src-tauri/permissions/autogenerated/`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- Focused Rust tests for dApp state sanitization and navigation-action validation.
- JS syntax check for trusted UI.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop dapp`
- Runtime mock smoke with `.env` Alchemy RPC and local-only simulation.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop dapp`: passed, 5 focused dApp tests.
- `cargo nextest run -p framkey-desktop`: passed, 70 tests.
- Runtime mock smoke with `FRAMKEY_WALLET_MODE=mock_in_memory`, `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`, `FRAMKEY_RPC_TIMEOUT_MS=30000`, and repo `.env` Alchemy token: passed dApp WebView load, provider injection, trusted dApp session snapshot with `queryExposed=false` and `fragmentExposed=false`, RPC Health, Portfolio, account approval, Permit typed-data signing, `personal_sign`, transaction review/signing, and the expected unfunded mock-account broadcast failure. Temporary Activity state was owner-only `0600`; sensitive keyword scans found no Alchemy endpoint/token, raw transaction fields, signatures, wallet secret, or recovery share bytes.

## Risks

- WebView history availability is platform-controlled; back/forward controls can request browser history movement but may no-op when history is empty.
- Page-load events can be asynchronous, so the trusted UI may briefly show the requested target before the finished URL arrives.

# Recovery Set Builder UX

Status: completed

Archived from `PLANS.md` on 2026-06-01 to keep the active plan file focused.

## Goal

Make recovery drill/rewrap safer to operate from the Tauri UI by replacing the bare recovery-file textarea workflow with guided set selection and live policy feedback.

## Scope

- Add trusted UI controls that fill the recovery file list from the latest generated/restored backup plan using documented valid combinations.
- Show live status for the current recovery file input: file count, whether it matches the current backup plan, and whether it satisfies cloud-plus-physical or local-plus-remote recovery policy.
- Keep manual path entry for moved/imported files, but make its status explicit before the user runs `Check Recovery Set` or `Recover Keychain Vault`.
- Disable action buttons when no file paths are present, and keep the explicit overwrite checkbox required for recovery rewrap.
- Update docs to describe the guided recovery set builder.

## Invariants

- The UI must never parse or display recovery share bytes, recovery root key bytes, wallet secret, KEK, DEK, RRK, raw transaction data, signatures, Alchemy token, or RPC URL.
- The builder is display/input help only; the signer helper remains the authority for read-only recovery drill and recovery rewrap validation.
- Cloud-only share sets must remain visibly insufficient.
- Manual path entry must remain possible for files moved outside the original generated directory.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- JS syntax check for trusted UI.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop recovery`
- Runtime mock recovery autosmoke.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop recovery`: passed, 5 focused recovery tests.
- `cargo nextest run -p framkey-desktop`: passed, 70 tests.
- Runtime mock recovery autosmoke with `FRAMKEY_WALLET_MODE=mock_in_memory`, `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`, `FRAMKEY_DESKTOP_RECOVERY_AUTOSMOKE=1`, `FRAMKEY_RPC_TIMEOUT_MS=30000`, and temporary Activity/Recovery state paths: passed recovery pack generation, cloud-only failure, recommended-set recovery drill success, dApp account approval, Permit typed-data signing, `personal_sign`, transaction review/signing, RPC Health, Portfolio, and the expected unfunded mock-account broadcast failure. Temporary state files and recovery pack directory were owner-only and cleaned after verification.

## Risks

- If the user moves backup files, path matching against the latest plan may be unknown; the UI should still let the helper validate manually entered paths.
- This does not replace a future native file picker; it makes the current generated-plan workflow safer without adding plugin/dependency churn.

# Trusted Recovery File Picker

Status: completed

Archived from `PLANS.md` on 2026-06-01 to keep the active plan file focused.

## Goal

Reduce manual path entry in the Recovery workspace by adding trusted native file/folder selection for recovery share files and recovery output directories.

## Scope

- Add trusted-main-window-only commands for selecting recovery share files and a recovery output directory on macOS.
- Wire Recovery UI buttons to fill the recovery file list and recovery output directory fields.
- Keep manual path entry and generated-plan set builder controls available.
- Return selected paths only to the trusted UI, never to the untrusted dApp WebView.
- Document the picker as current macOS product UX and keep future Tauri dialog/plugin replacement possible.

## Invariants

- The picker must not read selected file contents, parse recovery share bytes, or return recovery share material.
- The untrusted dApp WebView must not gain filesystem, dialog, Keychain, GBxCart, signer-helper, Alchemy token/RPC URL, raw transaction, signature, wallet secret, KEK, DEK, RRK, recovery root key, or recovery share access.
- Recovery drill/rewrap validation remains inside the existing signer-helper path.
- User cancellation should be a non-error result so the UI can leave existing paths unchanged.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/capabilities/default.json`
- `apps/framkey-desktop/src-tauri/permissions/autogenerated/`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests for picker output parsing and cancellation classification.
- JS syntax check for trusted UI.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop picker`
- Runtime mock recovery autosmoke to ensure recovery flows are unchanged.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop picker`: passed, 3 focused picker tests.
- `cargo nextest run -p framkey-desktop recovery`: passed, 8 focused recovery tests.
- `cargo nextest run -p framkey-desktop`: passed, 73 tests.
- Runtime mock recovery autosmoke with `FRAMKEY_WALLET_MODE=mock_in_memory`, `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`, `FRAMKEY_DESKTOP_RECOVERY_AUTOSMOKE=1`, `FRAMKEY_RPC_TIMEOUT_MS=30000`, and repo `.env` Alchemy token: passed recovery smoke, dApp session smoke, RPC Health with token/RPC URL not exposed, Activity smoke, account approval, Permit typed-data signing, `personal_sign`, transaction review/signing, and the expected unfunded mock-account broadcast failure. Temporary Activity/Recovery state files and recovery pack directory were owner-only and cleaned after verification.

## Risks

- The first implementation is macOS-specific, matching the current Keychain/Touch ID product path; future production sandboxing may prefer a Tauri dialog plugin.
- Automated tests should not invoke a real blocking file picker; they should cover parsing and trusted command wiring while runtime smoke covers adjacent recovery behavior.

# Archived 2026-06-01 Desktop wallet completed plans

# Known-Chain Add Network Compatibility

Status: completed

## Goal

Improve common DeFi multi-chain compatibility by supporting `wallet_addEthereumChain` for FRAMKey's known Alchemy-backed chains without trusting dApp-supplied RPC endpoints.

## Scope

- Treat `wallet_addEthereumChain` as a trusted-review-gated network management request.
- Accept only chains already in the FRAMKey supported Alchemy chain set.
- Derive and verify the RPC endpoint from the local Alchemy token before completing the request.
- Return `null` on success without persisting dApp chain metadata or rewriting config files.
- Keep `wallet_switchEthereumChain` as the operation that changes the active session chain.
- Add a local test dApp control for add-chain compatibility checks.
- Update docs and roadmap to describe the supported method and security boundary.

## Invariants

- The dApp-provided `rpcUrls`, block explorer URLs, chain name, and currency metadata must not become trusted wallet configuration.
- The untrusted dApp WebView must not receive Alchemy token/RPC URL, filesystem, Keychain, GBxCart, signer-helper, wallet secret, KEK, DEK, RRK, recovery root key, recovery share bytes, raw transaction, or signature access.
- Unsupported chains, missing Alchemy token, invalid params, and endpoint verification failures must fail before any session/network mutation.
- `wallet_addEthereumChain` must not silently switch chains; active network changes remain explicit through trusted `wallet_switchEthereumChain` or the trusted Wallet workspace selector.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/ui/dapp.html`
- `apps/framkey-desktop/ui/dapp.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests for add-chain review, unsupported/missing-token failures, and no session switch on successful add.
- JS syntax check for the local dApp test control.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop chain`
- Runtime mock smoke with `.env` Alchemy RPC and deterministic local simulation.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo fmt --all -- --check`: passed after running `cargo fmt --all`.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop chain`: passed, 11 focused chain/RPC tests including `wallet_addEthereumChain`.
- `cargo nextest run -p framkey-desktop`: passed, 75 tests.
- Runtime mock recovery autosmoke with `FRAMKEY_WALLET_MODE=mock_in_memory`, `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`, `FRAMKEY_DESKTOP_RECOVERY_AUTOSMOKE=1`, `FRAMKEY_RPC_TIMEOUT_MS=30000`, and repo `.env` Alchemy token: passed dApp session smoke, RPC Health with token/RPC URL not exposed, recovery smoke, Activity smoke, account approval, Permit typed-data signing, `personal_sign`, transaction review/signing, and the expected unfunded mock-account broadcast failure. Temporary Activity/Recovery state files and recovery pack directory were owner-only and cleaned after verification.

## Risks

- Some dApps may expect add-chain and switch-chain as separate calls; returning success without switching is closer to the wallet-add spec but should be watched in remote compatibility telemetry.
- The current supported chain list is intentionally narrow and Alchemy-backed; broader chain support should remain explicit rather than accepting arbitrary dApp RPC metadata.

# Trusted Watch Asset Compatibility

Status: completed

## Goal

Improve normal DeFi wallet UX by supporting trusted `wallet_watchAsset` requests for ERC-20 tokens and surfacing approved watched tokens in the Portfolio panel.

## Scope

- Accept `wallet_watchAsset` only for ERC-20 assets with valid address, symbol, and decimals.
- Capture watch-asset requests in the trusted review queue and require explicit user approval.
- Store approved watched assets in process-local trusted state keyed by chain and contract.
- Merge watched zero-balance tokens into the trusted Portfolio view without treating dApp metadata as policy input.
- Add a local dApp test button for watch-asset compatibility checks.
- Update docs and roadmap to describe the method and trust boundary.

## Invariants

- `wallet_watchAsset` must not grant account access, sign, submit transactions, switch networks, read files, access Keychain/GBxCart/signer-helper, or expose Alchemy token/RPC URL.
- DApp-provided token metadata is display-only and must not affect transaction policy, signer access, or RPC configuration.
- The dApp must not receive a watched-asset list or any portfolio/history API.
- Malformed tokens, unsupported asset types, control characters, and oversized fields must fail before entering the review queue.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/review.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/dapp.html`
- `apps/framkey-desktop/ui/dapp.js`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests for watch-asset validation, approval storage, portfolio merge, and malformed request rejection.
- JS syntax checks for trusted UI and local dApp.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop watch_asset`
- Runtime mock smoke with `.env` Alchemy RPC and deterministic local simulation.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo fmt --all -- --check`: passed after running `cargo fmt --all`.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop watch_asset`: passed, 2 focused watch-asset tests.
- `cargo nextest run -p framkey-desktop`: passed, 77 tests.
- Runtime mock recovery autosmoke with `FRAMKEY_WALLET_MODE=mock_in_memory`, `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`, `FRAMKEY_DESKTOP_RECOVERY_AUTOSMOKE=1`, `FRAMKEY_RPC_TIMEOUT_MS=30000`, and repo `.env` Alchemy token: passed dApp session smoke, RPC Health with token/RPC URL not exposed, recovery smoke, account approval, `wallet_watchAsset` approval, watched Portfolio merge with `watched=1` and `tokenCount=1`, Permit typed-data signing, `personal_sign`, transaction review/signing, Activity smoke, and the expected unfunded mock-account broadcast failure. Temporary Activity/Recovery state files and recovery pack directory were owner-only and cleaned after verification.

## Risks

- This first slice keeps watched assets process-local; persistent watched tokens can follow once the UX and compatibility behavior are proven.
- DApp-provided symbols can be misleading, so Portfolio must show the contract address and distinguish watched tokens from Alchemy-discovered nonzero balances.

# Persistent Watched Token Wallet State

Status: completed

## Goal

Make approved `wallet_watchAsset` ERC-20 tokens survive app restarts so Portfolio behaves like a normal wallet token list instead of forgetting user-approved watched assets after every process exit.

## Scope

- Persist only trusted-public watched ERC-20 metadata approved through the existing trusted review path.
- Restore watched assets on desktop startup before Portfolio snapshots are rendered.
- Keep account permissions, provider events, pending review queue, compatibility evidence, raw params, calldata, signatures, and dApp session state process-local.
- Expose sanitized persistence status in the Portfolio payload and UI so the user can see whether watched tokens were restored or whether local wallet-state persistence needs attention.
- Keep owner-only Unix/macOS file and directory permissions consistent with Activity and Recovery state.
- Add tests for sanitized roundtrip, corrupt-file fallback, private file mode, and restarted Portfolio merge.
- Update docs and roadmap with the new persistence boundary.

## Invariants

- Persisted wallet state must not include Alchemy token/RPC URL, raw provider params, calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, recovery share bytes, decision tokens, account grants, or dApp transaction history.
- DApp-provided watched-token metadata remains display-only and must not affect transaction policy, signer access, RPC configuration, account exposure, or network switching.
- Malformed or unsupported persisted watched-token entries must be dropped or surfaced as a sanitized warning without blocking wallet startup.
- The untrusted dApp WebView must not receive a watched-token list or any new wallet-state API.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests for watched-token persistence and Portfolio restore behavior.
- JS syntax check for trusted UI.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop wallet_state`
- `cargo nextest run -p framkey-desktop`
- Runtime mock recovery autosmoke with `.env` Alchemy RPC and deterministic local simulation.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo fmt --all -- --check`: passed after running `cargo fmt --all`.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop wallet_state`: passed, 3 focused wallet-state tests.
- `cargo nextest run -p framkey-desktop watch_asset`: passed, 3 focused watch-asset tests including restart restore.
- `cargo nextest run -p framkey-desktop`: passed, 81 tests.
- Runtime mock recovery autosmoke with `FRAMKEY_WALLET_MODE=mock_in_memory`, `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`, `FRAMKEY_DESKTOP_RECOVERY_AUTOSMOKE=1`, `FRAMKEY_RPC_TIMEOUT_MS=30000`, repo `.env` Alchemy token, and temporary Activity/Recovery/Wallet state paths: passed dApp session smoke, RPC Health with token/RPC URL not exposed, recovery smoke, account approval, `wallet_watchAsset` approval, watched Portfolio merge with `watched=1` and `tokenCount=1`, Permit typed-data signing, `personal_sign`, transaction review/signing, Activity smoke, and the expected unfunded mock-account broadcast failure. Temporary Activity/Recovery/Wallet state files and recovery pack directory were owner-only; wallet-state contained only approved USDC watched-token metadata and no Alchemy endpoint/token, decision token, wallet secret, recovery root key, share hex, raw transaction, or test calldata marker. Temporary files were cleaned after verification.

## Risks

- DApp-provided symbols can become stale or misleading; Portfolio must keep showing the contract address and watched marker.
- Persistence is local trusted UI state only; deleting the local state file should not affect the vault or any recovery backup files.

# Trusted Native Send Flow

Status: completed

## Goal

Make FRAMKey usable as a wallet without relying on a dApp by adding a trusted Wallet workspace flow for sending a native-token transfer through the same review, signer-helper/mock signing, broadcast, and Transaction Activity pipeline used by `eth_sendTransaction`.

## Scope

- Add a trusted-main-window-only command for native transfers with recipient address and decimal native amount input.
- Validate recipient, amount, chain, and RPC readiness before capture.
- Reuse existing transaction preparation, policy review, user approval, signing, broadcast, and Activity recording behavior.
- Add a Wallet UI send form with recipient, amount, send status, and post-send refresh behavior.
- Keep this first native wallet flow limited to native-token transfers with no calldata.
- Update docs and roadmap.

## Invariants

- The untrusted dApp WebView must not gain a native-send command, filesystem access, Keychain/GBxCart/signer-helper access, Alchemy token/RPC URL, or wallet/recovery secret access.
- Native send must still require trusted review approval before signing.
- The desktop process must not log or persist raw signed transactions, raw calldata, signatures, Alchemy credentials, wallet secret, KEK, DEK, RRK, recovery root key, recovery share bytes, or decision tokens.
- Failed broadcasts must still be captured in Transaction Activity with sanitized recovery guidance.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/capabilities/default.json`
- `apps/framkey-desktop/src-tauri/permissions/autogenerated/framkey_send_native_transfer.toml`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests for amount parsing, validation, review capture, signing/broadcast, and Activity recording.
- JS syntax check for trusted UI.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop native_send`
- `cargo nextest run -p framkey-desktop`
- Runtime mock recovery autosmoke with `.env` Alchemy RPC and deterministic local simulation.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed after `cargo fmt --all`.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop native_send`: passed, 3 focused tests for conservative amount parsing, invalid request rejection before review, and review/sign/broadcast/Activity recording.
- `cargo nextest run -p framkey-desktop`: passed, 84 tests.
- Runtime mock recovery autosmoke with `FRAMKEY_WALLET_MODE=mock_in_memory`, `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`, `.env` Alchemy RPC, and temporary Activity/Recovery/Wallet state paths: passed window/session smoke, real RPC Health with token/RPC URL not exposed, recovery smoke, account approval, `wallet_watchAsset`, Portfolio watched-token merge, Permit typed-data signing, `personal_sign`, transaction review/signing, Activity smoke, and the expected unfunded mock-account broadcast failure. Native-send backend behavior is covered by focused Rust tests; this runtime smoke does not auto-submit the trusted native-send form.
- Temporary Activity/Recovery/Wallet state files were owner-only. A sensitive-field scan found no Alchemy endpoint/token, decision token, raw transaction, share hex, private key, signature, or recovery root key; the only `alchemy` match was public token image metadata from an approved watched asset. Temporary smoke files were cleaned after verification.

## Risks

- Decimal input parsing must be conservative and reject ambiguous formats rather than guessing user intent.
- This first Wallet-native send path covers native transfers only; ERC-20 send can follow after token selection, decimals, and approval semantics are designed.

# Trusted ERC-20 Send Flow

Status: completed

## Goal

Let the trusted Wallet workspace send ERC-20 tokens directly from Portfolio, closing the normal wallet-use gap between read-only token discovery and dApp-driven transactions.

## Scope

- Add a trusted-main-window-only command for ERC-20 `transfer(address,uint256)` with token contract, recipient, decimal amount, token decimals, symbol, and chain id.
- Validate contract, recipient, amount, decimals, chain, and RPC readiness before creating a review request.
- Encode transfer calldata locally and reuse the existing transaction preparation, policy review, user approval, signer-helper/mock signing, broadcast, and Transaction Activity pipeline.
- Add Portfolio token send controls that prefill the selected token, display status, and refresh Portfolio/Activity after completion.
- Update docs and roadmap.

## Invariants

- The untrusted dApp WebView must not gain the token-send command or any filesystem, signer-helper, Keychain, GBxCart, Alchemy credential, wallet-secret, or recovery-secret access.
- ERC-20 send must require trusted review approval before signing.
- Token metadata from Portfolio is display/input context only; the token contract and encoded calldata determine what is signed.
- The desktop process must not log or persist raw signed transactions, signatures, Alchemy credentials, wallet secret, KEK, DEK, RRK, recovery root key, recovery share bytes, or decision tokens.
- Failed broadcasts must still be represented in Transaction Activity with sanitized guidance.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/capabilities/default.json`
- `apps/framkey-desktop/src-tauri/permissions/autogenerated/framkey_send_token_transfer.toml`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- Focused Rust tests for token decimal parsing/encoding, validation-before-review, review capture, signing/broadcast, and Activity recording.
- JS syntax check for trusted UI and dApp script.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop token_send`
- `cargo nextest run -p framkey-desktop`
- Runtime mock recovery autosmoke with `.env` Alchemy RPC and deterministic local simulation.
- Debug Tauri bundle build.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop token_send`: passed, 3 focused tests for decimal/raw uint256 encoding, ERC-20 calldata, invalid request rejection before review, and review/sign/broadcast/Activity recording.
- `cargo nextest run -p framkey-desktop`: passed, 87 tests.
- Runtime mock recovery autosmoke with `FRAMKEY_WALLET_MODE=mock_in_memory`, `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`, `.env` Alchemy RPC, and temporary Activity/Recovery/Wallet state paths: passed window/session smoke, real RPC Health with token/RPC URL not exposed, recovery smoke, account approval, `wallet_watchAsset`, Portfolio watched-token merge, Permit typed-data signing, `personal_sign`, transaction review/signing, Activity smoke, and the expected unfunded mock-account broadcast failure. ERC-20 send backend behavior is covered by focused Rust tests; this runtime smoke does not auto-submit the trusted token-send form.
- Temporary Activity/Recovery/Wallet state files were owner-only. A sensitive-field scan found no Alchemy endpoint/token, decision token, raw transaction, share hex, private key, signature, or recovery root key; the only `alchemy` match was public token image metadata from an approved watched asset. Temporary smoke files were cleaned after verification.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.

## Risks

- ERC-20 decimal metadata can be wrong; the UI must keep the contract visible and the review must show the exact transfer intent decoded from calldata.
- This covers direct ERC-20 transfer only; approval/allowance-management UX remains a separate product decision.

# Trusted Wallet Send UI Autosmoke

Status: completed

## Goal

Prove the trusted Wallet send forms themselves can submit native and ERC-20 transfers through the same UI controls a user sees, not only through backend unit tests.

## Scope

- Add an explicit development-only `FRAMKEY_DESKTOP_WALLET_SEND_AUTOSMOKE=1` capability.
- In mock wallet autosmoke, fill and submit the trusted native send form with a tiny native amount.
- After `wallet_watchAsset` populates Portfolio, select a sendable ERC-20 token and submit the trusted token send form.
- Report sanitized smoke events for started/skipped/result states without printing raw transactions, signatures, Alchemy credentials, wallet secrets, or recovery material.
- Update docs so this heavier smoke stays opt-in.

## Invariants

- Wallet-send autosmoke must only run with mock wallet status and the explicit wallet-send autosmoke flag.
- The untrusted dApp WebView must not gain native/token send commands or direct access to trusted UI form state.
- Autosmoke must not bypass review; it should rely on the same trusted review queue and auto-approval loop as manual smoke.
- Smoke event details must remain sanitized and should not include raw calldata, raw signed transactions, signatures, decision tokens, Alchemy endpoint/token, wallet secret, KEK, DEK, RRK, recovery root key, or recovery share bytes.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- JS syntax check for trusted UI and dApp script.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- Focused or full `cargo nextest run -p framkey-desktop`
- Runtime mock recovery autosmoke with `.env` Alchemy RPC, deterministic local simulation, and `FRAMKEY_DESKTOP_WALLET_SEND_AUTOSMOKE=1`.
- Sensitive-field scan of temporary smoke state.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop token_send native_send`: passed, 6 focused native/token send tests.
- `cargo nextest run -p framkey-desktop`: passed, 87 tests.
- Runtime mock recovery autosmoke with `FRAMKEY_WALLET_MODE=mock_in_memory`, `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`, `FRAMKEY_DESKTOP_RECOVERY_AUTOSMOKE=1`, `FRAMKEY_DESKTOP_WALLET_SEND_AUTOSMOKE=1`, `.env` Alchemy RPC, and temporary Activity/Recovery/Wallet state paths: passed dApp smoke, recovery smoke, RPC health, watched-token Portfolio merge, native send form submission, token send form submission, review auto-approval, signing, broadcast attempts, Activity recording, and expected unfunded mock-account failures.
- Temporary Activity/Recovery/Wallet state files were owner-only. Activity contained three sanitized failed transactions: trusted ERC-20 transfer, trusted native transfer, and local dApp approval transaction. Sensitive-field scan found no Alchemy endpoint/token, decision token, raw transaction, share hex, private key, signature, or recovery root key; the only `alchemy` match was public token image metadata from an approved watched asset. Temporary smoke files were cleaned after verification.

## Risks

- This smoke intentionally submits extra mock transactions and can take longer on slow RPC, so it should remain opt-in.
- The token-send smoke depends on a sendable watched token being visible in Portfolio after the local dApp `wallet_watchAsset` flow.

# Product UI Visual QA
Status: completed

## Goal

Run the Tauri wallet as a real desktop app and use screen-level evidence to improve product usability across Wallet, DeFi, and Recovery instead of relying only on backend tests and smoke logs.

## Scope

- Start the desktop app in mock mode with development smoke data for account, Portfolio, transaction Activity, and recovery backup state.
- Inspect Wallet, DeFi, and Recovery workspaces through real app screenshots/accessibility state.
- Fix concrete visual/layout issues that affect scanning, text fit, button placement, or common workflow clarity.
- Keep visual fixes scoped to trusted desktop UI files and documentation/plan notes.

## Invariants

- Do not perform real transactions, Keychain unlocks, Touch ID prompts, GBxCart writes, cloud uploads, or filesystem deletion through GUI actions.
- Use mock wallet and temporary state paths for visual QA.
- Keep dApp WebView untrusted and do not grant it new filesystem, signer-helper, recovery, Alchemy credential, or wallet-secret access.
- Do not expose Alchemy endpoint/token, raw params, calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, or recovery share bytes in screenshots or persisted diagnostics.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- Computer Use or screenshot inspection of Wallet, DeFi, and Recovery workspaces.
- JS syntax check for trusted UI and dApp script.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- Focused or full `cargo nextest run -p framkey-desktop`
- Runtime mock smoke with wallet-send and recovery autosmoke after visual fixes.

Completed verification:

- `cargo tauri build --debug --bundles app --no-sign`: passed and refreshed `target/debug/bundle/macos/FRAMKey.app`.
- Runtime mock smoke with `FRAMKEY_WALLET_MODE=mock_in_memory`, `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only`, recovery autosmoke, wallet-send autosmoke, `.env` Alchemy RPC, and temporary Activity/Recovery/Wallet state paths: passed recovery backup/drill smoke, dApp smoke, RPC health, watched-token Portfolio merge, native send form, token send form, review auto-approval, and expected unfunded mock-account failures.
- Computer Use visual QA against the rebuilt bundle: Wallet rendered account/RPC/Portfolio/send/activity state coherently; DeFi now opens with DeFi Browser first and DeFi Session visible before Activity/Review; Recovery still opens with Create/Recover and the Recovery Backup Plan visible.
- Request Review is now ordered after Activity and bounded with an internal scroll area so accumulated smoke reviews do not expand the workspace indefinitely.
- Temporary Activity/Recovery/Wallet state files were owner-only `0600`; sensitive scan found no Alchemy endpoint/token, decision token, raw transaction, share hex, private key, signature, wallet secret, or recovery root key bytes. The only sensitive-keyword hits were `recoveryRootKeyPrinted=false` booleans.
- Temporary smoke state and recovery smoke files were cleaned after verification.
- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `echo $RUSTC_WRAPPER`: `sccache`.
- `sccache --show-stats`: available and healthy.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 87 tests.
- `git diff --check`: passed for tracked diff; relevant files are currently untracked, so a direct trailing-whitespace and conflict-marker scan over `PLANS.md`, `PLANS.archive.md`, and `apps/framkey-desktop/ui/styles.css` also passed.

## Risks

- Visual QA is still a sampled check; it should cover normal desktop and narrow/mobile-ish widths before claiming broader polish.
- Smoke data uses an unfunded mock account, so failed Activity entries are expected and should not be treated as real transaction failures.

# Wallet Product UI/UX Redesign

Status: completed

## Goal

Make the Tauri wallet app feel like a modern wallet product instead of a debug console, while preserving the existing trusted UI, dApp WebView, recovery, and signer-helper trust boundaries.

## Scope

- Redesign the trusted desktop shell around first-order wallet jobs: account readiness, assets, send, DeFi connection, pending review, activity, and recovery backup health.
- Replace the top tab strip feel with a more product-like navigation surface on desktop and a compact grid on narrow screens.
- Rebalance the visual hierarchy so Wallet and DeFi workflows surface primary panels first, while Activity, Review, and Diagnostics remain available without dominating the first viewport.
- Improve card rhythm, spacing, typography, tone colors, button hierarchy, and responsive constraints without changing dApp permissions or signer/recovery behavior.

## Invariants

- Do not expose Alchemy token/RPC URL, raw provider params, calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, or recovery share bytes.
- Do not grant dApps any new Tauri commands, filesystem, Keychain, GBxCart, signer-helper, recovery, or trusted wallet-send access.
- Do not perform real transactions, real Keychain unlocks, Touch ID prompts, GBxCart writes, cloud uploads, or GUI file deletion during visual QA.
- Keep diagnostics accessible for development, but visually secondary to normal wallet and DeFi flows.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/styles.css`
- `apps/framkey-desktop/ui/main.js`
- `PLANS.md`

## Verification

- Rebuild or run the Tauri app in mock mode.
- Computer Use visual inspection of Wallet, DeFi, Recovery, and Diagnostics at desktop size.
- JS syntax checks for trusted UI and dApp script.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop`

Completed verification:

- `cargo tauri build --debug --bundles app --no-sign` passed and refreshed the debug macOS app bundle for visual QA.
- Mock runtime smoke passed through dApp connect/signing, recovery backup/drill state, watch-asset, native send, and token send paths; unfunded send attempts failed closed as expected.
- Computer Use visual QA checked Wallet, DeFi, and Recovery first viewports: product overview panels lead each workspace, debug/session details are lower priority, and DeFi Browser is full-width beneath the DeFi decision surface.
- `node --check apps/framkey-desktop/ui/main.js` passed.
- `node --check apps/framkey-desktop/ui/dapp.js` passed.
- `cargo fmt --all -- --check` passed.
- `cargo check -p framkey-desktop` passed.
- `cargo nextest run -p framkey-desktop` passed: 87 tests run, 87 passed.
- Direct whitespace/conflict-marker scan on touched UI files and `PLANS.md` passed.
- Runtime temp state files were absent after smoke shutdown; no lingering `framkey-desktop` process was found.

## Risks

- This is a larger visual change than prior polish, so screenshots are required to catch layout regressions.
- CSS-only hierarchy can improve product feel, but later iconography/illustration and dedicated onboarding may still be needed before a production wallet release.
# Archived From PLANS.md On 2026-06-02

# Create Success Backup Summary UX

Status: completed

## Goal

Make the post-create recovery backup summary understandable for non-technical users by showing next actions first and hiding full paths/details by default.

## Scope

- Replace the create-success file/path grid with five user-facing material cards and collapsed file details.
- Hide the destructive create confirmation/button after a successful write so backup actions are the only active controls in that success panel.
- Keep all generated file paths and reveal actions available for audit/debug.
- Keep recovery plan/detail panels unchanged outside the immediate create-success status.

## Invariants

- Do not change recovery file generation, threshold policy, or backup placement semantics.
- Do not expose KEK, DEK, RRK, wallet secret, private key, or recovery share bytes.
- Do not remove the detailed file list; only move it behind an explicit details panel.

## Likely Files

- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `cargo tauri build --debug --bundles app --no-sign`

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- `node --check apps/framkey-desktop/ui/main.js`: passed after the click-target fix.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed after the click-target fix.
- `cargo fmt --all -- --check`: passed after the click-target fix.
- `cargo tauri build --debug --bundles app --no-sign`: passed after the click-target fix and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- Restarted the rebuilt app, pid `17304`.

# Desktop Create Backup-Set Folder

Status: completed

## Goal

Prevent desktop wallet creation from failing when the default recovery folder already contains an older backup pack, while preserving create-new safety for every generated recovery file.

## Scope

- Treat the selected recovery folder as a parent folder in the desktop create flow.
- Write each new wallet's recovery pack into a deterministic backup-set subfolder based on generation and backup set id.
- Keep low-level recovery file writes using create-new semantics so existing backup material is never overwritten.
- Keep CLI recovery pack output behavior unchanged.

## Invariants

- Do not overwrite existing recovery files.
- Do not expose or print KEK, DEK, RRK, wallet secret, private key, or recovery share bytes.
- Keep explicit connected-device replacement confirmation before write.
- Do not add cloud-provider API integration.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `README.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check`
- `cargo test -p framkey-desktop recovery_backup_set_out_dir_uses_unique_child_directory`
- `cargo check -p framkey-desktop`
- `cargo tauri build --debug --bundles app --no-sign`

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo test -p framkey-desktop recovery_backup_set_out_dir_uses_unique_child_directory`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- Restarted the rebuilt app, pid `5545`.

# Entitlement-Free Local Keychain Binding

Status: completed

## Goal

Keep the local KEK protected by macOS Keychain + Touch ID without requiring an Apple Developer Program signing identity, Team ID, provisioning profile, or Keychain access group entitlement.

## Scope

- Store the KEK in a local non-synchronizing macOS login Keychain generic-password item without entitlement-gated `SecAccessControl`.
- Require LocalAuthentication before storing or loading the KEK, and store a hash of the evaluated Touch ID domain state in the KEK blob.
- Reject KEK blobs when the Touch ID enrollment domain state changes; recovery must rebind the Mac.
- Use a new default Keychain service namespace so old protected development items do not block the entitlement-free path.
- Keep the mirrored create/recover UI language from the previous slice.
- Remove the signed-build workaround script once the entitlement-free path works.

## Invariants

- Do not add cloud-provider API integration.
- Do not expose or print KEK, DEK, RRK, wallet secret, private key, or recovery share bytes.
- Keep explicit connected-device replacement confirmation before write.
- Do not fall back to storing the KEK in a plain file or prompting the user for a remembered password.

## Likely Files

- `Cargo.toml`
- `crates/framkey-keychain-macos/Cargo.toml`
- `crates/framkey-keychain-macos/src/lib.rs`
- `crates/framkey-signer-helper/src/main.rs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `crates/framkey-cli/src/main.rs`
- `crates/framkey-native-host/src/main.rs`
- `README.md`
- `docs/vault-format.md`
- `docs/threat-model.md`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `cargo fmt --all -- --check`
- `cargo test -p framkey-keychain-macos`
- CLI create smoke with a temporary Keychain account and file device.
- `cargo check -p framkey-desktop`
- `cargo check -p framkey-cli`
- `cargo tauri build --debug --bundles app --no-sign`

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `echo ${RUSTC_WRAPPER:-<unset>}`: `sccache`.
- `sccache --show-stats`: available, local cache configured.
- `cargo fmt --all -- --check`: passed.
- `cargo test -p framkey-keychain-macos`: passed, 6 tests.
- `cargo check -p framkey-cli`: passed.
- `cargo check -p framkey-signer-helper`: passed.
- `cargo check -p framkey-native-host`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo build -p framkey-signer-helper`: passed.
- CLI create smoke with temporary Keychain account `codex-local-auth-1780311630`: passed; output used `access_policy = local_biometry_current_set` and `keychain_service = io.framkey.local-kek`.
- CLI open smoke for the generated vault: passed; Touch ID-gated KEK load decrypted address `0x0d9851b7734946c4fbb90488ee09e4e896ac4c72`.
- CLI recovery rebind smoke with cloud plus local physical shares and temporary account `codex-local-auth-recover-1780311665`: passed; `wallet_secret_touched = false`.
- CLI open smoke for the recovered vault: passed; recovered address matched `0x0d9851b7734946c4fbb90488ee09e4e896ac4c72`.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.

# Mirrored Create And Recover Flows

Status: completed

## Goal

Make create and recover read as two mirrored setup flows in Safety: both end by binding the Mac key and writing the connected GBA, while create starts from a new wallet and recover starts from existing backup material.

## Scope

- Reframe the recovery panel heading so it is the existing-backup path, not the whole Safety workspace.
- Reframe create as a step list using the same visual language as recovery.
- Keep the existing Tauri commands, recovery policy, Keychain behavior, and device write gate unchanged.

## Invariants

- Do not add cloud-provider API integration.
- Do not expose or print KEK, DEK, RRK, wallet secret, private key, or recovery share bytes.
- Keep explicit connected-device replacement confirmation before write.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `cargo fmt --all -- --check`
- `cargo tauri build --debug --bundles app --no-sign`
- Restart app and visually inspect Safety.

# Keychain Create UX and Rebinding Fix

Status: completed

## Goal

Fix the real create flow so the user can see that FRAMKey is waiting for macOS authorization and then writing the connected GBA device, while avoiding reuse of stale local Keychain ACLs during create/recovery replacement flows.

## Scope

- Add an explicit in-progress status for `Create wallet and backups`.
- Add the same in-progress status for restore writes that replace the connected vault device.
- Disable the create inputs while the Tauri command is in flight and restore them on success/failure.
- Force create/recovery helper operations to create a fresh strict local KEK instead of silently reusing an older Keychain item.
- Keep normal signing/opening behavior unchanged: each operation still loads the current Keychain item and asks macOS for authorization.

## Invariants

- Do not print or log KEK, DEK, RRK, wallet secret, private key, or recovery share bytes.
- Do not change vault, recovery-share, or encrypted vault backup serialization.
- Do not change recovery threshold policy.
- Do not remove the explicit connected-device replacement checkbox.

## Likely Files

- `crates/framkey-keychain-macos/src/lib.rs`
- `crates/framkey-signer-helper/src/main.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `cargo fmt --all -- --check`
- `cargo check -p framkey-keychain-macos`
- `cargo test -p framkey-keychain-macos`
- `cargo check -p framkey-signer-helper`
- `cargo check -p framkey-desktop`
- `cargo tauri build --debug --bundles app --no-sign`

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `echo ${RUSTC_WRAPPER:-<unset>}`: `sccache`.
- `sccache --show-stats`: available, local cache configured.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-keychain-macos`: passed.
- `cargo test -p framkey-keychain-macos`: passed, 4 tests.
- `cargo check -p framkey-signer-helper`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo check -p framkey-cli`: passed.
- `cargo check -p framkey-native-host`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- Restarted the rebuilt app, pid `39823`.
- Computer Use visual QA confirmed the create panel is present, still gated by the connected-device replacement checkbox, and the gate was left unchecked after inspection.

# Keychain SecAccessControl Hardening

Status: completed

## Goal

Move the local KEK from application-enforced Touch ID checks to macOS-enforced Keychain access control so each unlock/sign operation is gated by the OS Keychain item policy.

## Scope

- Replace the current generic-password blob storage path in `framkey-keychain-macos` with `SecAccessControl`-protected generic password items.
- Use a device-local, passcode-required, current-biometry policy for newly created or rebound KEKs.
- Drop legacy KEK compatibility; old local Keychain items can be deleted/recreated through create or recovery.
- Preserve the existing public API used by signer helper, desktop app, native host, and CLI.

## Invariants

- Do not change vault, DEK wrapper, RRK, or recovery-share serialization.
- Do not print or log KEK, DEK, RRK, wallet secret, or share bytes.
- Restore must still bind the recovered vault to the current Mac Keychain item before writing the configured vault device.
- Signing must continue to load the KEK per request through the signer helper.

## Likely Files

- `crates/framkey-keychain-macos/src/lib.rs`
- `PLANS.md`

## Verification

- `cargo check -p framkey-keychain-macos`
- `cargo test -p framkey-keychain-macos`
- `cargo check -p framkey-cli`
- `cargo check -p framkey-native-host`
- `cargo check -p framkey-signer-helper`
- `cargo tauri build --debug --bundles app --no-sign`

Completed verification:

- `cargo check -p framkey-keychain-macos`: passed.
- `cargo test -p framkey-keychain-macos`: passed, 4 tests.
- `cargo check -p framkey-signer-helper`: passed.
- `cargo check -p framkey-cli`: passed.
- `cargo check -p framkey-native-host`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- Restarted rebuilt debug app and confirmed the Safety workspace loads in pid `15403`.

# Recovery File Selection Feedback

Status: completed

## Goal

Make recovery file selection self-explanatory after the user picks files: show what was selected, whether it looks sufficient, what is missing in plain language, and what the safety check concluded.

## Scope

- Show selected vault backup and recovery files as short user-readable chips.
- Add missing-material guidance without requiring the user to understand share groups or policy combinations.
- Let the read-only safety check result override the preliminary client-side hint.
- Keep raw paths available only in Advanced.
- Clarify that the first restore item is the encrypted wallet backup, not a recovery share.
- Make Cloud and Local recovery boxes explicit input slots; do not infer slot state from filenames or remembered backup metadata.

## Invariants

- Do not print recovery share bytes or secret material.
- Do not change backend recovery policy or threshold behavior.
- Restore still requires local vault backup, recovery files, passing safety check, and explicit device replacement confirmation.
- Do not add cloud-provider API integration.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `cargo fmt --all -- --check`
- Rebuild debug app and visually inspect Safety empty/selected states.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- Computer Use visual QA confirmed Safety shows Cloud and Physical source tiles inside the Recovery files step, with the raw paths still collapsed behind Advanced.

Current verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- Computer Use visual QA confirmed the restore flow now shows "Encrypted wallet backup" first, no generic recovery-file picker, and explicit Cloud / Local recovery slots.

# Recovery Wizard Cognitive-Load Reduction

Status: completed

## Goal

Make wallet recovery feel like one guided task with the fewest possible concepts on screen: choose the downloaded vault backup, choose recovery files, let FRAMKey check them, then explicitly restore.

## Scope

- Replace the restore form with a four-step recovery wizard.
- Hide recovery policy combinations and raw path text behind secondary/advanced UI.
- Collapse "start a new wallet" because it is a different job from restoring.
- Hide empty backup-plan/policy panels until there is a generated or restored recovery plan to show.
- Keep existing command IDs and backend restore/check behavior.

## Invariants

- Restore still requires a local encrypted vault backup file.
- Restore still requires recovery shares that pass the existing recovery policy.
- The destructive device write still requires an explicit checkbox.
- Do not expose wallet secret, DEK, KEK, RRK, private key, recovery share bytes, or cloud credentials.
- Do not add cloud-provider API integration.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/styles.css`
- `apps/framkey-desktop/ui/main.js`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- `cargo fmt --all -- --check`
- Rebuild debug app and visually inspect Safety.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- Computer Use visual QA confirmed Safety now opens as a four-step recovery wizard with no top status-card strip, no visible path textarea by default, collapsed new-wallet creation, and restore gated behind file selection plus safety check plus explicit device replacement confirmation.

# Non-Home Workspace Product Refinement

Status: completed

## Goal

Refactor Apps, Safety, Activity, and System into compact user-task workspaces instead of header-heavy card stacks, while leaving Home as the primary wallet landing page.

## Scope

- Remove the large workspace hero/header treatment outside Home.
- Rework Apps around app launch, connection state, and pending approvals.
- Rework Safety around three user jobs: create a recoverable vault, restore from downloaded backup, and verify/track backup placement.
- Rework Activity as a readable transaction timeline with receipt actions secondary.
- Rework System as lower-priority diagnostics with compact status modules.
- Keep existing command IDs, trusted approval gates, recovery controls, and mock/dev flows working.

## Invariants

- Do not expose Alchemy token/RPC URL, raw provider params, calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, private key, or recovery share bytes.
- Do not grant dApps any new access to trusted wallet, filesystem, Keychain, GBxCart, signer-helper, recovery, or secret material.
- Keep recovery restore requiring a local encrypted vault backup file plus a valid recovery set.
- Keep pending approvals visible in Apps, Activity, and System.
- Do not change Home in this slice except where shared responsive shell rules require it.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/styles.css`
- `apps/framkey-desktop/ui/main.js`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js`
- `node --check apps/framkey-desktop/ui/dapp.js`
- Runtime visual QA against the running Tauri dev app for Apps, Safety, Activity, and System.
- Static DOM check that key command IDs still exist.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and rebuilt `/absolute/path/to/FRAMKey/target/debug/bundle/macos/FRAMKey.app`.
- Static DOM check confirmed the local vault backup chooser IDs, hidden non-Home command strips, and folded raw output panel are present.
- Computer Use visual QA confirmed Apps, Safety, Activity, and System no longer start with a large workspace hero; Safety starts on restore/recovery work, Apps starts on app choices, and System keeps raw command output collapsed.

# Consumer Wallet Desktop Product Redesign

Status: completed

## Goal

Reframe the trusted Tauri UI as a polished consumer desktop wallet, not a developer control panel: the first screen should communicate balance, trust state, primary actions, and next best step with strong product taste and low operational noise.

## Scope

- Rename and reorganize navigation around consumer jobs: Home, Apps, Safety, Activity, and System, while preserving existing workspace identifiers where practical.
- Replace module-first panels with composed product surfaces: an account command center, curated DeFi launch surface, backup/safety scorecard, activity ledger, and system details tucked behind a lower-priority workspace.
- Rewrite visible copy so it reads like a wallet product rather than bridge/debug terminology.
- Restyle the entire trusted UI shell: restrained desktop app chrome, left rail, premium card hierarchy, action hierarchy, app cards, status tone system, and responsive behavior.
- Keep all existing trusted commands, review gates, dApp isolation, recovery controls, signer-helper boundaries, and autosmoke flows intact.

## Invariants

- Do not expose Alchemy token/RPC URL, raw provider params, calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, private key, or recovery share bytes.
- Do not grant untrusted dApp WebView access to trusted wallet-send, filesystem, Keychain, GBxCart, signer-helper, recovery, or secret material.
- Do not hide pending review or failure states; make them calm but still explicit.
- Do not perform real card writes, real Keychain unlocks, Touch ID prompts, cloud upload/delete operations, or funded transactions during QA.
- Keep diagnostics available for development, but make it visually and navigationally secondary.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/styles.css`
- `apps/framkey-desktop/ui/main.js`
- `PLANS.md`

## Verification

- Completed `node --check apps/framkey-desktop/ui/main.js`.
- Completed `node --check apps/framkey-desktop/ui/dapp.js`.
- Completed `cargo fmt --all -- --check`.
- Completed `cargo check -p framkey-desktop`.
- Completed `cargo nextest run -p framkey-desktop` with 87 passed tests.
- Completed `cargo tauri build --debug --bundles app --no-sign`.
- Completed conflict-marker and trailing-whitespace scan over project files touched by this slice.
- Completed Computer Use visual QA for Home, Apps, Safety, Activity, and System; the trusted desktop now starts directly with the left rail and content, with no product header below the macOS titlebar.
- Generated and installed a new FRAMKey app icon candidate for the UI and Tauri bundle assets.

## Risks

- This is a broader UI reshaping than prior slices; visual QA needs to inspect the real desktop bundle, not just static files.
- The product shell can improve perceived quality now, but a future production wallet still needs brand/icon assets, onboarding, and packaged app signing/notarization.

# Product Startup Wallet-First UX

Status: completed

## Goal

Make the Tauri wallet app start like a consumer wallet: open the trusted FRAMKey window first, and open the untrusted dApp WebView only when the user chooses an app or when an explicit development smoke/start target asks for it.

## Scope

- Stop opening the local test dApp WebView on normal startup.
- Preserve explicit startup automation with `FRAMKEY_DESKTOP_START_URL`, `FRAMKEY_DESKTOP_START_DAPP`, `FRAMKEY_DESKTOP_DAPP_URL`, `FRAMKEY_DESKTOP_AUTOSMOKE`, or remote provider smoke settings.
- Keep trusted UI buttons for Uniswap, Aave, Test App, custom URLs, and dApp navigation working.
- Make the initial dApp/session copy read as "no app open" instead of implying Local Test is already active.
- Update docs so manual product startup and smoke startup are distinct.

## Invariants

- Do not remove the untrusted dApp isolation boundary.
- Do not expose Alchemy token/RPC URL, raw provider params, calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, private key, or recovery share bytes.
- Do not weaken account grants, trusted approval, transaction policy, signer-helper, recovery, or mock-only autosmoke checks.
- Existing remote Uniswap/Aave smoke commands must still open the requested dApp and run the configured provider smoke.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- Focused Rust tests for startup dApp target selection and unopened session defaults.
- JS syntax checks for trusted UI and dApp script.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- `cargo nextest run -p framkey-desktop dapp`
- Rebuild debug Tauri app and visually confirm normal startup opens the trusted wallet window without the local dApp WebView in front.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop dapp`: passed, 9 focused dApp/startup tests.
- `cargo tauri build --debug --bundles app --no-sign`: passed.
- Computer Use visual QA against the rebuilt bundle: normal mock startup opened the trusted FRAMKey wallet window with Apps showing `No app open`, disabled back/forward/reload controls, and no local dApp WebView in front; clicking Test App then created and focused the untrusted dApp WebView.
- `cargo nextest run -p framkey-desktop`: passed, 91 tests.
- Temporary startup QA state files were scanned for sensitive fields before cleanup; no Alchemy endpoint/token, decision token, raw transaction, share hex, private key, signature, wallet secret, or recovery root key was found.

## Risks

- Existing smoke workflows previously relied on default local dApp startup; explicit smoke flags must continue to open the dApp to avoid losing coverage.
- Some diagnostics now start empty until a dApp is opened, so empty-state copy needs to be product-clear rather than looking broken.

# Home Onboarding Recovery Path

Status: completed

## Goal

Make first-run Home explain and guide the product-critical setup path: create the encrypted vault, generate the documented recovery files, place cloud and physical backups, verify recovery, then connect the vault and open DeFi apps.

## Scope

- Add a Home setup path card that uses existing trusted vault/recovery/account state.
- Make the next best action dynamic: create backup files, place backups, check recovery set, connect vault, or open Apps.
- Route setup actions to existing trusted Safety, Connect, and Apps controls without adding dApp APIs or new secret paths.
- Keep the card useful in mock mode for UI/runtime QA.
- Update docs and plan notes for the wallet-first onboarding flow.

## Invariants

- Do not expose Alchemy token/RPC URL, raw provider params, calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, private key, or recovery share bytes.
- Do not grant untrusted dApps any new filesystem, Keychain, GBxCart, signer-helper, recovery, or trusted wallet-send access.
- Do not auto-run real Keychain, Touch ID, GBxCart writes, cloud uploads, or recovery rewrap from Home; Home only routes the user to existing trusted controls.
- Keep cloud-only recovery explicitly insufficient.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- JS syntax checks for trusted UI and dApp script.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- Focused or full `cargo nextest run -p framkey-desktop`
- Rebuild debug Tauri app and visually confirm Home shows the setup path before account/recovery state exists and routes to Safety/Test flows without opening dApp by default.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed.
- Computer Use visual QA against the rebuilt bundle: Home shows the setup path before account/recovery state exists; Create Backup Files routes to Safety's Create Recoverable Vault area without starting a dApp, writing the device, or uploading files.
- `cargo nextest run -p framkey-desktop`: passed, 91 tests.
- Temporary onboarding QA state files were scanned for sensitive fields before cleanup; no Alchemy endpoint/token, decision token, raw transaction, share hex, private key, signature, wallet secret, or recovery root key was found.

## Risks

- This is mostly UX orchestration; real recovery safety remains proven by the existing signer-helper recovery drill and rewrap tests.
- If state restoration is stale or files are moved outside FRAMKey, the Home card must keep pointing back to Safety rather than implying recovery is complete.

# Recovery Placement Checklist UX

Status: completed

## Goal

Make the Safety workspace turn generated recovery files into a clear destination checklist: iCloud Drive, Google Drive, one local physical backup, and one remote physical backup, with policy readiness visible at the same time.

## Scope

- Add a placement checklist above the raw recovery file cards.
- Reuse the existing local placement state and recovery-set builder instead of creating a second source of truth.
- Show each destination's file path, short hash, placement state, recovery role, and safe user action.
- Keep manifest, guide, and per-file reveal cards available for audit/debug detail.
- Update docs so recovery placement is described as a guided trusted UI workflow.

## Invariants

- Do not upload files, delete files, copy secret bytes, or auto-run recovery from the checklist.
- Do not expose Alchemy token/RPC URL, raw provider params, calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, private key, or recovery share bytes.
- Keep cloud-only placement explicitly insufficient.
- Keep the untrusted dApp WebView isolated from recovery paths, filesystem pickers, Keychain, GBxCart, and signer-helper access.

## Likely Files

- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- JS syntax checks for trusted UI and dApp script.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- Full `cargo nextest run -p framkey-desktop`
- Rebuild debug Tauri app and visually confirm the Safety recovery placement checklist in mock recovery autosmoke.
- Sensitive-field scan of temporary runtime state after visual QA.

Completed verification:

- Archived older completed plan sections into `PLANS.archive.md`; `PLANS.md` is back under the project size threshold at 339 lines.
- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `echo ${RUSTC_WRAPPER:-<unset>}`: `sccache`.
- `sccache --show-stats`: available, local cache configured.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 91 tests.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.
- Computer Use visual QA against a temporary public-metadata recovery state: Safety rendered destination cards for iCloud Drive, Google Drive, local physical, and remote physical backups. Checking iCloud plus Google changed placement to `Needs physical`; checking one local physical share changed placement to `Recoverable`, enabled `Use Checked Recovery Set`, and kept cloud-only explicitly insufficient.
- Temporary QA state files were scanned before cleanup; no Alchemy endpoint/token, decision token, raw transaction, share hex, private key, signature, wallet secret, or recovery root key bytes were found. The only hits were `recoveryRootKeyPrinted=false` booleans.
- Temporary default `~/.framkey/recovery-state.json`, generated QA files, and running FRAMKey app processes were cleaned up.
- `git diff --check` and direct trailing-whitespace/conflict-marker scan over touched files passed.

## Risks

- Placement checkboxes are local UI bookkeeping; moving files outside FRAMKey can make the checklist stale until the user updates it.
- The UI must not imply that both cloud destinations alone are recoverable.

# Create Vault Device Write Gate

Status: completed

## Goal

Make the Safety `Create Recoverable Vault` flow honest and hard to misuse: creating a real vault writes the encrypted vault image to the configured GBxCart/file device, so the UI must require explicit operator confirmation before invoking the backend instead of presenting the checkbox as optional.

## Scope

- Rename the visible action/copy from backup-pack-only language to vault-plus-backups language.
- Disable the create button until the write confirmation checkbox is checked.
- Keep backend configured-device overwrite enforcement intact as the safety backstop.
- Update docs to describe the trusted flow as an explicit write-gated vault creation action.
- Verify normal startup and recovery UI still render without touching Keychain, GBxCart, or cloud storage.

## Invariants

- Do not generate or write a real vault unless the user explicitly confirms the configured-device write.
- Do not grant dApps filesystem, Keychain, GBxCart, signer-helper, recovery, or secret access.
- Do not expose Alchemy token/RPC URL, raw provider params, calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, private key, or recovery share bytes.
- Recovery smoke remains the mock/development path for generating disposable backup files without touching the configured vault device.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `docs/product-roadmap.md`
- `PLANS.md`

## Verification

- JS syntax checks for trusted UI and dApp script.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- Focused or full `cargo nextest run -p framkey-desktop`
- Rebuild debug Tauri app and visually confirm Safety disables the create action until explicit write confirmation is checked.
- Runtime QA must not write GBxCart, invoke Touch ID, upload files, or persist recovery secrets.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `echo ${RUSTC_WRAPPER:-<unset>}`: `sccache`.
- `sccache --show-stats`: available, local cache configured.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo nextest run -p framkey-desktop`: passed, 91 tests.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.
- Computer Use visual QA against mock mode: Safety shows `Create Vault + Backups`, the explicit configured-device write checkbox, explanatory copy, and a disabled `Confirm Device Write` button before confirmation.
- Static UI guard check confirmed the checkbox `change` handler calls `updateCreateVaultActionState`, which disables the button until checked and changes the enabled label to `Create Vault + Backups`; `createVault()` also fails closed if invoked without confirmation.
- Temporary QA state files were scanned before cleanup; no Alchemy endpoint/token, decision token, raw transaction, share hex, private key, signature, wallet secret, or recovery root key bytes were found.
- Temporary QA files and running FRAMKey app processes were cleaned up.
- Direct trailing-whitespace/conflict-marker scan over touched files passed.

## Risks

- This keeps the real creation path deliberately gated; users wanting backup-only dry runs should use mock/recovery smoke rather than the production create-vault action.

# Headerless Desktop Shell

Status: completed

## Goal

Remove the in-app product header from the trusted desktop wallet so the macOS titlebar remains the only top app identity chrome and the app body starts directly with navigation plus wallet content.

## Scope

- Remove visible brand/status lockup from the trusted main window shell.
- Preserve the existing status update DOM targets as hidden sinks so runtime JS behavior does not change.
- Add final CSS overrides that keep the shell headerless across desktop and narrow layouts.
- Update docs/plan notes to reflect that the trusted UI no longer has an app-level header.

## Invariants

- Do not change dApp WebView trust boundaries, signer-helper access, recovery behavior, RPC configuration, or wallet action policy.
- Do not expose Alchemy token/RPC URL, raw provider params, calldata, signatures, wallet secret, KEK, DEK, RRK, recovery root key, or recovery share bytes.
- Keep this as a layout fix only.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- JS syntax check for trusted UI and dApp script.
- `cargo fmt --all -- --check`
- `cargo check -p framkey-desktop`
- Rebuild debug Tauri app and visually confirm no in-app header appears below the macOS titlebar.

Completed verification:

- `node --check apps/framkey-desktop/ui/main.js`: passed.
- `node --check apps/framkey-desktop/ui/dapp.js`: passed.
- `echo ${RUSTC_WRAPPER:-<unset>}`: `sccache`.
- `sccache --show-stats`: available, local cache configured.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p framkey-desktop`: passed.
- `cargo tauri build --debug --bundles app --no-sign`: passed and produced `target/debug/bundle/macos/FRAMKey.app`.
- Computer Use visual QA against the rebuilt debug app: Home and Safety start directly under the native macOS titlebar with left navigation and content; the visible FRAMKey product lockup and `Trusted desktop` status header are gone.
- Runtime app processes launched for visual QA were stopped after inspection.

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
