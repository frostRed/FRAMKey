# FRAMKey

FRAMKey is an experimental cartridge-backed wallet vault. It stores an
encrypted single-key wallet in a GBA save/FRAM image, unlocks daily use through
macOS Keychain plus local device-owner authentication, and signs through a
short-lived helper process.

The current product path is Tauri-first: a trusted FRAMKey desktop UI owns
wallet state, approval, recovery, and diagnostics, while an untrusted embedded
dApp WebView receives only a constrained injected provider.

## Status

FRAMKey is a prototype, not a production wallet.

- Do not use it for real funds.
- It is not a hardware wallet security model. A GBA cartridge is removable
  storage, not an isolated signing element or trusted display.
- The EOA wallet secret can enter Mac process memory during signing. The design
  goal is to keep that window narrow and inside a short-lived helper, not to
  claim cold-wallet guarantees.
- Packaged local builds can be ad-hoc unsigned for testing. Public
  distribution still needs code signing, notarization, hardened runtime,
  dependency audit, crash-dump review, and broader transaction-policy coverage.

See [docs/threat-model.md](docs/threat-model.md) for the detailed boundary.

## What Works Today

- Fixed-size GBA save-image vault container with redundant superblocks,
  Reed-Solomon repair, payload hashes, and generation metadata.
- macOS Keychain KEK wrapper gated by LocalAuthentication.
- Short-lived `framkey-signer-helper` for vault generation, public account
  derivation, recovery rewrap, SIWE signing, Permit signing, EVM transaction
  signing, and BTC PSBT signing.
- Tauri desktop wallet with trusted Home, DeFi, Safety, Activity, and System
  workspaces.
- Untrusted dApp WebView with EIP-1193/EIP-6963 provider injection, origin
  scoped account grants, controlled SIWE-only `personal_sign`, controlled
  Permit/Permit2 `eth_signTypedData_v4`, and policy-gated `eth_sendTransaction`.
- Conservative transaction review with local decoding, optional Alchemy
  simulation, known-counterparty labels, Permit policy, Uniswap/Aave intent
  handling, and fail-closed blockers for unknown or unsupported requests.
- EVM account plus BTC mainnet/Testnet4 account surfaces from the same
  secp256k1 vault secret. BTC balance and P2WPKH sends are trusted-UI-only and
  are not exposed to the dApp provider.
- Recovery backup packs made of four plain `.dat` files. Recovery requires
  cloud plus physical material, or two physical groups; cloud files alone are
  intentionally insufficient.
- GBxCart save-image read/write paths for the verified GBA save types, with
  readback verification.
- CH347T SPI NOR physical-backup write/read paths through `flashrom`, with a
  privileged macOS helper for the desktop Safety workspace.
- Read-only Chrome native-messaging bridge for extension-boundary testing.

## Repository Layout

```text
apps/framkey-desktop/       Tauri wallet app and trusted UI
extension/chrome/           read-only development browser extension
crates/framkey-core         shared IDs, errors, wallet types
crates/framkey-crypto       secret containers and encrypted box metadata
crates/framkey-device       cartridge/save-image device abstraction
crates/framkey-gbxcart      native GBxCart serial boundary
crates/framkey-ch347        CH347/flashrom device boundary
crates/framkey-vault        vault and save-image format
crates/framkey-recovery     grouped recovery policy model
crates/framkey-ipc          helper/native-host message framing
crates/framkey-evm          EVM address, signing, typed-data logic
crates/framkey-btc          BTC account, UTXO, PSBT, transaction logic
crates/framkey-simulation   transaction decoding, simulation, policy context
crates/framkey-keychain-macos
                            macOS Keychain KEK wrapper
crates/framkey-signer-helper
                            short-lived signing/recovery helper
crates/framkey-native-host  Chrome native-messaging host
crates/framkey-cli          development CLI
crates/framkey-testkit      test support
docs/                       product, security, format, and workflow notes
```

## Requirements

- Rust 1.88 or newer.
- macOS for the Keychain, LocalAuthentication, and Tauri desktop paths.
- `cargo-nextest` for the preferred test runner.
- GBxCart RW hardware for native GBA save-image tests.
- `flashrom` 1.4 or newer with `ch347_spi` support for CH347T workflows.
- An Alchemy token is optional for live EVM read RPC and simulation. The repo
  `.env` is ignored by Git.

## Quick Start

Run the normal Rust checks:

```bash
cargo fmt --all
cargo check --workspace
cargo nextest run --workspace
```

Start the Tauri desktop wallet in development mode:

```bash
cargo build -p framkey-signer-helper -p framkey-ch347-helper
cargo run -p framkey-desktop
```

Run the desktop app with an in-memory mock wallet for UI and dApp flow testing:

```bash
FRAMKEY_WALLET_MODE=mock_in_memory \
FRAMKEY_SIMULATION_PROVIDER=local_decoder_only \
cargo run -p framkey-desktop
```

Build a local debug app bundle:

```bash
cargo build -p framkey-signer-helper -p framkey-ch347-helper
cd apps/framkey-desktop/src-tauri
cargo tauri build --debug --bundles app --no-sign
```

`--no-sign` is for local testing only. It does not produce a notarized public
release.

## Common CLI Workflows

Build and inspect a non-secret test vault image:

