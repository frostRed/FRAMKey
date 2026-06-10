# Release Security Hardening

Status: completed

## Goal

Fix the release-blocking security issues found in the pre-public audit: local vault rollback detection, CH347 privileged helper response handling, Tauri CSP inline-script exposure, and the missing Rust-side trusted-window check on `framkey_status`.

## Scope

- Add desktop-local high-water generation memory for Keychain vault images and enforce it before normal unlock/signing helper access.
- Update the high-water state only after a vault image has been successfully opened/signed by the signer helper or after create/recover writes the configured device successfully.
- Remove the CH347 privileged helper's root write to a user-owned response path by returning the helper response over stdout.
- Tighten the Tauri CSP to avoid inline script execution.
- Require trusted main-window validation in the `framkey_status` command handler.

## Invariants

- Do not change the vault save-image format or recovery bundle format.
- Do not update high-water generation state from an unauthenticated higher-generation image before helper validation succeeds.
- Do not expose wallet secrets, recovery shares, Keychain KEKs, private keys, backup bytes, or RPC credentials in new state or errors.
- Do not expose CH347 or status commands to untrusted dApp windows.
- Keep changes localized to existing desktop/helper/vault boundaries.

## Likely Files

- `crates/framkey-vault/src/keychain_vault.rs`
- `crates/framkey-vault/src/lib.rs`
- `apps/framkey-desktop/src-tauri/src/paths.rs`
- `apps/framkey-desktop/src-tauri/src/signer_runtime.rs`
- `apps/framkey-desktop/src-tauri/src/recovery_ops.rs`
- `apps/framkey-desktop/src-tauri/src/ch347_helper.rs`
- `apps/framkey-desktop/src-tauri/src/commands.rs`
- `apps/framkey-desktop/src-tauri/tauri.conf.json`
- `crates/framkey-ch347-helper/src/main.rs`
- `crates/framkey-ch347-helper/src/lib.rs`
- focused tests near the affected boundaries

## Verification

- Passed: `cargo fmt --all -- --check`
- Passed: `cargo check -p framkey-vault -p framkey-ch347-helper -p framkey-desktop`
- Passed: `cargo nextest run -p framkey-vault -p framkey-ch347-helper -p framkey-desktop --no-fail-fast` (186 tests passed; one existing leaky-test marker reported by nextest)
- Passed: `node --check apps/framkey-desktop/ui/main.js`
- Passed: `git diff --check`
- Passed: `cargo clippy -p framkey-vault -p framkey-ch347-helper -p framkey-desktop --all-targets --no-deps -- -A clippy::large_enum_variant -A clippy::needless_range_loop -D warnings`

## Main Risks

- A rollback check must not create an irreversible denial-of-service from forged higher-generation unauthenticated images.
- CH347 helper stdout must remain bounded and parseable through the existing administrator launcher path.
- CSP tightening must not break the current external-script-only UI load path.

# Safety Workspace UI Reorganization

Status: completed

## Goal

Reorganize the Safety workspace so ROM backup, wallet restore, wallet creation, and placement status are easy to scan without showing every long workflow at once.

## Scope

- Keep existing CH347 write/read, recovery restore, and create-vault commands unchanged.
- Add a Safety task selector that shows one primary workflow at a time.
- Keep backup-pack placement/status visible as supporting context instead of mixing it into the primary action stack.
- Preserve trusted-window boundaries and all existing confirmation requirements.
- Update CSS/JS only as needed for the new Safety information architecture.

## Invariants

