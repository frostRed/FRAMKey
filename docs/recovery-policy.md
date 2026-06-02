# Recovery Policy

FRAMKey recovery separates durability from recovery authority.

Cloud storage can hold encrypted vault backups and the cloud recovery group, but iCloud plus Google Drive must not be enough to recover a wallet.

## Default Grouped Policy

```text
Total threshold: 2-of-3 groups

Group 1: Cloud
  iCloud share
  Google Drive share
  internal threshold: 2-of-2

Group 2: Local Physical
  local physical backup
  internal threshold: 1-of-1

Group 3: Remote Physical
  off-site physical backup
  internal threshold: 1-of-1
```

## Recovery Matrix

```text
iCloud + Google                         no
iCloud + Google + local physical        yes
iCloud + Google + remote physical       yes
local physical + remote physical        yes
iCloud + local physical                 no
Google + local physical                 no
single physical backup                  no
main GBA card alone                     no
main GBA card + current Mac local auth   daily use, not recovery
main GBA card + new Mac                  no, unless recovery pack is present
```

## Key Structure

```text
Wallet Secret
  encrypted by DEK

DEK
  wrapped by Mac Keychain KEK for daily use
  wrapped by RRK for recovery

RRK
  split into grouped recovery shares
```

The user-facing product should call these recovery groups and recovery cards, not Shamir shares or RRK.

## Backup Pack Files

`vault build-keychain-encrypted-image --recovery-out-dir <dir>` creates:

- `backup-01.dat`
- `backup-02.dat`
- `backup-03.dat`
- `backup-04.dat`

Each file is a recovery bundle with encrypted vault data plus one recovery share. The `.dat` names are intentionally plain so the files look like ordinary backup artifacts in Finder and cloud folders.

Recommended placement:

- Put `backup-01.dat` in iCloud Drive.
- Put `backup-02.dat` in Google Drive.
- Put `backup-03.dat` on local physical storage such as a TF card or USB drive.
- Put `backup-04.dat` away from the main Mac, GBA card, and cloud accounts.

The current implementation creates the recovery wrapper and backup pack at wallet-generation time inside `framkey-signer-helper`. The CLI and trusted desktop UI write files and report paths plus BLAKE3 hashes only; they do not print the RRK, wallet secret, DEK, or share bytes.

Recovery rewrap is implemented for Keychain vaults. Given one backup bundle as the encrypted vault source and enough backup bundles for recovery authorization, `framkey-signer-helper` reconstructs the RRK, decrypts only the recovery DEK wrapper, adds a new macOS Keychain DEK wrapper, and returns a rewritten encrypted save image. The desktop UI does not connect to cloud-provider storage APIs during recovery; the user downloads the needed backup files locally and selects them. Recovery rewrap does not decrypt the wallet secret.

The current code verifies the share math and recovery rewrap in tests:

- iCloud + Google alone: no
- iCloud + Google + local physical: yes
- iCloud + Google + remote physical: yes
- local physical + remote physical: yes
