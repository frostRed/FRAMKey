# Threat Model

## Product Position

FRAMKey v0 is a removable encrypted vault plus software signer prototype. It is meant to be stronger than a browser-only wallet for small testnet or low-value use, but weaker than a real hardware wallet with an isolated signing chip and trusted display.

## Trust Boundaries

Near-term Tauri DeFi Browser path:

```text
trusted FRAMKey app UI
  -> untrusted dApp WebView
  -> injected provider
  -> local approval broker
  -> configured save-image device
  -> GBA save/FRAM storage
  -> encrypted save image bytes
  -> short-lived signer helper
  -> macOS Keychain KEK
```

The current Tauri foundation exposes account, chain, status, allowlisted read RPC, controlled SIWE-only `personal_sign`, controlled Permit/Permit2 `eth_signTypedData_v4`, and policy-gated `eth_sendTransaction`. Signing requests are captured into a trusted UI approval broker, summarized, bounded, and signed only after an unexpired trusted-window approval plus the relevant policy authorization. The signer helper validates the requested account against the vault-derived EVM address before EVM signing. SIWE `personal_sign` requests must match the requesting origin, signer account, active chain, and a short expiration window before signer-helper access. Permit/Permit2 requests must also pass backend semantic checks for exact recognized EIP-712 type schema, signer/owner binding, active-chain domain binding, known Permit2/verifying-contract semantics, known spender, bounded future deadlines, and non-max allowance amounts. Non-SIWE personal messages, unknown typed-data, and raw `eth_sign` methods are still captured and rejected before signer-helper access. Transaction review can use offline local decoding, Alchemy asset-change simulation, supported Universal Router subcommand decoding, and sanitized Aave account health-factor evidence. Desktop-controlled EVM broadcasting submits policy-authorized signed transactions through the configured Alchemy RPC after rejecting unsupported transaction envelopes and unsafe fee/nonce preparation inputs. BTC balance and send are trusted UI-only: the desktop app queries configured Esplora-compatible backends for selected-account UTXOs, builds P2WPKH PSBTs locally, captures a BTC transaction review, and asks signer-helper to sign only after network, address, UTXO ownership, fee, change, dust, RBF, sighash, and final-transaction checks pass. BTC accounts, balances, UTXOs, PSBTs, raw transactions, and backend URLs are not exposed to the untrusted dApp WebView.

Long-term Chrome/Brave extension path:

```text
dApp page
  -> browser extension provider
  -> Chrome native messaging host
  -> GBxCart / save-image device
  -> GBA save/FRAM storage
  -> encrypted save image bytes
  -> short-lived signer helper
  -> macOS Keychain KEK
```

Only the signer helper may touch decrypted EOA wallet material in the Keychain-protected MVP path. The extension, native host, CLI, and device layer must not. Explicit dev/test vault commands remain local plumbing only and are not a production protection path.

## Expected Protections

- GBA card alone leaks only encrypted vault data.
- Cloud folders leak only encrypted vault backups and at most one recovery group.
- dApp JavaScript cannot directly access wallet secrets.
- Browser extension does not sign locally.
- Remote dApp content inside the Tauri DeFi Browser is untrusted and must not receive direct Tauri command, filesystem, Keychain, GBxCart, or signer-helper access.
- Browser bridge methods are read-only until an approval broker, simulation layer, and transaction parser exist.
- Tauri approval broker decisions are real for SIWE-only `personal_sign` and policy-validated Permit/Permit2 `eth_signTypedData_v4`; `eth_sendTransaction` additionally requires an `allowed` transaction policy. The current default transaction policy emits ordinary-signable `allowed` or fail-closed `blocked` outcomes: unknown active approval spenders/operators, unsupported Universal Router commands, multicall incomplete semantics, risky Uniswap swap parameters, third-party Aave recipients/accounts, and missing or malformed Aave risk evidence do not reach signer-helper or mock signing. Non-SIWE personal messages, unknown typed-data, and raw `eth_sign` approvals remain dry-run/blocked before signing. BTC transaction approvals are real only for trusted UI-origin P2WPKH sends whose PSBT summary has `canSign=true`; they do not create any dApp-facing BTC signing method. Alchemy simulation responses, Aave account data, and BTC Esplora UTXOs are kept as audit/input data, not as a full post-transaction position simulator, and broader transaction policy is still required before larger real-funds use.
- macOS Keychain KEK is only a DEK wrapper, not the wallet secret. New local KEKs are stored in a local, non-synchronizing macOS login Keychain item. Each KEK store/load is gated by macOS device-owner LocalAuthentication, so the system can use Touch ID or the account password in one prompt flow. Opening and signing existing vaults read the KEK once and do not write the Keychain item. Connect and signing paths trigger macOS authorization when needed; the trusted Diagnostics panel can also launch the real signer helper for a Keychain-only `Repair Signing Access` probe. That probe can trigger the system authorization prompt but does not read the card, pass a vault image, or decrypt the wallet secret. Local ad-hoc builds can bind the login Keychain ACL partition-list to the current signer-helper `CDHash` with `vault trust-keychain-helper-access`; FRAMKey does not use an allow-all-applications ACL or accept the login Keychain password as a command argument. Replacement create and recovery flows reset the local KEK instead of reusing an existing local Keychain item.
- Configured Keychain vaults use private local app state to remember the highest validated vault generation per wallet. Normal open/sign paths reject older configured vault images before signer-helper access, and the high-water state advances only after helper validation/signing succeeds or after create/recover has successfully written the configured device.
- Keychain vault build, open, recovery rewrap, SIWE `personal_sign`, and transaction signing operations delegate sensitive handling to a one-request signer helper process.
- Helper requests run as the helper process identity so LocalAuthentication prompts and Keychain access bind to the real helper binary; the CLI can pin the helper binary BLAKE3 hash.
- Recovery does not require displaying or storing plaintext seed phrases.

## Not Protected

- Fully compromised Mac.
- Replaced desktop app or signer helper binary.
- User approving a malicious transaction after a misleading UI.
- Malicious or compromised dependency supply chain.
- Physical copying of the GBA save area.
- Rollback before this Mac has recorded local high-water generation state for that wallet, such as on a fresh install or after intentionally erasing private app state.

## Required Hardening Before Real Funds

- Code signing, notarization, hardened runtime, and sandboxing.
- Production code signing, notarization, hardened runtime, and network-denied signer-helper sandboxing. This is separate from the local Keychain item format; the personal development path intentionally avoids Keychain access group entitlements.
- Crash dump and secret logging audit.
- Dependency locking and auditing.
- Transaction parser coverage for approvals, Permit, Permit2, typed data, and unknown calldata.
- Broader transaction policy coverage, asset-change normalization, and fail-closed allow rules before real-value transaction signing.
- Production recovery UX polish, backup-health checks, and recovery drills for Mac replacement or local Keychain loss.