- Do not weaken overwrite confirmations for CH347 ROM writes, GBA writes, or restore.
- Do not expose backup bytes, recovery shares, wallet secrets, Keychain material, private keys, signatures, or RPC credentials.
- Do not move CH347 commands into dApp-visible surfaces.
- Do not change recovery policy semantics: cloud pair plus one physical, or local plus off-site physical.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js` (passed)
- `cargo fmt --all -- --check` (passed)
- `cargo check -p framkey-desktop` (passed)
- Browser visual smoke of the Safety workspace at desktop width (passed; one active Safety workflow visible)
- Browser visual smoke of the Safety workspace at 390px width (passed; no horizontal overflow)
- `cargo tauri build --debug --bundles app --no-sign` (passed)
- `git diff --check` (passed)

## Main Risks

- Hiding inactive Safety panels must not prevent status state from updating in memory.
- The task selector must remain reachable on mobile and must not create nested-card visual clutter.

# CH347 ROM Backup Reader

Status: completed

## Goal

Add a trusted desktop Safety action that reads the connected CH347 SPI ROM, verifies the ROM dump hash, and saves either the embedded FRAMKey physical backup payload or the full-chip image to a user-selected local directory.

## Scope

- Extend the existing `framkey-ch347-helper` privileged sidecar protocol with a read operation.
- Parse the FRAMKey physical-backup ROM header written by the CH347 writer and extract the original `backup-xx.dat` payload when present.
- Preserve full-chip image read support when the ROM has no FRAMKey physical-backup header.
- Add trusted Tauri commands and Safety UI controls for choosing an output directory and running the read operation.
- Return metadata only: paths, sizes, BLAKE3 hashes, chip selection, speed, and storage format.

## Invariants

- Do not expose CH347 operations to dApps or untrusted windows.
- Do not return backup bytes, recovery share bytes, wallet secrets, Keychain material, private keys, signatures, or RPC credentials to the UI.
- Do not write outside the selected output directory.
- Use create-new output semantics so existing backup files are not overwritten.
- Keep the helper launch boundary restricted to a fixed `--request` path argument and stdout response IPC.

## Likely Files

- `crates/framkey-ch347-helper/src/lib.rs`
- `crates/framkey-ch347-helper/src/main.rs`
- `crates/framkey-ch347-helper/src/tests.rs`
- `apps/framkey-desktop/src-tauri/src/ch347_helper.rs`
- `apps/framkey-desktop/src-tauri/src/recovery_ops.rs`
- `apps/framkey-desktop/src-tauri/src/commands.rs`
- `apps/framkey-desktop/src-tauri/src/config.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`

## Verification

- `cargo fmt --all -- --check` (passed)
- `cargo check -p framkey-ch347-helper -p framkey-desktop` (passed)
- `cargo nextest run -p framkey-ch347-helper physical_backup_rom_image_extracts_original_payload fake_flashrom_read_extracts_backup_payload_to_output_dir fake_flashrom_write_round_trip_returns_privileged_metadata` (passed)
- `cargo nextest run -p framkey-desktop ch347_backup_read_uses_fake_flashrom_and_saves_extracted_payload ch347_backup_write_uses_fake_flashrom_and_returns_verification_metadata` (passed)
- `node --check apps/framkey-desktop/ui/main.js` (passed)
- `TAURI_ENV_DEBUG=1 scripts/prepare-tauri-sidecars.sh` (passed)
- `cargo tauri build --debug --bundles app --no-sign` (passed)
- `git diff --check` (passed)

## Main Risks

- 512MiB ROM reads still move the whole chip through flashrom before extraction, so exact verification can be slow.
- Existing ROMs written before the FRAMKey container change may only be recoverable as full-chip images.

# CH347 Recovery Bundle ROM Image Writer

Status: completed

## Goal

Allow the desktop CH347 physical-backup writer to accept ordinary FRAMKey `backup-xx.dat` recovery bundle files by wrapping them into a full-chip SPI NOR image before invoking flashrom.

## Scope

- Probe the connected CH347/flashrom target to learn the SPI ROM capacity.
- Preserve support for full-chip images when the selected file already matches the chip size.
- For smaller backup files, build a bounded FRAMKey physical-backup image with a small header, the selected backup payload, and `0xFF` padding to the detected chip size.
- Keep write and readback verification exact over the full image while reporting selected payload size/hash separately from full ROM image size/hash.
- Update UI copy/docs/tests so users can choose `backup-xx.dat` rather than needing to prebuild a 16 MiB image.

## Invariants

- Do not write partial flashrom images; flashrom still receives exactly one full-chip image.
- Do not parse wallet secrets, recovery share bytes, encrypted vault contents, KEK, DEK, RRK, private keys, signatures, or RPC credentials.
- Do not expose CH347 operations to dApps or untrusted windows.
- Do not guess a chip size when flashrom probe cannot report one for a smaller backup payload.
- Keep the existing privileged helper boundary and shell-argument restrictions.

## Likely Files

- `crates/framkey-ch347/src/flashrom.rs`
- `crates/framkey-ch347/src/device.rs`
- `crates/framkey-ch347/src/tests.rs`
- `crates/framkey-ch347-helper/src/lib.rs`
- `crates/framkey-ch347-helper/src/tests.rs`
- `apps/framkey-desktop/src-tauri/src/recovery_ops.rs`
- `apps/framkey-desktop/src-tauri/src/tests.rs`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check` (passed)
- `cargo check -p framkey-ch347 -p framkey-ch347-helper -p framkey-desktop` (passed)
- `cargo nextest run -p framkey-ch347 flashrom_probe_output_extracts_chip_size` (passed)
- `cargo nextest run -p framkey-ch347-helper request_accepts_512_mib_input_limit smaller_backup_payload_is_wrapped_as_full_rom_image fake_flashrom_write_round_trip_returns_privileged_metadata` (passed)
- `cargo nextest run -p framkey-desktop ch347_backup_write_uses_fake_flashrom_and_returns_verification_metadata` (passed)
- `node --check apps/framkey-desktop/ui/main.js` (passed)
- `TAURI_ENV_DEBUG=1 scripts/prepare-tauri-sidecars.sh` (passed)
- `cargo tauri build --debug --bundles app --no-sign` (passed)
- `git diff --check` (passed)

