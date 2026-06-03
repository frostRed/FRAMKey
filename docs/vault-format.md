# Vault Format

The wallet vault payload is a JSON `VaultFile` stored inside a fixed-size GBA save image. The save image format is intentionally non-compatible with the original two-slot test layout.

## Principles

- Store encrypted wallet material only.
- Store DEK wrappers, never plaintext DEK.
- Keep device storage independent from wallet semantics.
- Use generations for write ordering and rollback detection.
- Use redundant superblocks plus Reed-Solomon shards so bounded media corruption can be detected and reconstructed before wallet payload parsing.
- Treat Reed-Solomon reconstruction as storage repair only; payload BLAKE3 and `VaultFile` validation still decide whether a vault is trusted.

## Vault Fields

```text
VaultFile
  magic = "FRAMKEY\0"
  format_version = 1
  wallet_id
  generation
  created_at
  updated_at
  wallet_type = evm_eoa_secp256k1
  public_address
  encrypted_wallet_secret
  dek_wrappers
  recovery_policy
```

## Save Image Direction

```text
Save Image
  Superblock copy 0, 1024 bytes
  Superblock copy 1, 1024 bytes
  Superblock copy 2, 1024 bytes
  Superblock copy 3, 1024 bytes
  Interleaved Reed-Solomon shard region
  Optional trailing padding
```

The intended write sequence is:

1. Build the next-generation `VaultFile` payload.
2. Split payload bytes across 16 data shards.
3. Generate 8 Reed-Solomon parity shards.
4. Hash every full shard and hash the plaintext payload.
5. Write four hashed superblock copies.
6. Interleave shard bytes into the save image.
7. Write the full image and verify by reading it back.

GBxCart writes are full-image writes. FRAMKey verifies same-session readback and then opens a fresh GBxCart session for a second readback before reporting write success.

## Save Image Wire Format

The current image is 64 KiB by default:

```text
0x0000..0x03ff  superblock copy 0
0x0400..0x07ff  superblock copy 1
0x0800..0x0bff  superblock copy 2
0x0c00..0x0fff  superblock copy 3
0x1000..        interleaved shard bytes
```

For a 64 KiB image, `(65536 - 4096) / 24 = 2560`, so each shard is 2560 bytes and the payload capacity is `16 * 2560 = 40960` bytes.

Superblock fields:

```text
0x00  8 bytes   "FRKSAVE\0"
0x08  u16 le    format version = 2
0x0a  u16 le    superblock length = 1024
0x0c  u32 le    image size
0x10  u32 le    header length = 4096
0x14  u64 le    generation
0x1c  u32 le    payload length
0x20  u8        data shard count = 16
0x21  u8        parity shard count = 8
0x22  u8        total shard count = 24
0x23  u8        superblock copy index
0x24  u32 le    shard size
0x28  u32 le    payload capacity
0x30  32 bytes  BLAKE3 hash of payload bytes
0x50  768 bytes BLAKE3 hashes of all 24 full shards
0x3c0 32 bytes  BLAKE3 hash of superblock bytes 0x000..0x3bf
```

Shard bytes are stored interleaved, not contiguously:

```text
for byte_index in 0..shard_size:
  for shard_index in 0..24:
    image[4096 + byte_index * 24 + shard_index] = shard[shard_index][byte_index]
```

On read, FRAMKey scans all four superblocks and uses any valid copy. It deinterleaves shards, checks each shard hash, treats hash failures as erasures, reconstructs with Reed-Solomon when at least 16 shards remain, and finally checks the payload hash before parsing `VaultFile`.

The CLI can build and inspect a non-secret test image:

```bash
cargo run -p framkey-cli -- vault build-test-image --out framkey-test-vault.sav --generation 1
cargo run -p framkey-cli -- vault inspect-image --path framkey-test-vault.sav
```

Pass `--image-size 32768` only when building an explicit 32 KiB save-size fixture.

This format is the current save container for test, dev, and Keychain encrypted vault payloads.

## macOS Keychain Encrypted Vault

The Keychain encrypted vault uses the same Reed-Solomon save image container. The reconstructed payload is JSON containing a `VaultFile` with:

- `encrypted_wallet_secret`: a generated 32-byte EVM secp256k1 test private key encrypted with a random DEK.
- `dek_wrappers`: one `mac_keychain` wrapper that encrypts the DEK with a 32-byte KEK stored in macOS Keychain.
- `wallet_id`, `generation`, `wallet_type`, `device_id`, and `keychain_item_id` metadata authenticated through AEAD AAD.

