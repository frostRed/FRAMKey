use framkey_core::{FramkeyError, Generation, Result};
use reed_solomon_erasure::galois_8::ReedSolomon;

use crate::{
    constants::{
        SAVE_IMAGE_FORMAT_VERSION, SAVE_IMAGE_HEADER_LEN, SAVE_MAGIC, SAVE_RS_DATA_SHARDS,
        SAVE_RS_PARITY_SHARDS, SAVE_RS_TOTAL_SHARDS, SAVE_SUPERBLOCK_COPIES, SAVE_SUPERBLOCK_LEN,
    },
    types::{SaveImageInspection, SaveShardInspection, SaveSuperblockInspection},
};

const SUPERBLOCK_PAYLOAD_HASH_OFFSET: usize = 48;
const SUPERBLOCK_SHARD_HASHES_OFFSET: usize = 80;
const SUPERBLOCK_HASHED_LEN: usize = 960;
const SUPERBLOCK_HASH_OFFSET: usize = 960;
const HASH_LEN: usize = 32;

pub fn build_save_image_with_payload(
    image_size: usize,
    generation: Generation,
    payload: &[u8],
) -> Result<Vec<u8>> {
    let layout = SaveImageLayout::new(image_size)?;
    if payload.len() > layout.payload_capacity() {
        return Err(FramkeyError::invalid_data(format!(
            "payload length {} exceeds Reed-Solomon payload capacity {}",
            payload.len(),
            layout.payload_capacity()
        )));
    }

    let mut image = vec![0xFF; image_size];
    let mut shards = build_data_shards(&layout, payload);
    shards.extend((0..SAVE_RS_PARITY_SHARDS).map(|_| vec![0_u8; layout.shard_size]));
    reed_solomon()?.encode(&mut shards).map_err(|error| {
        FramkeyError::invalid_data(format!("Reed-Solomon encode failed: {error}"))
    })?;

    let shard_hashes = shard_hashes(&shards)?;
    write_interleaved_shards(&mut image, &layout, &shards)?;
    write_superblocks(
        &mut image,
        &layout,
        generation,
        payload.len(),
        *blake3::hash(payload).as_bytes(),
        &shard_hashes,
    )?;

    Ok(image)
}

pub fn inspect_save_image(image: &[u8]) -> Result<SaveImageInspection> {
    let recovered = recover_save_image(image)?;

    Ok(SaveImageInspection {
        image_size: image.len(),
        header_len: SAVE_IMAGE_HEADER_LEN,
        format_version: SAVE_IMAGE_FORMAT_VERSION,
        generation: recovered.superblock.generation.0,
        payload_len: recovered.payload.len(),
        payload_hash: hash_to_hex(blake3::hash(&recovered.payload).as_bytes()),
        payload_hash_valid: true,
        data_shards: SAVE_RS_DATA_SHARDS,
        parity_shards: SAVE_RS_PARITY_SHARDS,
        shard_size: recovered.layout.shard_size,
        valid_shard_count: recovered.valid_shard_count,
        recovered_shard_count: recovered.recovered_shard_count,
        superblocks: recovered.superblock_inspections,
        shards: recovered.shard_inspections,
    })
}

