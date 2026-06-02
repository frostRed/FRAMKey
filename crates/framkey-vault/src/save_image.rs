use framkey_core::{FramkeyError, Generation, Result};

use crate::{
    constants::{
        SAVE_IMAGE_FORMAT_VERSION, SAVE_IMAGE_HEADER_LEN, SAVE_MAGIC, SAVE_SLOT_HEADER_LEN,
        SAVE_SLOT_MAGIC,
    },
    types::{SaveImageInspection, SaveSlot, SaveSlotInspection},
};

pub fn build_save_image_with_payload(
    image_size: usize,
    active_slot: SaveSlot,
    generation: Generation,
    payload: &[u8],
) -> Result<Vec<u8>> {
    let layout = SaveImageLayout::new(image_size)?;
    if payload.len() > layout.slot_payload_capacity() {
        return Err(FramkeyError::invalid_data(format!(
            "payload length {} exceeds slot payload capacity {}",
            payload.len(),
            layout.slot_payload_capacity()
        )));
    }

    let mut image = vec![0xFF; image_size];
    write_slot(&mut image, &layout, SaveSlot::A, generation, payload)?;
    write_slot(&mut image, &layout, SaveSlot::B, Generation(0), &[])?;
    write_header(&mut image, &layout, active_slot, generation)?;
    Ok(image)
}

pub fn inspect_save_image(image: &[u8]) -> Result<SaveImageInspection> {
    let header = ParsedSaveHeader::parse(image)?;
    let layout = SaveImageLayout::new(header.image_size)?;
    if header.header_len != SAVE_IMAGE_HEADER_LEN {
        return Err(FramkeyError::unsupported(format!(
            "save image header length {}",
            header.header_len
        )));
    }
    if header.slot_size != layout.slot_size {
        return Err(FramkeyError::invalid_data(format!(
            "slot size mismatch: header says {}, layout says {}",
            header.slot_size, layout.slot_size
        )));
    }

    let active_slot_bytes = slot_region(image, &layout, header.active_slot)?;
    let active_slot_hash = blake3::hash(active_slot_bytes);
    let slots = [SaveSlot::A, SaveSlot::B]
        .into_iter()
        .map(|slot| inspect_slot(image, &layout, slot))
        .collect::<Result<Vec<_>>>()?;

    Ok(SaveImageInspection {
        image_size: image.len(),
        header_len: header.header_len,
        slot_size: layout.slot_size,
        active_slot: header.active_slot,
        latest_generation: header.latest_generation.0,
        active_slot_hash: hash_to_hex(active_slot_hash.as_bytes()),
        active_slot_hash_valid: active_slot_hash.as_bytes() == &header.active_slot_hash,
        slots,
    })
}