## Main Risks

- This creates a FRAMKey physical-backup container on the ROM; future restore-from-ROM still needs a matching extractor/read path if the user wants to recover directly from a ROM dump.
- flashrom probe output format may vary; parser tests cover known output but the UI still needs the improved error chain for unusual chips.

# CH347 Helper Error Visibility

Status: completed

## Goal

Expose the actual CH347 helper or flashrom failure reason in the desktop UI instead of showing only the outer `CH347 privileged helper write/readback verification failed` context.

## Scope

- Preserve the existing privileged-helper/write/readback behavior.
- Include full sanitized `anyhow` error chains when converting helper and desktop errors into user-visible provider errors.
- Add focused tests so future context wrappers do not hide the root CH347/flashrom cause again.
- Rebuild the local debug app after the diagnostic fix.

## Invariants

- Do not include backup bytes, recovery share bytes, wallet secrets, Keychain material, private keys, signatures, or RPC credentials in errors.
- Do not change CH347 write semantics, helper launch privileges, flashrom argv construction, or approval policy.
- Keep errors bounded so flashrom output cannot flood the UI or logs.

## Likely Files

- `apps/framkey-desktop/src-tauri/src/config.rs`
- `apps/framkey-desktop/src-tauri/src/tests.rs`
- `crates/framkey-ch347-helper/src/lib.rs`
- `crates/framkey-ch347-helper/src/tests.rs`
- `PLANS.md`

## Verification

- `cargo fmt --all -- --check` (passed)
- `cargo check -p framkey-ch347-helper -p framkey-desktop` (passed)
- `cargo nextest run -p framkey-ch347-helper error_response_keeps_root_cause_context` (passed)
- `cargo nextest run -p framkey-desktop ch347_provider_error_keeps_helper_root_cause` (passed)
- `node --check apps/framkey-desktop/ui/main.js` (passed)
- `TAURI_ENV_DEBUG=1 scripts/prepare-tauri-sidecars.sh` (passed)
- `cargo tauri build --debug --bundles app --no-sign` (passed)
- `git diff --check` (passed)

## Main Risks

- More complete errors can expose local filesystem paths and flashrom device output; that is acceptable for the trusted desktop UI but must remain bounded and secret-free.
- The current reported error may still require one more real hardware attempt after this fix to see the true root cause.

# Safety CH347 Entry And Approval Panel Scope

Status: completed

## Goal

Make the CH347 physical backup writer discoverable from the Safety first screen and stop rendering the same `Pending approvals` card across unrelated workspaces.

## Scope