pub fn save_image_payload(image: &[u8]) -> Result<Vec<u8>> {
    Ok(recover_save_image(image)?.payload)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SaveImageLayout {
    image_size: usize,
    shard_size: usize,
}

impl SaveImageLayout {
    pub(crate) fn new(image_size: usize) -> Result<Self> {
        if image_size < SAVE_IMAGE_HEADER_LEN + SAVE_RS_TOTAL_SHARDS {
            return Err(FramkeyError::invalid_data(format!(
                "save image size {image_size} is too small for FRAMKey Reed-Solomon layout"
            )));
        }

        let shard_region_len = image_size - SAVE_IMAGE_HEADER_LEN;
        let shard_size = shard_region_len / SAVE_RS_TOTAL_SHARDS;
        if shard_size == 0 {
            return Err(FramkeyError::invalid_data(
                "save image shard size must be greater than zero",
            ));
        }

        Ok(Self {
            image_size,
            shard_size,
        })
    }

    fn payload_capacity(self) -> usize {
        self.shard_size * SAVE_RS_DATA_SHARDS
    }

    pub(crate) fn shard_byte_offset(self, shard_index: usize, byte_index: usize) -> Result<usize> {
        if shard_index >= SAVE_RS_TOTAL_SHARDS {
            return Err(FramkeyError::invalid_data(format!(
                "shard index {shard_index} outside Reed-Solomon layout"
            )));
        }
        if byte_index >= self.shard_size {
            return Err(FramkeyError::invalid_data(format!(
                "shard byte index {byte_index} outside Reed-Solomon layout"
            )));
        }

        Ok(SAVE_IMAGE_HEADER_LEN + (byte_index * SAVE_RS_TOTAL_SHARDS) + shard_index)
    }

    fn encoded_shard_region_len(self) -> usize {
        self.shard_size * SAVE_RS_TOTAL_SHARDS
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSuperblock {
    copy_index: usize,
    image_size: usize,
    generation: Generation,
    payload_len: usize,
    payload_hash: [u8; HASH_LEN],
    shard_hashes: [[u8; HASH_LEN]; SAVE_RS_TOTAL_SHARDS],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecoveredSaveImage {
    layout: SaveImageLayout,
    superblock: ParsedSuperblock,
    payload: Vec<u8>,
    valid_shard_count: usize,
    recovered_shard_count: usize,
    superblock_inspections: Vec<SaveSuperblockInspection>,
    shard_inspections: Vec<SaveShardInspection>,
}

fn build_data_shards(layout: &SaveImageLayout, payload: &[u8]) -> Vec<Vec<u8>> {
    let mut payload_region = vec![0_u8; layout.payload_capacity()];
    payload_region[..payload.len()].copy_from_slice(payload);
    payload_region
        .chunks_exact(layout.shard_size)
        .map(<[u8]>::to_vec)
        .collect()
}

fn recover_save_image(image: &[u8]) -> Result<RecoveredSaveImage> {
    let layout = SaveImageLayout::new(image.len())?;
    let (superblock, superblock_inspections) = parse_superblocks(image, &layout)?;
    let raw_shards = read_interleaved_shards(image, &layout)?;

    let mut shard_inspections = Vec::with_capacity(SAVE_RS_TOTAL_SHARDS);
    let mut shards = Vec::with_capacity(SAVE_RS_TOTAL_SHARDS);
    let mut valid_shard_count = 0_usize;
    for (index, shard) in raw_shards.into_iter().enumerate() {
        let actual_hash = blake3::hash(&shard);
        let hash_valid = actual_hash.as_bytes() == &superblock.shard_hashes[index];
        if hash_valid {
            valid_shard_count += 1;
            shards.push(Some(shard));
        } else {
            shards.push(None);
        }
        shard_inspections.push(SaveShardInspection {
            shard_index: index,
            is_data_shard: index < SAVE_RS_DATA_SHARDS,
            hash: hash_to_hex(actual_hash.as_bytes()),
            hash_valid,
            recovered: false,
        });
    }

    if valid_shard_count < SAVE_RS_DATA_SHARDS {
        return Err(FramkeyError::invalid_data(format!(
            "only {valid_shard_count} valid Reed-Solomon shards remain; need at least {SAVE_RS_DATA_SHARDS}"
        )));
    }

    reed_solomon()?.reconstruct(&mut shards).map_err(|error| {
        FramkeyError::invalid_data(format!("Reed-Solomon reconstruction failed: {error}"))
    })?;
    let recovered_shard_count = SAVE_RS_TOTAL_SHARDS - valid_shard_count;

    let shards = shards
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| FramkeyError::invalid_data("Reed-Solomon reconstruction left holes"))?;
    for (index, shard) in shards.iter().enumerate() {
        let actual_hash = blake3::hash(shard);
        if actual_hash.as_bytes() != &superblock.shard_hashes[index] {
            return Err(FramkeyError::invalid_data(format!(
                "reconstructed shard {index} hash mismatch"
            )));
        }
        if !shard_inspections[index].hash_valid {
            shard_inspections[index].hash = hash_to_hex(actual_hash.as_bytes());
            shard_inspections[index].hash_valid = true;
            shard_inspections[index].recovered = true;
        }
    }

    let payload = payload_from_shards(&superblock, &layout, &shards)?;
    let payload_hash = blake3::hash(&payload);
    if payload_hash.as_bytes() != &superblock.payload_hash {
        return Err(FramkeyError::invalid_data(
            "reconstructed save payload hash mismatch",
        ));
    }

    Ok(RecoveredSaveImage {
        layout,
        superblock,
        payload,
        valid_shard_count,
        recovered_shard_count,
        superblock_inspections,
        shard_inspections,
    })
}

fn write_interleaved_shards(
    image: &mut [u8],
    layout: &SaveImageLayout,
    shards: &[Vec<u8>],
) -> Result<()> {
    if shards.len() != SAVE_RS_TOTAL_SHARDS {
        return Err(FramkeyError::invalid_data(format!(
            "expected {SAVE_RS_TOTAL_SHARDS} shards, found {}",
            shards.len()
        )));
    }
    for shard in shards {
        if shard.len() != layout.shard_size {
            return Err(FramkeyError::invalid_data(format!(
                "shard length {} does not match layout shard size {}",
                shard.len(),
                layout.shard_size
            )));
        }
    }

    for byte_index in 0..layout.shard_size {
        for shard_index in 0..SAVE_RS_TOTAL_SHARDS {
            let offset = layout.shard_byte_offset(shard_index, byte_index)?;
            image[offset] = shards[shard_index][byte_index];
        }
    }

    Ok(())
}

fn read_interleaved_shards(image: &[u8], layout: &SaveImageLayout) -> Result<Vec<Vec<u8>>> {
    let end = SAVE_IMAGE_HEADER_LEN + layout.encoded_shard_region_len();
    if end > image.len() {
        return Err(FramkeyError::invalid_data(
            "interleaved shard region is outside save image",
        ));
    }

    let mut shards = vec![vec![0_u8; layout.shard_size]; SAVE_RS_TOTAL_SHARDS];
    for byte_index in 0..layout.shard_size {
        for (shard_index, shard) in shards.iter_mut().enumerate() {
            let offset = layout.shard_byte_offset(shard_index, byte_index)?;
            shard[byte_index] = image[offset];
        }
    }
    Ok(shards)
}

fn write_superblocks(
    image: &mut [u8],
    layout: &SaveImageLayout,
    generation: Generation,
    payload_len: usize,
    payload_hash: [u8; HASH_LEN],
    shard_hashes: &[[u8; HASH_LEN]; SAVE_RS_TOTAL_SHARDS],
) -> Result<()> {
    for copy_index in 0..SAVE_SUPERBLOCK_COPIES {
        let mut block = [0_u8; SAVE_SUPERBLOCK_LEN];
        block[0..8].copy_from_slice(&SAVE_MAGIC);
        block[8..10].copy_from_slice(&SAVE_IMAGE_FORMAT_VERSION.to_le_bytes());
        block[10..12].copy_from_slice(&(SAVE_SUPERBLOCK_LEN as u16).to_le_bytes());
        block[12..16].copy_from_slice(&(layout.image_size as u32).to_le_bytes());
        block[16..20].copy_from_slice(&(SAVE_IMAGE_HEADER_LEN as u32).to_le_bytes());
        block[20..28].copy_from_slice(&generation.0.to_le_bytes());
        block[28..32].copy_from_slice(&(payload_len as u32).to_le_bytes());
        block[32] = SAVE_RS_DATA_SHARDS as u8;
        block[33] = SAVE_RS_PARITY_SHARDS as u8;
        block[34] = SAVE_RS_TOTAL_SHARDS as u8;
        block[35] = copy_index as u8;
        block[36..40].copy_from_slice(&(layout.shard_size as u32).to_le_bytes());
        block[40..44].copy_from_slice(&(layout.payload_capacity() as u32).to_le_bytes());
        block[SUPERBLOCK_PAYLOAD_HASH_OFFSET..SUPERBLOCK_PAYLOAD_HASH_OFFSET + HASH_LEN]
            .copy_from_slice(&payload_hash);
        for (index, hash) in shard_hashes.iter().enumerate() {
            let start = SUPERBLOCK_SHARD_HASHES_OFFSET + (index * HASH_LEN);
            block[start..start + HASH_LEN].copy_from_slice(hash);
        }
        let header_hash = blake3::hash(&block[..SUPERBLOCK_HASHED_LEN]);
        block[SUPERBLOCK_HASH_OFFSET..SUPERBLOCK_HASH_OFFSET + HASH_LEN]
            .copy_from_slice(header_hash.as_bytes());

        let start = copy_index * SAVE_SUPERBLOCK_LEN;
        let end = start + SAVE_SUPERBLOCK_LEN;
        image[start..end].copy_from_slice(&block);
    }

    Ok(())
}

fn parse_superblocks(
    image: &[u8],
    layout: &SaveImageLayout,
) -> Result<(ParsedSuperblock, Vec<SaveSuperblockInspection>)> {
    if image.len() < SAVE_IMAGE_HEADER_LEN {
        return Err(FramkeyError::invalid_data(
            "save image is too small for FRAMKey Reed-Solomon header",
        ));
    }

    let mut parsed = Vec::new();
    let mut inspections = Vec::with_capacity(SAVE_SUPERBLOCK_COPIES);
    for copy_index in 0..SAVE_SUPERBLOCK_COPIES {
        let start = copy_index * SAVE_SUPERBLOCK_LEN;
        let end = start + SAVE_SUPERBLOCK_LEN;
        match parse_superblock(copy_index, &image[start..end], layout) {
            Ok(superblock) => {
                inspections.push(SaveSuperblockInspection {
                    copy_index,
                    valid: true,
                    generation: Some(superblock.generation.0),
                    error: None,
                });
                parsed.push(superblock);
            }
            Err(error) => inspections.push(SaveSuperblockInspection {
                copy_index,
                valid: false,
                generation: None,
                error: Some(error.to_string()),
            }),
        }
    }

    parsed
        .into_iter()
        .max_by_key(|superblock| {
            (
                superblock.generation.0,
                std::cmp::Reverse(superblock.copy_index),
            )
        })
        .map(|superblock| (superblock, inspections))
        .ok_or_else(|| FramkeyError::invalid_data("no valid FRAMKey Reed-Solomon superblock found"))
}

fn parse_superblock(
    copy_index: usize,
    block: &[u8],
    layout: &SaveImageLayout,
) -> Result<ParsedSuperblock> {
    if block.len() != SAVE_SUPERBLOCK_LEN {
        return Err(FramkeyError::invalid_data("superblock length mismatch"));
    }
    if block[0..8] != SAVE_MAGIC {
        return Err(FramkeyError::invalid_data("save image magic mismatch"));
    }

    let version = read_u16_le(block, 8)?;
    if version != SAVE_IMAGE_FORMAT_VERSION {
        return Err(FramkeyError::unsupported(format!(
            "save image format version {version}"
        )));
    }
    let superblock_len = read_u16_le(block, 10)? as usize;
    if superblock_len != SAVE_SUPERBLOCK_LEN {
        return Err(FramkeyError::invalid_data(format!(
            "superblock length {superblock_len} does not match expected {SAVE_SUPERBLOCK_LEN}"
        )));
    }
    let image_size = read_u32_le(block, 12)? as usize;
    if image_size != layout.image_size {
        return Err(FramkeyError::invalid_data(format!(
            "save image size mismatch: superblock says {image_size}, actual {}",
            layout.image_size
        )));
    }
    let header_len = read_u32_le(block, 16)? as usize;
    if header_len != SAVE_IMAGE_HEADER_LEN {
        return Err(FramkeyError::invalid_data(format!(
            "save image header length {header_len} does not match expected {SAVE_IMAGE_HEADER_LEN}"
        )));
    }
    let generation = Generation(read_u64_le(block, 20)?);
    let payload_len = read_u32_le(block, 28)? as usize;
    if payload_len > layout.payload_capacity() {
        return Err(FramkeyError::invalid_data(format!(
            "payload length {payload_len} exceeds Reed-Solomon payload capacity {}",
            layout.payload_capacity()
        )));
    }
    validate_shard_counts(block[32], block[33], block[34])?;
    let block_copy_index = block[35] as usize;
    if block_copy_index != copy_index {
        return Err(FramkeyError::invalid_data(format!(
            "superblock copy index mismatch: expected {copy_index}, found {block_copy_index}"
        )));
    }
    let shard_size = read_u32_le(block, 36)? as usize;
    if shard_size != layout.shard_size {
        return Err(FramkeyError::invalid_data(format!(
            "shard size mismatch: superblock says {shard_size}, layout says {}",
            layout.shard_size
        )));
    }
    let payload_capacity = read_u32_le(block, 40)? as usize;
    if payload_capacity != layout.payload_capacity() {
        return Err(FramkeyError::invalid_data(format!(
            "payload capacity mismatch: superblock says {payload_capacity}, layout says {}",
            layout.payload_capacity()
        )));
    }

    let expected_header_hash = blake3::hash(&block[..SUPERBLOCK_HASHED_LEN]);
    let actual_header_hash = read_hash(block, SUPERBLOCK_HASH_OFFSET, "superblock hash")?;
    if expected_header_hash.as_bytes() != &actual_header_hash {
        return Err(FramkeyError::invalid_data("superblock hash mismatch"));
    }

    let payload_hash = read_hash(block, SUPERBLOCK_PAYLOAD_HASH_OFFSET, "payload hash")?;
    let mut shard_hashes = [[0_u8; HASH_LEN]; SAVE_RS_TOTAL_SHARDS];
    for (index, hash) in shard_hashes.iter_mut().enumerate() {
        let start = SUPERBLOCK_SHARD_HASHES_OFFSET + (index * HASH_LEN);
        *hash = read_hash(block, start, "shard hash")?;
    }

    Ok(ParsedSuperblock {
        copy_index,
        image_size,
        generation,
        payload_len,
        payload_hash,
        shard_hashes,
    })
}

fn validate_shard_counts(data_shards: u8, parity_shards: u8, total_shards: u8) -> Result<()> {
    if data_shards as usize != SAVE_RS_DATA_SHARDS
        || parity_shards as usize != SAVE_RS_PARITY_SHARDS
        || total_shards as usize != SAVE_RS_TOTAL_SHARDS
    {
        return Err(FramkeyError::unsupported(format!(
            "save image Reed-Solomon layout {data_shards}+{parity_shards}={total_shards}"
        )));
    }
    Ok(())
}

fn payload_from_shards(
    superblock: &ParsedSuperblock,
    layout: &SaveImageLayout,
    shards: &[Vec<u8>],
) -> Result<Vec<u8>> {
    if shards.len() < SAVE_RS_DATA_SHARDS {
        return Err(FramkeyError::invalid_data(
            "Reed-Solomon data shards are missing",
        ));
    }
    let mut payload_region = Vec::with_capacity(layout.payload_capacity());
    for shard in &shards[..SAVE_RS_DATA_SHARDS] {
        if shard.len() != layout.shard_size {
            return Err(FramkeyError::invalid_data(
                "Reed-Solomon shard size mismatch",
            ));
        }
        payload_region.extend_from_slice(shard);
    }
    Ok(payload_region[..superblock.payload_len].to_vec())
}

fn shard_hashes(shards: &[Vec<u8>]) -> Result<[[u8; HASH_LEN]; SAVE_RS_TOTAL_SHARDS]> {
    if shards.len() != SAVE_RS_TOTAL_SHARDS {
        return Err(FramkeyError::invalid_data(format!(
            "expected {SAVE_RS_TOTAL_SHARDS} Reed-Solomon shards, found {}",
            shards.len()
        )));
    }

    let mut hashes = [[0_u8; HASH_LEN]; SAVE_RS_TOTAL_SHARDS];
    for (index, shard) in shards.iter().enumerate() {
        hashes[index] = *blake3::hash(shard).as_bytes();
    }
    Ok(hashes)
}

fn reed_solomon() -> Result<ReedSolomon> {
    ReedSolomon::new(SAVE_RS_DATA_SHARDS, SAVE_RS_PARITY_SHARDS).map_err(|error| {
        FramkeyError::invalid_data(format!("Reed-Solomon layout is invalid: {error}"))
    })
}

fn read_hash(bytes: &[u8], offset: usize, field: &str) -> Result<[u8; HASH_LEN]> {
    let bytes = bytes
        .get(offset..offset + HASH_LEN)
        .ok_or_else(|| FramkeyError::invalid_data(format!("{field} outside buffer")))?;
    let mut hash = [0_u8; HASH_LEN];
    hash.copy_from_slice(bytes);
    Ok(hash)
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

fn hash_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