pub fn active_slot_payload(image: &[u8]) -> Result<&[u8]> {
    let header = ParsedSaveHeader::parse(image)?;
    let layout = SaveImageLayout::new(header.image_size)?;
    let slot_bytes = slot_region(image, &layout, header.active_slot)?;
    let parsed = parse_slot_payload(slot_bytes, &layout, header.active_slot)?;
    let actual_payload_hash = blake3::hash(parsed.payload);
    if actual_payload_hash.as_bytes() != &parsed.expected_payload_hash {
        return Err(FramkeyError::invalid_data(
            "active slot payload hash is invalid",
        ));
    }

    Ok(parsed.payload)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SaveImageLayout {
    image_size: usize,
    slot_size: usize,
}

impl SaveImageLayout {
    pub(crate) fn new(image_size: usize) -> Result<Self> {
        if image_size < SAVE_IMAGE_HEADER_LEN + (SAVE_SLOT_HEADER_LEN * 2) {
            return Err(FramkeyError::invalid_data(format!(
                "save image size {image_size} is too small"
            )));
        }

        let slot_area = image_size - SAVE_IMAGE_HEADER_LEN;
        if !slot_area.is_multiple_of(2) {
            return Err(FramkeyError::invalid_data(
                "save image slot area must divide evenly into two slots",
            ));
        }

        let slot_size = slot_area / 2;
        if slot_size < SAVE_SLOT_HEADER_LEN {
            return Err(FramkeyError::invalid_data(
                "save image slot is smaller than slot header",
            ));
        }

        Ok(Self {
            image_size,
            slot_size,
        })
    }

    fn slot_payload_capacity(self) -> usize {
        self.slot_size - SAVE_SLOT_HEADER_LEN
    }

    pub(crate) fn slot_offset(self, slot: SaveSlot) -> usize {
        SAVE_IMAGE_HEADER_LEN + (slot.index() as usize * self.slot_size)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSaveHeader {
    header_len: usize,
    image_size: usize,
    slot_size: usize,
    active_slot: SaveSlot,
    latest_generation: Generation,
    active_slot_hash: [u8; 32],
}

impl ParsedSaveHeader {
    fn parse(image: &[u8]) -> Result<Self> {
        if image.len() < SAVE_IMAGE_HEADER_LEN {
            return Err(FramkeyError::invalid_data(
                "save image is too small for FRAMKey header",
            ));
        }

        if image[0..8] != SAVE_MAGIC {
            return Err(FramkeyError::invalid_data("save image magic mismatch"));
        }

        let version = read_u16_le(image, 8)?;
        if version != SAVE_IMAGE_FORMAT_VERSION {
            return Err(FramkeyError::unsupported(format!(
                "save image format version {version}"
            )));
        }

        let header_len = read_u16_le(image, 10)? as usize;
        let image_size = read_u32_le(image, 12)? as usize;
        if image_size != image.len() {
            return Err(FramkeyError::invalid_data(format!(
                "save image size mismatch: header says {image_size}, actual {}",
                image.len()
            )));
        }

        let slot_size = read_u32_le(image, 16)? as usize;
        let active_slot = SaveSlot::from_index(image[20])?;
        let latest_generation = Generation(read_u64_le(image, 24)?);
        let mut active_slot_hash = [0_u8; 32];
        active_slot_hash.copy_from_slice(&image[32..64]);

        Ok(Self {
            header_len,
            image_size,
            slot_size,
            active_slot,
            latest_generation,
            active_slot_hash,
        })
    }
}

fn write_header(
    image: &mut [u8],
    layout: &SaveImageLayout,
    active_slot: SaveSlot,
    latest_generation: Generation,
) -> Result<()> {
    let active_slot_hash = blake3::hash(slot_region(image, layout, active_slot)?);

    let mut header = [0_u8; SAVE_IMAGE_HEADER_LEN];
    header[0..8].copy_from_slice(&SAVE_MAGIC);
    header[8..10].copy_from_slice(&SAVE_IMAGE_FORMAT_VERSION.to_le_bytes());
    header[10..12].copy_from_slice(&(SAVE_IMAGE_HEADER_LEN as u16).to_le_bytes());
    header[12..16].copy_from_slice(&(layout.image_size as u32).to_le_bytes());
    header[16..20].copy_from_slice(&(layout.slot_size as u32).to_le_bytes());
    header[20] = active_slot.index();
    header[24..32].copy_from_slice(&latest_generation.0.to_le_bytes());
    header[32..64].copy_from_slice(active_slot_hash.as_bytes());

    image[..SAVE_IMAGE_HEADER_LEN].copy_from_slice(&header);
    Ok(())
}

fn write_slot(
    image: &mut [u8],
    layout: &SaveImageLayout,
    slot: SaveSlot,
    generation: Generation,
    payload: &[u8],
) -> Result<()> {
    if payload.len() > layout.slot_payload_capacity() {
        return Err(FramkeyError::invalid_data("slot payload is too large"));
    }

    let slot_offset = layout.slot_offset(slot);
    let slot_end = slot_offset + layout.slot_size;
    image[slot_offset..slot_end].fill(0xFF);

    let payload_hash = blake3::hash(payload);
    let mut header = [0_u8; SAVE_SLOT_HEADER_LEN];
    header[0..8].copy_from_slice(&SAVE_SLOT_MAGIC);
    header[8..10].copy_from_slice(&SAVE_IMAGE_FORMAT_VERSION.to_le_bytes());
    header[10] = slot.index();
    header[12..20].copy_from_slice(&generation.0.to_le_bytes());
    header[20..24].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    header[24..56].copy_from_slice(payload_hash.as_bytes());

    image[slot_offset..slot_offset + SAVE_SLOT_HEADER_LEN].copy_from_slice(&header);
    let payload_offset = slot_offset + SAVE_SLOT_HEADER_LEN;
    image[payload_offset..payload_offset + payload.len()].copy_from_slice(payload);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct ParsedSlotPayload<'a> {
    generation: u64,
    payload: &'a [u8],
    expected_payload_hash: [u8; 32],
}

fn parse_slot_payload<'a>(
    slot_bytes: &'a [u8],
    layout: &SaveImageLayout,
    slot: SaveSlot,
) -> Result<ParsedSlotPayload<'a>> {
    if slot_bytes[0..8] != SAVE_SLOT_MAGIC {
        return Err(FramkeyError::invalid_data(format!(
            "slot {:?} magic mismatch",
            slot
        )));
    }

    let version = read_u16_le(slot_bytes, 8)?;
    if version != SAVE_IMAGE_FORMAT_VERSION {
        return Err(FramkeyError::unsupported(format!(
            "slot {:?} format version {version}",
            slot
        )));
    }

    let parsed_slot = SaveSlot::from_index(slot_bytes[10])?;
    if parsed_slot != slot {
        return Err(FramkeyError::invalid_data(format!(
            "slot index mismatch: expected {:?}, found {:?}",
            slot, parsed_slot
        )));
    }

    let generation = read_u64_le(slot_bytes, 12)?;
    let payload_len = read_u32_le(slot_bytes, 20)? as usize;
    if payload_len > layout.slot_payload_capacity() {
        return Err(FramkeyError::invalid_data(format!(
            "slot {:?} payload length {payload_len} exceeds capacity {}",
            slot,
            layout.slot_payload_capacity()
        )));
    }

    let mut expected_payload_hash = [0_u8; 32];
    expected_payload_hash.copy_from_slice(&slot_bytes[24..56]);
    let payload_start = SAVE_SLOT_HEADER_LEN;
    let payload_end = payload_start + payload_len;
    let payload = &slot_bytes[payload_start..payload_end];

    Ok(ParsedSlotPayload {
        generation,
        payload,
        expected_payload_hash,
    })
}

fn inspect_slot(
    image: &[u8],
    layout: &SaveImageLayout,
    slot: SaveSlot,
) -> Result<SaveSlotInspection> {
    let slot_bytes = slot_region(image, layout, slot)?;
    let parsed = parse_slot_payload(slot_bytes, layout, slot)?;
    let actual_payload_hash = blake3::hash(parsed.payload);

    Ok(SaveSlotInspection {
        slot,
        generation: parsed.generation,
        payload_len: parsed.payload.len(),
        payload_hash: hash_to_hex(actual_payload_hash.as_bytes()),
        payload_hash_valid: actual_payload_hash.as_bytes() == &parsed.expected_payload_hash,
        payload_preview: redacted_payload_preview(parsed.payload.len()),
    })
}

fn redacted_payload_preview(payload_len: usize) -> String {
    format!("<redacted {payload_len} bytes>")
}

fn slot_region<'a>(image: &'a [u8], layout: &SaveImageLayout, slot: SaveSlot) -> Result<&'a [u8]> {
    let start = layout.slot_offset(slot);
    let end = start + layout.slot_size;
    image
        .get(start..end)
        .ok_or_else(|| FramkeyError::invalid_data("slot region is outside save image"))
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Result<u16> {
    let bytes = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| FramkeyError::invalid_data("u16 field outside buffer"))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32> {
    let bytes = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| FramkeyError::invalid_data("u32 field outside buffer"))?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_u64_le(bytes: &[u8], offset: usize) -> Result<u64> {
    let bytes = bytes
        .get(offset..offset + 8)
        .ok_or_else(|| FramkeyError::invalid_data("u64 field outside buffer"))?;
    Ok(u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}

fn hash_to_hex(hash: &[u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    for byte in hash {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}