- Promote the CH347 writer into the Safety command surface so the user can find the file-pick/write/verify flow without scrolling through recovery restore details.
- Keep CH347 writing in the trusted Safety workspace and preserve the existing helper/write/readback verification boundary.
- Limit the full pending-approval review panel to the DeFi approval workflow; keep other workspaces focused on their own jobs.
- Update docs so the workspace model no longer says approvals are visible in every workspace.

## Invariants

- Do not change approval semantics, policy checks, signer-helper routing, or dApp exposure.
- Do not expose CH347 commands outside the trusted main window or add them to dApp-facing surfaces.
- Do not remove the first-screen DeFi pending approval path; only remove duplicated full panels from unrelated workspaces.
- Do not add layout-heavy redesign beyond the entry and scope fixes.

## Likely Files

- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `node --check apps/framkey-desktop/ui/main.js` (passed)
- `git diff --check` (passed)
- Static browser check at `http://127.0.0.1:4177/` (passed; Safety shows CH347 entry first, Safety hides `Pending approvals`, DeFi keeps the full approval panel, Home pending state changes the app action to `Review approval`)

## Main Risks

- Hiding the full review panel outside DeFi could make an already-pending request less visible if the user navigates away; the left-nav badge or DeFi callout should remain the navigation affordance.
- Moving the CH347 panel upward must not obscure recovery restore, which remains a separate Safety workflow.

# CH347 Controlled Privileged Helper

Status: completed

## Goal

Add a narrow privileged helper path so the trusted desktop Safety action can perform CH347/flashrom write plus fresh readback verification on macOS without running the whole Tauri app as root.

## Scope

- Add a `framkey-ch347-helper` sidecar binary that accepts only a local JSON request file and writes a local JSON response file.
- Keep the helper limited to one operation: read the selected backup image, verify its size and BLAKE3, invoke the CH347/flashrom backend, and return metadata for exact readback verification.
- Add desktop runtime logic that creates private temp request/response files, verifies the helper hash when configured, and on macOS launches only this helper through administrator authorization.
- Preserve the existing non-privileged CH347 backend for CLI and non-macOS paths.
- Add config/env overrides for the CH347 helper path and optional BLAKE3 pin, plus sidecar packaging support.
- Update UI copy/docs to make the macOS admin prompt and helper boundary explicit.

## Invariants

- Do not run the whole desktop app as root.
- Do not pass backup bytes, chip names, flashrom paths, or speed values through shell-expanded command text; the shell command may contain only quoted helper/request/response file paths.
- The privileged helper must not expose Keychain, signer-helper, wallet signing, recovery authorization, dApp provider, network, or filesystem browsing behavior.
- A successful result must still mean one flashrom write and one fresh readback exact match.
- No raw backup bytes, recovery share bytes, wallet secret bytes, KEK, DEK, RRK, private keys, signatures, or RPC credentials may be returned or logged.

## Likely Files

- `Cargo.toml`
- `crates/framkey-ch347-helper/Cargo.toml`
- `crates/framkey-ch347-helper/src/*`
- `scripts/prepare-tauri-sidecars.sh`
- `apps/framkey-desktop/src-tauri/tauri.conf.json`
- `apps/framkey-desktop/src-tauri/Cargo.toml`
- `apps/framkey-desktop/src-tauri/src/config.rs`
- `apps/framkey-desktop/src-tauri/src/constants.rs`
- `apps/framkey-desktop/src-tauri/src/paths.rs`
- `apps/framkey-desktop/src-tauri/src/recovery_ops.rs`
- `apps/framkey-desktop/src-tauri/src/signer_runtime.rs`
- `apps/framkey-desktop/src-tauri/src/tests.rs`
- `apps/framkey-desktop/ui/main.js`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `echo $RUSTC_WRAPPER` (passed; `sccache`)
- `sccache --show-stats` (passed)
- `TAURI_ENV_DEBUG=1 scripts/prepare-tauri-sidecars.sh` (passed; prepared signer and CH347 helpers)
- `cargo fmt --all -- --check` (passed)
- `cargo check -p framkey-ch347-helper -p framkey-ch347 -p framkey-desktop` (passed)
- `cargo nextest run -p framkey-ch347-helper -p framkey-ch347 -p framkey-desktop` (passed; 164 tests)
- `cargo check -p framkey-cli` (passed)
- `node --check apps/framkey-desktop/ui/main.js` (passed)
- `git diff --check` (passed)