CLI workflow:

```bash
cargo build -p framkey-signer-helper
cargo run -p framkey-cli -- vault init-keychain-kek
cargo run -p framkey-cli -- vault build-keychain-encrypted-image --out framkey-keychain-vault.sav --generation 1 --recovery-out-dir recovery-pack
cargo run -p framkey-cli -- vault open-keychain-encrypted-image --path framkey-keychain-vault.sav
```

The KEK is stored in a local, non-synchronizing macOS login Keychain generic-password item. The personal local build path does not require Apple Developer Program signing entitlements for Keychain access groups. Each KEK store/load first evaluates LocalAuthentication with macOS device-owner authentication, letting the system use Touch ID or the account password in one prompt flow. Creating a replacement vault or recovering a vault resets the local KEK instead of reusing an existing Keychain item. Opening or signing an existing vault reads the KEK once and does not write the Keychain item. `vault rebind-keychain-kek` can explicitly rebind a current-format local KEK to the current local-auth policy without decrypting the wallet secret or modifying the save image. Connect and signing paths trigger macOS authorization when needed. The desktop Diagnostics panel can also ask the real signer-helper process to probe only Keychain KEK access through `Repair Signing Access`; that probe does not read the card, pass a vault image, or decrypt a wallet secret. `vault trust-keychain-helper-access` remains a local ad-hoc build helper: it parses the current signer-helper `CDHash` and asks `/usr/bin/security` to set a `cdhash:<helper CDHash>` partition-list on the configured Keychain item. It does not accept a password argument, does not read vault data, and does not use an allow-all-applications ACL. Build, open, signing, and recovery rewrap commands delegate sensitive handling to `framkey-signer-helper`; the CLI receives encrypted save image bytes or public metadata only. It does not print the plaintext KEK, DEK, RRK, wallet secret, or recovery share bytes.

When recovery backups are requested, the active vault also contains:

- `dek_wrappers`: one `recovery` wrapper that encrypts the DEK with a generated 32-byte recovery root key.
- `recovery_policy`: the generated policy id and `standard 2-of-3 grouped recovery` label.

The helper returns a recovery backup pack to the CLI. The CLI writes four recovery bundle files, `backup-01.dat` through `backup-04.dat`, into `--recovery-out-dir` using create-new semantics so existing backup files are not silently overwritten. Each bundle embeds encrypted vault data plus one recovery share. The trusted desktop create flow treats the selected recovery folder as a parent and writes each new pack into a fresh `framkey-backup-g<generation>-<backup-set>` child folder before using the same create-new file writes. Recovery rewrap uses one bundle as the encrypted vault source and enough bundle shares to reconstruct the RRK, decrypts only the recovery DEK wrapper, and adds a current `mac_keychain` wrapper to a rewritten encrypted save image. It does not decrypt the wallet secret.

The signer helper can also perform the current EOA smoke signature:

```bash
cargo run -p framkey-cli -- signer personal-sign --device file --path framkey-keychain-vault.sav --message "FRAMKey signer helper smoke"
```

The CLI recovers the signer address from the returned `personal_sign` signature and fails the command if recovery does not match the helper-reported address.

## Dev/Test Encrypted Vault

The dev/test encrypted vault uses the same Reed-Solomon save image container. The reconstructed payload is JSON containing a `VaultFile` with:

- `encrypted_wallet_secret`: a generated 32-byte EVM secp256k1 test private key encrypted with a random DEK.
- `dek_wrappers`: one `dev_test` wrapper that encrypts the DEK with a caller-provided 32-byte dev KEK.
- `wallet_id`, `generation`, and `wallet_type` metadata authenticated through AEAD AAD.

CLI workflow:

```bash
cargo run -p framkey-cli -- vault generate-dev-kek
export FRAMKEY_DEV_KEK_HEX=<dev_kek_hex>
cargo run -p framkey-cli -- vault build-dev-encrypted-image --out framkey-dev-vault.sav --generation 1
cargo run -p framkey-cli -- vault open-dev-encrypted-image --path framkey-dev-vault.sav
```

The dev open command decrypts the DEK, decrypts the test wallet secret, and prints metadata plus a BLAKE3 hash of the decrypted secret. It does not print the plaintext secret. This dev KEK wrapper remains only an explicit local plumbing path; use the Keychain wrapper for local-machine protection.