```bash
cargo run -p framkey-cli -- vault build-test-image --out framkey-test-vault.sav --generation 1
cargo run -p framkey-cli -- vault inspect-image --path framkey-test-vault.sav
```

Create a Keychain-encrypted vault and recovery pack:

```bash
cargo build -p framkey-signer-helper
cargo run -p framkey-cli -- vault init-keychain-kek
cargo run -p framkey-cli -- vault build-keychain-encrypted-image \
  --out framkey-keychain-vault.sav \
  --generation 1 \
  --recovery-out-dir recovery-pack
cargo run -p framkey-cli -- vault open-keychain-encrypted-image \
  --path framkey-keychain-vault.sav
```

Smoke-test signer-helper `personal_sign` with a file-backed vault:

```bash
cargo run -p framkey-cli -- signer personal-sign \
  --device file \
  --path framkey-keychain-vault.sav \
  --message "FRAMKey signer helper smoke"
```

Read/write a GBxCart GBA save image:

```bash
cargo run -p framkey-cli -- device read-save \
  --device gbx-cart \
  --port /dev/cu.usbserial-210 \
  --save-type gba-sram-fram-512kbit \
  --out read.sav

cargo run -p framkey-cli -- device write-save \
  --device gbx-cart \
  --port /dev/cu.usbserial-210 \
  --save-type gba-sram-fram-512kbit \
  --input read.sav
```

Read/write a CH347T SPI NOR image through `flashrom`:

```bash
cargo run -p framkey-cli -- device read-save \
  --device ch347 \
  --expected-save-size 8388608 \
  --out ch347-read.bin

cargo run -p framkey-cli -- device write-save \
  --device ch347 \
  --expected-save-size 8388608 \
  --input ch347-read.bin
```

Use `--chip <flashrom-chip-name>`, `--flashrom <path>`, or `--spispeed <speed>`
only when the default `flashrom` probe needs help.

## Desktop Configuration

The desktop app reads optional configuration from:

```text
~/.framkey/desktop.json
```

Useful development environment variables:

- `FRAMKEY_WALLET_MODE=mock_in_memory`
- `FRAMKEY_SIMULATION_PROVIDER=local_decoder_only` or `alchemy_asset_changes`
- `ALCHEMY_TOKEN` or `FRAMKEY_ALCHEMY_TOKEN`
- `FRAMKEY_RPC_URL` or `FRAMKEY_ALCHEMY_RPC_URL`
- `FRAMKEY_GBXCART_PORT`
- `FRAMKEY_SIGNER_HELPER` / `FRAMKEY_SIGNER_HELPER_BLAKE3`
- `FRAMKEY_CH347_HELPER` / `FRAMKEY_CH347_HELPER_BLAKE3`
- `FRAMKEY_DESKTOP_START_URL=local`, `uniswap`, `aave`, or an `http`/`https`
  URL for development dApp WebView startup

The trusted UI reports helper readiness, RPC health, recovery state, and
sanitized transaction activity without returning wallet material, recovery
share bytes, Alchemy tokens, or full RPC URLs.

See [docs/tauri-defi-browser.md](docs/tauri-defi-browser.md) for the full app
configuration and smoke workflows.

## Recovery Model

FRAMKey separates durability from recovery authority. Backup pack files contain
encrypted vault data plus one recovery share each:

```text
backup-01.dat  iCloud Drive
backup-02.dat  Google Drive
backup-03.dat  local physical storage
backup-04.dat  off-site physical storage
```

Recovery requires one of:

- iCloud + Google Drive + one physical backup
- local physical backup + off-site physical backup

Cloud-only recovery must fail. The recovery rewrap path binds a recovered vault
to the current Mac Keychain item without decrypting the wallet secret.

See [docs/recovery-policy.md](docs/recovery-policy.md).

## Security Invariants

- Browser extension and dApp WebView remain secret-free.
- Native messaging host remains a relay/orchestrator, not a signer.
- Device code reads and writes opaque save images; it does not understand
  wallets.
- Desktop trusted UI owns confirmation and policy context, but it should not
  keep long-lived plaintext wallet material.
- Signer helper is the only Keychain-vault path that may touch decrypted wallet
  material, and it is designed as a short-lived one-request process.
- Cloud storage is encrypted durability material, not sufficient recovery
  authority.
- Unknown, malformed, unsupported, or evidence-missing signing requests fail
  closed before signer-helper access.

## Documentation

- [docs/product-roadmap.md](docs/product-roadmap.md) - product direction and
  version plan
- [docs/threat-model.md](docs/threat-model.md) - trust boundaries and missing
  production hardening
- [docs/tauri-defi-browser.md](docs/tauri-defi-browser.md) - Tauri app,
  provider, policy, RPC, and smoke workflows
- [docs/vault-format.md](docs/vault-format.md) - save-image and vault wire
  format
- [docs/recovery-policy.md](docs/recovery-policy.md) - backup group model and
  recovery matrix
- [docs/btc-wallet-strategy.md](docs/btc-wallet-strategy.md) - BTC account,
  UTXO, and PSBT constraints
- [docs/gbxcart-notes.md](docs/gbxcart-notes.md) - GBxCart hardware notes and
  verified save types
- [docs/browser-bridge.md](docs/browser-bridge.md) - read-only Chrome bridge
  development path