## Main Risks

- AppleScript administrator authorization is a practical local-dev privileged launcher, not a hardened SMJobBless/LaunchDaemon installation; production packaging will still need code signing and a stronger installer-managed helper.
- macOS may still reject unsigned helper behavior depending on local policy; the desktop must surface helper launch failures clearly.
- flashrom/chip behavior remains hardware-dependent, so exact readback verification stays the final software guarantee.

# CH347 Physical Backup Writer UI

Status: completed

## Goal

Add a trusted desktop Safety action that lets the user pick one physical backup image, write it once through CH347/flashrom, then perform one fresh readback verification of the exact bytes.

## Scope

- Add trusted Tauri commands for picking a physical backup file and running CH347 write/readback verification.
- Keep the selected backup file as opaque bytes; do not parse backup layout, wallet metadata, recovery shares, or secrets.
- Use flashrom CH347 auto-detect by default so the flow is not tied to `W25Q128.V`; allow an optional chip override only for flashrom disambiguation.
- Use the selected file length as the expected image size and reject empty files before hardware access.
- Add a compact Safety UI control surface for file selection, flashrom path, optional chip, SPI speed, confirmation, and verification status.

## Invariants

- The write command is trusted-main-window only and must not be exposed to dApp/browser windows.
- No raw backup bytes, wallet secrets, KEK, DEK, recovery root key, recovery shares, plaintext private keys, or raw signatures may be logged or returned.
- Invoke flashrom through argv only; do not compose shell strings from file paths, chip names, or speed values.
- A successful result must mean the write completed and a fresh readback matched exactly.
- This physical backup writer is durability media, not a signing device or recovery authorization path.

## Likely Files

- `crates/framkey-ch347/src/device.rs`
- `crates/framkey-ch347/src/lib.rs`
- `crates/framkey-ch347/src/tests.rs`
- `apps/framkey-desktop/src-tauri/Cargo.toml`
- `apps/framkey-desktop/src-tauri/src/config.rs`
- `apps/framkey-desktop/src-tauri/src/recovery_ops.rs`
- `apps/framkey-desktop/src-tauri/src/commands.rs`
- `apps/framkey-desktop/src-tauri/src/main.rs`
- `apps/framkey-desktop/src-tauri/capabilities/default.json`
- `apps/framkey-desktop/src-tauri/permissions/autogenerated/*`
- `apps/framkey-desktop/ui/index.html`
- `apps/framkey-desktop/ui/main.js`
- `apps/framkey-desktop/ui/styles.css`
- `README.md`
- `docs/tauri-defi-browser.md`
- `PLANS.md`

## Verification

- `echo $RUSTC_WRAPPER` (passed; `sccache`)
- `sccache --show-stats` (passed)
- `cargo fmt --all` (passed)
- `cargo check -p framkey-ch347 -p framkey-desktop` (passed)
- `node --check apps/framkey-desktop/ui/main.js` (passed)
- `cargo nextest run -p framkey-ch347 -p framkey-desktop` (passed; 157 tests)
- `cargo check -p framkey-cli` (passed)
- `cargo fmt --all -- --check` (passed)
- `git diff --check` (passed)

## Main Risks

- On macOS, CH347 access through flashrom may still require root privileges or a future privileged helper; this slice should surface flashrom failures clearly instead of hiding them.
- flashrom chip auto-detect is flashrom-specific, so ambiguous chips may still require a chip override string from flashrom's own chip list.
- Hardware wiring, write protection, voltage, clip contact, and in-circuit interference can still fail; the app must rely on exact fresh readback verification for the software guarantee.

# CH347T SPI ROM Device Backend

Status: completed

## Goal

Add a CH347T-backed FRAMKey device path that can probe, read, write, and fresh-readback-verify SPI NOR ROM storage through the CLI before any GUI integration.

## Scope

