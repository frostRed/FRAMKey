# GBxCart Notes

Phase 0 should prove byte-stable save-image behavior before any vault or wallet logic is trusted.

## Baseline Workflow

1. Use FlashGBX or another known-good tool to read the cartridge save data.
2. Store the first sample under `save_image_samples/` with hardware notes.
3. Record file size and BLAKE3 hash using `framkey device read-save`.
4. Write the image back only after a separate backup exists.
5. Read again and verify the BLAKE3 hash matches.

## What To Record

- GBxCart hardware version.
- Cartridge type and visible label.
- Detected save type if known.
- Save image size.
- Tool and version used for any non-FRAMKey read/write.
- BLAKE3 hash before and after write.
- Any insertion, power, or timing anomalies.

## Current Implementation Boundary

`framkey-gbxcart` implements a narrow native serial transport for GBxCart RW with Lesserkuma firmware:

- CH340/CH341 USB serial detection, with `--port` override.
- Firmware probe for the `L` custom firmware family.
- GBA mode setup and header read.
- Explicit `gba-eeprom-64k` save read/write.
- Explicit `gba-sram-fram-256k` save read/write.
- Explicit `gba-sram-fram-512kbit` save read/write.
- Explicit `gba-sram-fram-1mbit` save read/write with 64 KiB SRAM bank switching.
- Write operations perform an immediate native readback comparison before returning success.
- Non-mirrored 1 Mbit writes are refused if a pre-write 128 KiB read shows mirrored 64 KiB banks.

It does not yet implement broad save-type auto-detection, GB/GBC saves, GBA FLASH saves, other unlicensed SRAM bank-switching variants, ROM dumping, ROM flashing, RTC handling, or firmware updates.

## Native EEPROM64K Workflow

```bash
cargo run -p framkey-cli -- device probe --device gbx-cart --port /dev/cu.usbserial-210 --save-type gba-eeprom-64k
cargo run -p framkey-cli -- device read-save --device gbx-cart --port /dev/cu.usbserial-210 --save-type gba-eeprom-64k --out read.sav
cargo run -p framkey-cli -- device write-save --device gbx-cart --port /dev/cu.usbserial-210 --save-type gba-eeprom-64k --input read.sav
cargo run -p framkey-cli -- device read-save --device gbx-cart --port /dev/cu.usbserial-210 --save-type gba-eeprom-64k --out after.sav
cmp read.sav after.sav
```

For a standard 32 KiB GBA SRAM/FRAM cartridge, use `--save-type gba-sram-fram-256k` with the same read/write/readback workflow.

For a 512 Kbit / 64 KiB GBA SRAM/FRAM cartridge, use `--save-type gba-sram-fram-512kbit`. This is the recommended target for the current modified A88J cartridge because its first 64 KiB SRAM/FRAM window is stable and sufficient for FRAMKey.

For a 1 Mbit / 128 KiB SRAM/FRAM cartridge, use `--save-type gba-sram-fram-1mbit`. This path treats the save area as two 64 KiB banks and switches banks before each bank read/write. If the cartridge or mod wiring mirrors those banks, the CLI will still read 128 KiB but will refuse writes whose two 64 KiB banks differ.

## Modified Cartridge Note

Stock ROM metadata and a cartridge's physical save bus can disagree when a cartridge has been modified. The A88J validation cartridge used on 2026-05-31 is FRAM-modded and presents as SRAM/FRAM even though stock A88J metadata points to EEPROM64K. For modified cartridges, choose the save type that matches the physical save bus and keep a separate artifact record for each protocol tested.

For the same A88J cartridge, both native `gba-sram-fram-1mbit` and FlashGBX v3.37 `sram1m` reads returned byte-identical 128 KiB images with mirrored 64 KiB banks. A bank1-only marker write also appeared in bank0, then the original image was restored. Treat this cartridge's independent upper 64 KiB as unverified until the FRAM mod's bank-select wiring is identified; use `gba-sram-fram-512kbit` for normal development on this card.
