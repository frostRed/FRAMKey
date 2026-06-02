# Browser Bridge Read-Only MVP

This slice wires the development Chrome extension to `framkey-native-host` through Chrome Native Messaging. It is intentionally read-only.

This is no longer the next signing/product surface. The Chrome bridge remains useful for proving native messaging and extension provider boundaries, but FRAMKey's next product milestone is the Tauri DeFi Browser described in `docs/product-roadmap.md`.

## Supported Methods

The injected provider supports:

- `eth_chainId`
- `eth_accounts`
- `eth_requestAccounts`
- `framkey_getStatus`
- `wallet_getCapabilities`

The bridge explicitly rejects signing and transaction methods:

- `eth_sendTransaction`
- `eth_sign`
- `eth_signTransaction`
- `eth_signTypedData*`
- `personal_sign`

## Trust Boundary

```text
dApp page
  -> injected FRAMKey EIP-1193 provider
  -> content script
  -> extension service worker
  -> Chrome Native Messaging
  -> framkey-native-host
  -> configured save-image device
  -> framkey-signer-helper
  -> macOS Keychain
```

The extension stores only per-origin account authorization. It does not touch wallet secrets, KEKs, DEKs, or decrypted wallet material. The native host reads the save image and invokes the signer helper to open the Keychain vault and derive the public EVM address.

## Development Install

Build the native binaries:

```bash
cargo build -p framkey-native-host -p framkey-signer-helper
```

Load the extension:

1. Open `chrome://extensions`.
2. Enable Developer mode.
3. Click Load unpacked.
4. Select `extension/chrome`.
5. Copy the extension ID.

Register the native host manifest at:

```text
~/Library/Application Support/Google/Chrome/NativeMessagingHosts/dev.framkey.native_host.json
```

Example manifest:

```json
{
  "name": "dev.framkey.native_host",
  "description": "FRAMKey development native host",
  "path": "/absolute/path/to/FRAMKey/target/debug/framkey-native-host",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://<extension-id>/"
  ]
}
```

## Native Host Configuration

The native host reads optional JSON config from:

```text
~/.framkey/native-host.json
```

Default development behavior assumes:

- GBxCart port `/dev/cu.usbserial-210`
- save type `gba-sram-fram-512kbit`
- chain id `0x1`
- Keychain service `io.framkey.local-kek`
- Keychain account `default`
- signer helper next to the native host binary

Example config:

```json
{
  "chain_id": "0x1",
  "device": {
    "kind": "gbx_cart",
    "port": "/dev/cu.usbserial-210",
    "save_type": "gba-sram-fram-512kbit"
  },
  "keychain": {
    "service": "io.framkey.local-kek",
    "account": "default"
  },
  "signer_helper": {
    "path": "/absolute/path/to/FRAMKey/target/debug/framkey-signer-helper",
    "blake3": "<optional-helper-blake3>",
    "allow_unsandboxed": false
  }
}
```

For fixture testing without GBxCart:

```json
{
  "chain_id": "0x1",
  "device": {
    "kind": "file",
    "path": "/absolute/path/to/FRAMKey/save_image_samples/20260531-signer-helper-live-smoke/keychain-signer-helper-readback.sav"
  }
}
```

Environment overrides are also supported for local runs:

- `FRAMKEY_NATIVE_HOST_CONFIG`
- `FRAMKEY_NATIVE_HOST_CHAIN_ID`
- `FRAMKEY_SAVE_IMAGE_PATH`
- `FRAMKEY_GBXCART_PORT`
- `FRAMKEY_GBA_SAVE_TYPE`
- `FRAMKEY_EXPECTED_SAVE_SIZE`
- `FRAMKEY_KEYCHAIN_SERVICE`
- `FRAMKEY_KEYCHAIN_ACCOUNT`
- `FRAMKEY_SIGNER_HELPER`
- `FRAMKEY_SIGNER_HELPER_BLAKE3`
- `FRAMKEY_NATIVE_HOST_ALLOW_UNSANDBOXED_HELPER`

## Direct Smoke

The native host speaks Chrome Native Messaging, so requests are length-prefixed JSON. A simple direct smoke can use a small script to write the frame.

Example methods to smoke:

- `eth_chainId`: does not touch the card.
- `framkey_getStatus`: does not touch the card.
- `personal_sign`: must be rejected.
- `framkey_getAccount`: reads the configured save image and triggers Touch ID through the helper.

`eth_requestAccounts` in the extension calls native-host `framkey_getAccount`, stores the origin grant in `chrome.storage.local`, then returns `[address]` to the dApp.