- Add `crates/framkey-ch347` as the CH347T/SPI-ROM hardware boundary.
- Reuse `flashrom`'s `ch347_spi` programmer for SPI NOR chip support instead of hand-rolling ROM erase/write algorithms in FRAMKey.
- Add CLI support for `--device ch347`, with optional `--chip`, flashrom path, SPI speed, and expected-size checks.
- Keep the device layer limited to opaque image bytes; it must not parse wallets, recovery bundles, shares, or secrets.
- Ensure writes validate input size when configured and always perform an explicit fresh readback comparison after the external write step.
- Update README examples for the CH347T workflow.

## Invariants

- Do not expose wallet secrets, KEK, DEK, recovery root key, recovery shares, plaintext private key material, or raw signatures in output or temp filenames.
- Do not add CH347T access to dApp-facing or browser-extension surfaces.
- Do not let untrusted chip names or speed values become shell strings; invoke external tools with argv only.
- Do not silently write a differently sized image than the selected chip/expected storage size.
- Treat CH347T ROM storage as durability media, not a signing element or hardware-wallet security boundary.

## Likely Files

- `Cargo.toml`
- `crates/framkey-ch347/Cargo.toml`
- `crates/framkey-ch347/src/*`
- `crates/framkey-device/src/info.rs`
- `crates/framkey-cli/Cargo.toml`
- `crates/framkey-cli/src/args.rs`
- `crates/framkey-cli/src/device.rs`
- `README.md`
- `PLANS.md`
- `PLANS.archive.md`

## Verification

- `echo $RUSTC_WRAPPER` (passed; `sccache`)
- `sccache --show-stats` (passed)
- `cargo fmt --all -- --check` (passed)
- `cargo check -p framkey-device -p framkey-ch347 -p framkey-cli` (passed)
- `cargo nextest run -p framkey-device -p framkey-ch347 -p framkey-cli` (passed; 18 tests)
- `cargo run -p framkey-cli -- device probe --help` (passed; `ch347`, optional `--chip`, `--spispeed`, and `--flashrom` are visible)
- `command -v flashrom` / `flashrom --version` (passed; flashrom v1.7.0 is installed at `/opt/homebrew/sbin/flashrom`)
- `flashrom -L` lookup for common SPI NOR names (passed; exact Winbond 64Mbit names include `W25Q64JV-.Q` and `W25Q64JV-.M`)
- `sudo /opt/homebrew/sbin/flashrom -p ch347_spi:spispeed=15M` (passed; found Winbond `W25Q128.V`, 16384 kB SPI)
- `sudo target/debug/framkey-cli device read-save --device ch347 --flashrom /opt/homebrew/sbin/flashrom --chip "W25Q128.V" --expected-save-size 16777216 --out ch347-read-1.bin` (passed; BLAKE3 `3453514c15204eae0aacef61fefe26cdb073f5dc5e0e139ec302bb971b23424d`)
- `sudo target/debug/framkey-cli device read-save --device ch347 --flashrom /opt/homebrew/sbin/flashrom --chip "W25Q128.V" --expected-save-size 16777216 --out ch347-read-2.bin` (passed; same BLAKE3)
- `cmp ch347-read-1.bin ch347-read-2.bin` after fixing sudo-created file access (passed; no output)
- `sudo target/debug/framkey-cli device write-save --device ch347 --flashrom /opt/homebrew/sbin/flashrom --chip "W25Q128.V" --expected-save-size 16777216 --input ch347-read-1.bin` (passed; write plus fresh readback verify)
- `sudo target/debug/framkey-cli device read-save --device ch347 --flashrom /opt/homebrew/sbin/flashrom --chip "W25Q128.V" --expected-save-size 16777216 --out ch347-after-write.bin` (passed; same BLAKE3 after write)
- `git diff --check` (passed)

## Main Risks

- `flashrom` availability and version are machine-dependent; the CLI should fail with a clear setup error if `flashrom` is missing or too old for CH347.
- SPI NOR chip names are flashrom-specific, so docs must avoid pretending FRAMKey has its own broad chip database.
- Hardware wiring, clip quality, voltage, write protection, and in-circuit interference can cause false reads/writes; FRAMKey should still require explicit readback verification and expected-size checks.

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
