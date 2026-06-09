# BTC Wallet Strategy

FRAMKey's BTC support is staged around account visibility first, then balance/RPC, then PSBT/UTXO signing. The current implementation has moved past receive-only accounts: after the vault is connected, trusted UI can query BTC balances through a configured Esplora-compatible backend and can submit controlled P2WPKH sends after PSBT/UTXO review. BTC must not reuse EVM provider, chain id, SIWE, Permit, ERC-20, or `eth_sendTransaction` paths.

## Network Choice

The default user-facing BTC test network is Testnet4.

- Testnet4 is the default because BIP94 is deployed and Bitcoin Core supports `-testnet4`.
- Testnet3 is not added because Bitcoin Core has signaled that Testnet3 support is intended to be phased out.
- Signet is reserved for controlled integration testing. It is useful when predictable block production matters, but it is not the default user wallet testnet account.
- Mainnet and Testnet4 are the default BTC account surfaces. Signet and regtest need explicit environment or developer-mode enablement before they should appear as wallet accounts.

## Current Account Surface

A secp256k1 single-key vault exposes:

- EVM active-chain EOA: dApp provider, SIWE, Permit, trusted native send, trusted ERC-20 send, review, signing, and broadcast.
- BTC mainnet P2WPKH: receive address, trusted UI balance refresh, and trusted UI PSBT send when a mainnet backend is configured.
- BTC Testnet4 P2WPKH: receive address, trusted UI balance refresh, and trusted UI PSBT send when a Testnet4 backend is configured.

BTC balance, send, and PSBT signing are trusted-UI-only. They are not exposed to the injected EIP-1193 provider or untrusted dApp WebView.

## Balance And RPC Strategy

BTC balance is an address/UTXO indexing problem, not only a JSON-RPC reachability problem. A Bitcoin Core node without a wallet import/indexing strategy cannot answer arbitrary address balance queries safely.

Implemented first backend:

- Esplora-compatible HTTP per network.
- Default public endpoints are provided for mainnet and Testnet4, with config/env overrides and explicit disable values.
- Balance refresh queries `/address/{address}/utxo`, parses confirmed and mempool UTXOs, and reports confirmed, unconfirmed, spendable, and UTXO-count fields.
- Broadcast posts the locally validated raw transaction to `/tx` and requires the returned txid to match the locally signed transaction id.
- BTC balance and send require an already connected trusted wallet session; they must not implicitly unlock Keychain, read the GBA card, or load the vault.

Other acceptable future backend options:

- Bitcoin Core watch-only wallet or descriptor import, bound to one network.
- Electrum server, bound to one network.

Backend invariants:

- Bind the backend to an explicit BTC network and reject cross-network reuse.
- Show unavailable backend states without blocking EVM wallet use.
- Avoid leaking mainnet addresses to testnet or third-party backends.
- Treat default public Esplora endpoints as a privacy trade-off; self-hosted or disabled endpoints are supported configuration choices.

## PSBT And UTXO Strategy

BTC signing must use a PSBT/UTXO review path. It must not sign raw transactions from an untrusted caller.

Current controlled send scope:

- Native SegWit P2WPKH only.
- Single-key vault-derived BTC mainnet or Testnet4 account only.
- Trusted UI command only; no dApp method, no EIP-1193 provider method, and no raw PSBT import from untrusted callers.
- Coin selection uses confirmed owned UTXOs from the selected network backend, capped input count, deterministic ordering, dust/change policy, bounded fee rate, and RBF sequence.
- Review summary shows network, source, recipient, amount, input value, fee, fee rate, change, estimated vbytes, selected inputs, outputs, simulation status, and policy blockers.
- Signer-helper and mock signing revalidate expected address, network, input ownership, P2WPKH witness data, SIGHASH_ALL, RBF sequence, fee, and final transaction shape before broadcast.

Future work remains for Taproot, descriptors, multisig, coin control, batching, hardware-style policies, fee estimation, backend freshness proofs, and Signet/regtest developer networks.
