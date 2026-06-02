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

The current Tauri foundation exposes account, chain, status, allowlisted read RPC, controlled `personal_sign`, controlled Permit/Permit2 `eth_signTypedData_v4`, and policy-gated `eth_sendTransaction`. Signing requests are captured into a trusted UI approval broker, summarized, bounded, and signed only after an unexpired trusted-window approval plus the relevant policy authorization. The signer helper validates the requested account against the vault-derived EVM address before signing. Unknown typed-data and raw `eth_sign` methods are still captured and rejected before signer-helper access. Transaction review can use offline local decoding or Alchemy asset-change simulation, and desktop-controlled broadcasting submits policy-authorized signed transactions through the configured Alchemy RPC.

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
- Tauri approval broker decisions are real for `personal_sign` and recognized Permit/Permit2 `eth_signTypedData_v4`; `eth_sendTransaction` additionally requires an `allowed` policy or an explicit high-risk override for overrideable warnings. The current transaction policy treats unknown active approval spenders/operators as high-risk even when simulation succeeds. Unknown typed-data and raw `eth_sign` approvals remain dry-run. Alchemy simulation responses are kept as audit input, and richer transaction policy is still required before real funds.
- macOS Keychain KEK is only a DEK wrapper, not the wallet secret. New local KEKs are stored in a local, non-synchronizing macOS login Keychain item. Each KEK store/load is gated by LocalAuthentication Touch ID, and the KEK blob includes a hash of the evaluated Touch ID domain state. Replacement create and recovery flows reset the local KEK instead of reusing older local Keychain items. Touch ID enrollment drift requires recovery rewrap.
- Keychain vault build, open, recovery rewrap, `personal_sign`, and transaction signing operations delegate sensitive handling to a one-request signer helper process.
- Helper requests run as the helper process identity so LocalAuthentication prompts and Keychain access bind to the real helper binary; the CLI can pin the helper binary BLAKE3 hash.
- Recovery does not require displaying or storing plaintext seed phrases.

## Not Protected

- Fully compromised Mac.
- Replaced desktop app or signer helper binary.
- User approving a malicious transaction after a misleading UI.
- Malicious or compromised dependency supply chain.
- Physical copying of the GBA save area.
- Rollback to an older encrypted vault if generation checks are not enforced.

## Required Hardening Before Real Funds

- Code signing, notarization, hardened runtime, and sandboxing.
- Production code signing, notarization, hardened runtime, and network-denied signer-helper sandboxing. This is separate from the local Keychain item format; the personal development path intentionally avoids Keychain access group entitlements.
- Crash dump and secret logging audit.
- Dependency locking and auditing.
- Transaction parser coverage for approvals, Permit, Permit2, typed data, and unknown calldata.
- Broader transaction policy coverage, asset-change normalization, and fail-closed allow rules before real-value transaction signing.
- Explicit rollback detection with local generation memory.
- Production recovery UX polish, backup-health checks, and recovery drills for Touch ID enrollment drift.
