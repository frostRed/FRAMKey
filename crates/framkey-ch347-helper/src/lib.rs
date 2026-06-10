use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use framkey_ch347::{Ch347Config, Ch347Device, Ch347SpiSpeed};
use framkey_core::FramkeyError;
use framkey_crypto::encode_hex;
use framkey_device::{SaveImage, VaultDevice};
use serde::{Deserialize, Serialize};

pub const CH347_HELPER_OPERATION: &str = CH347_HELPER_WRITE_OPERATION;
pub const CH347_HELPER_WRITE_OPERATION: &str = "write_ch347_backup";
pub const CH347_HELPER_READ_OPERATION: &str = "read_ch347_backup";
pub const CH347_HELPER_PROCESS: &str = "framkey-ch347-helper";
pub const MAX_CH347_HELPER_JSON_BYTES: usize = 16 * 1024;
pub const MAX_CH347_HELPER_IMAGE_BYTES: usize = 512 * 1024 * 1024;
pub const PHYSICAL_BACKUP_ROM_HEADER_BYTES: usize = 4096;
const PHYSICAL_BACKUP_ROM_MAGIC: &[u8; 16] = b"FRAMKEYPHYSBK01!";
const PHYSICAL_BACKUP_ROM_VERSION: u32 = 1;
const FULL_CHIP_IMAGE_FORMAT: &str = "full_chip_image";
const PHYSICAL_BACKUP_ROM_FORMAT: &str = "framkey_physical_backup_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ch347HelperWriteRequest {
    pub operation: String,
    pub input_path: PathBuf,
    pub flashrom_path: PathBuf,
    #[serde(default)]
    pub chip: Option<String>,
    #[serde(default)]
    pub spispeed: Option<String>,
    pub expected_size: usize,
    pub expected_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ch347HelperReadRequest {
    pub operation: String,
    pub output_dir: PathBuf,
    pub flashrom_path: PathBuf,
    #[serde(default)]
    pub chip: Option<String>,
    #[serde(default)]
    pub spispeed: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Ch347HelperRequest {
    Write(Ch347HelperWriteRequest),
    Read(Ch347HelperReadRequest),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum Ch347HelperResponse {
    Ok { result: Ch347HelperResult },
    Error { error: Ch347HelperError },
}

impl Ch347HelperResponse {
    pub fn ok(result: impl Into<Ch347HelperResult>) -> Self {
        Self::Ok {
            result: result.into(),
        }
    }

    pub fn error(error: Ch347HelperError) -> Self {
        Self::Error { error }
    }

    pub fn into_result(self) -> std::result::Result<Ch347HelperWriteResult, Ch347HelperError> {
        self.into_write_result()
    }

    pub fn into_write_result(
        self,
    ) -> std::result::Result<Ch347HelperWriteResult, Ch347HelperError> {
        let result = self.into_helper_result()?;
        match result {
            Ch347HelperResult::Write(result) => Ok(result),
            Ch347HelperResult::Read(_) => Err(Ch347HelperError {
                code: "INVALID_DATA".to_owned(),
                message: "CH347 helper returned a read result for a write request".to_owned(),
            }),
        }
    }

    pub fn into_read_result(self) -> std::result::Result<Ch347HelperReadResult, Ch347HelperError> {
        let result = self.into_helper_result()?;
        match result {
            Ch347HelperResult::Read(result) => Ok(result),
            Ch347HelperResult::Write(_) => Err(Ch347HelperError {
                code: "INVALID_DATA".to_owned(),
                message: "CH347 helper returned a write result for a read request".to_owned(),
            }),
        }
    }

    pub fn into_helper_result(self) -> std::result::Result<Ch347HelperResult, Ch347HelperError> {
        match self {
            Self::Ok { result } => Ok(result),
            Self::Error { error } => Err(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Ch347HelperResult {
    Write(Ch347HelperWriteResult),
    Read(Ch347HelperReadResult),
}

impl From<Ch347HelperWriteResult> for Ch347HelperResult {
    fn from(result: Ch347HelperWriteResult) -> Self {
        Self::Write(result)
    }
}

impl From<Ch347HelperReadResult> for Ch347HelperResult {
    fn from(result: Ch347HelperReadResult) -> Self {
        Self::Read(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ch347HelperError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ch347HelperWriteResult {
    pub operation: String,
    pub device: String,
    pub helper_process: String,
    pub privileged: bool,
    pub flashrom_path: String,
    pub chip: Option<String>,
    pub chip_detection: String,
    pub spi_speed: String,
    pub save_size: usize,
    pub payload_size: usize,
    pub rom_image_size: usize,
    pub backup_blake3: String,
    pub readback_blake3: String,
    pub payload_blake3: String,
    pub readback_payload_blake3: String,
    pub rom_image_blake3: String,
    pub readback_rom_image_blake3: String,
    pub storage_format: String,
    pub verified: bool,
    pub write_count: u8,
    pub readback_count: u8,
    pub layout_parsed: bool,
    pub backup_bytes_printed: bool,
    pub wallet_secret_touched: bool,
    pub recovery_share_bytes_printed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ch347HelperReadResult {
    pub operation: String,
    pub device: String,
    pub helper_process: String,
    pub privileged: bool,
    pub flashrom_path: String,
    pub chip: Option<String>,
    pub chip_detection: String,
    pub spi_speed: String,
    pub output_path: String,
    pub output_kind: String,
    pub output_size: usize,
    pub output_blake3: String,
    pub save_size: usize,
    pub payload_size: usize,
    pub rom_image_size: usize,
    pub payload_blake3: String,
    pub rom_image_blake3: String,
    pub storage_format: String,
    pub verified: bool,
    pub read_count: u8,
    pub layout_parsed: bool,
    pub backup_bytes_printed: bool,
    pub wallet_secret_touched: bool,
    pub recovery_share_bytes_printed: bool,
}

pub fn read_request_file(path: &Path) -> Result<Ch347HelperRequest> {
    let bytes = read_limited_file(path, MAX_CH347_HELPER_JSON_BYTES)
        .with_context(|| format!("failed to read CH347 helper request {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "failed to parse CH347 helper request JSON from {}",
            path.display()
        )
    })
}

pub fn write_response_file(path: &Path, response: &Ch347HelperResponse) -> Result<()> {
    let payload = response_json_bytes(response)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("failed to create CH347 helper response {}", path.display()))?;
    file.write_all(&payload)?;
    file.flush()?;
    Ok(())
}

pub fn response_json_bytes(response: &Ch347HelperResponse) -> Result<Vec<u8>> {
    let mut payload = serde_json::to_vec_pretty(response)?;
    payload.push(b'\n');
    Ok(payload)
}

pub fn read_response_bytes(bytes: &[u8]) -> Result<Ch347HelperResponse> {
    if bytes.len() > MAX_CH347_HELPER_JSON_BYTES {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 helper response exceeds {MAX_CH347_HELPER_JSON_BYTES} bytes"
        ))
        .into());
    }
    serde_json::from_slice(bytes).context("failed to parse CH347 helper response JSON")
}

pub fn read_response_file(path: &Path) -> Result<Ch347HelperResponse> {
    let bytes = read_limited_file(path, MAX_CH347_HELPER_JSON_BYTES)
        .with_context(|| format!("failed to read CH347 helper response {}", path.display()))?;
    read_response_bytes(&bytes).with_context(|| {
        format!(
            "failed to parse CH347 helper response from {}",
            path.display()
        )
    })
}

pub fn execute_request(request: Ch347HelperRequest) -> Result<Ch347HelperResult> {
    match request {
        Ch347HelperRequest::Write(request) => execute_write_request(request).map(Into::into),
        Ch347HelperRequest::Read(request) => execute_read_request(request).map(Into::into),
    }
}

pub fn execute_write_request(request: Ch347HelperWriteRequest) -> Result<Ch347HelperWriteResult> {
    request.validate()?;

    let bytes = fs::read(&request.input_path).with_context(|| {
        format!(
            "failed to read CH347 backup file {}",
            request.input_path.display()
        )
    })?;
    if bytes.len() != request.expected_size {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 helper input size changed: expected {}, got {}",
            request.expected_size,
            bytes.len()
        ))
        .into());
    }
    let backup_blake3 = encode_hex(blake3::hash(&bytes).as_bytes());
    if backup_blake3 != request.expected_blake3 {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 helper input hash changed: expected {}, got {}",
            request.expected_blake3, backup_blake3
        ))
        .into());
    }

    let spi_speed = parse_spi_speed(request.spispeed.as_deref())?;
    let spi_speed_label = spi_speed.unwrap_or(Ch347SpiSpeed::M15).as_flashrom_value();
    let chip_detection = if request.chip.is_some() {
        "override"
    } else {
        "auto"
    };
    let config = Ch347Config {
        chip: request.chip.clone(),
        flashrom_path: Some(request.flashrom_path.clone()),
        spi_speed,
        expected_size: None,
    };
    let probe = Ch347Device::new(config.clone())
        .probe()
        .context("failed to probe CH347 SPI ROM size before preparing backup image")?;
    let prepared = prepare_physical_backup_rom_image(&bytes, probe.save_size)?;
    let image = SaveImage::new(prepared.image);
    let mut device = Ch347Device::new(Ch347Config {
        expected_size: Some(image.len()),
        ..config
    });
    let report = device.write_save_image_verified(&image)?;

    Ok(Ch347HelperWriteResult {
        operation: CH347_HELPER_OPERATION.to_owned(),
        device: "ch347".to_owned(),
        helper_process: CH347_HELPER_PROCESS.to_owned(),
        privileged: true,
        flashrom_path: request.flashrom_path.display().to_string(),
        chip: request.chip,
        chip_detection: chip_detection.to_owned(),
        spi_speed: spi_speed_label.to_owned(),
        save_size: report.save_size,
        payload_size: prepared.payload_size,
        rom_image_size: report.save_size,
        backup_blake3: prepared.payload_blake3.clone(),
        readback_blake3: prepared.payload_blake3.clone(),
        payload_blake3: prepared.payload_blake3.clone(),
        readback_payload_blake3: prepared.payload_blake3,
        rom_image_blake3: report.input_blake3,
        readback_rom_image_blake3: report.readback_blake3,
        storage_format: prepared.storage_format.to_owned(),
        verified: report.exact_match,
        write_count: 1,
        readback_count: 1,
        layout_parsed: false,
        backup_bytes_printed: false,
        wallet_secret_touched: false,
        recovery_share_bytes_printed: false,
    })
}

pub fn execute_read_request(request: Ch347HelperReadRequest) -> Result<Ch347HelperReadResult> {
    request.validate()?;

    let spi_speed = parse_spi_speed(request.spispeed.as_deref())?;
    let spi_speed_label = spi_speed.unwrap_or(Ch347SpiSpeed::M15).as_flashrom_value();
    let chip_detection = if request.chip.is_some() {
        "override"
    } else {
        "auto"
    };
    let config = Ch347Config {
        chip: request.chip.clone(),
        flashrom_path: Some(request.flashrom_path.clone()),
        spi_speed,
        expected_size: None,
    };
    let probe = Ch347Device::new(config.clone())
        .probe()
        .context("failed to probe CH347 SPI ROM size before reading backup")?;
    let image = Ch347Device::new(Ch347Config {
        expected_size: probe.save_size,
        ..config
    })
    .read_save_image()
    .context("failed to read CH347 SPI ROM image")?;
    if image.len() > MAX_CH347_HELPER_IMAGE_BYTES {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 read image exceeds {} bytes",
            MAX_CH347_HELPER_IMAGE_BYTES
        ))
        .into());
    }

    let rom_image_blake3 = image.blake3_hash().to_string();
    let extracted = extract_physical_backup_from_rom_image(image.as_bytes())?;
    let output_path = write_extracted_backup_output(&request.output_dir, &extracted)?;

    Ok(Ch347HelperReadResult {
        operation: CH347_HELPER_READ_OPERATION.to_owned(),
        device: "ch347".to_owned(),
        helper_process: CH347_HELPER_PROCESS.to_owned(),
        privileged: true,
        flashrom_path: request.flashrom_path.display().to_string(),
        chip: request.chip,
        chip_detection: chip_detection.to_owned(),
        spi_speed: spi_speed_label.to_owned(),
        output_path: output_path.display().to_string(),
        output_kind: extracted.output_kind.to_owned(),
        output_size: extracted.bytes.len(),
        output_blake3: extracted.payload_blake3.clone(),
        save_size: image.len(),
        payload_size: extracted.payload_size,
        rom_image_size: image.len(),
        payload_blake3: extracted.payload_blake3,
        rom_image_blake3,
        storage_format: extracted.storage_format.to_owned(),
        verified: extracted.verified,
        read_count: 1,
        layout_parsed: extracted.layout_parsed,
        backup_bytes_printed: false,
        wallet_secret_touched: false,
        recovery_share_bytes_printed: false,
    })
}

struct PreparedPhysicalBackupImage {
    image: Vec<u8>,
    payload_size: usize,
    payload_blake3: String,
    storage_format: &'static str,
}

struct ExtractedPhysicalBackup {
    bytes: Vec<u8>,
    payload_size: usize,
    payload_blake3: String,
    storage_format: &'static str,
    output_kind: &'static str,
    output_prefix: &'static str,
    output_extension: &'static str,
    layout_parsed: bool,
    verified: bool,
}

fn prepare_physical_backup_rom_image(
    payload: &[u8],
    chip_size: Option<usize>,
) -> Result<PreparedPhysicalBackupImage> {
    let payload_blake3 = encode_hex(blake3::hash(payload).as_bytes());
    let Some(chip_size) = chip_size else {
        return Ok(PreparedPhysicalBackupImage {
            image: payload.to_vec(),
            payload_size: payload.len(),
            payload_blake3,
            storage_format: FULL_CHIP_IMAGE_FORMAT,
        });
    };

    if chip_size == 0 || chip_size > MAX_CH347_HELPER_IMAGE_BYTES {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 probed ROM size must be between 1 and {} bytes, got {chip_size}",
            MAX_CH347_HELPER_IMAGE_BYTES
        ))
        .into());
    }
    if payload.len() == chip_size {
        return Ok(PreparedPhysicalBackupImage {
            image: payload.to_vec(),
            payload_size: payload.len(),
            payload_blake3,
            storage_format: FULL_CHIP_IMAGE_FORMAT,
        });
    }
    let required = PHYSICAL_BACKUP_ROM_HEADER_BYTES
        .checked_add(payload.len())
        .ok_or_else(|| FramkeyError::invalid_data("CH347 backup payload size overflow"))?;
    if required > chip_size {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 backup payload plus FRAMKey header requires {required} bytes, but the probed ROM is {chip_size} bytes"
        ))
        .into());
    }

    let mut image = vec![0xff; chip_size];
    image[..PHYSICAL_BACKUP_ROM_MAGIC.len()].copy_from_slice(PHYSICAL_BACKUP_ROM_MAGIC);
    image[16..20].copy_from_slice(&PHYSICAL_BACKUP_ROM_VERSION.to_le_bytes());
    image[20..28].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    image[28..60].copy_from_slice(blake3::hash(payload).as_bytes());
    image[60..64].copy_from_slice(&(PHYSICAL_BACKUP_ROM_HEADER_BYTES as u32).to_le_bytes());
    image[PHYSICAL_BACKUP_ROM_HEADER_BYTES..PHYSICAL_BACKUP_ROM_HEADER_BYTES + payload.len()]
        .copy_from_slice(payload);

    Ok(PreparedPhysicalBackupImage {
        image,
        payload_size: payload.len(),
        payload_blake3,
        storage_format: PHYSICAL_BACKUP_ROM_FORMAT,
    })
}

fn extract_physical_backup_from_rom_image(image: &[u8]) -> Result<ExtractedPhysicalBackup> {
    if image.len() < PHYSICAL_BACKUP_ROM_MAGIC.len()
        || &image[..PHYSICAL_BACKUP_ROM_MAGIC.len()] != PHYSICAL_BACKUP_ROM_MAGIC
    {
        return extracted_full_chip_image(image);
    }
    if image.len() < 64 {
        return Err(FramkeyError::invalid_data(
            "CH347 ROM has a truncated FRAMKey physical-backup header",
        )
        .into());
    }

    let version = u32::from_le_bytes(image[16..20].try_into().expect("fixed header range"));
    if version != PHYSICAL_BACKUP_ROM_VERSION {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 ROM uses unsupported FRAMKey physical-backup version {version}"
        ))
        .into());
    }
    let payload_len_u64 = u64::from_le_bytes(image[20..28].try_into().expect("fixed header range"));
    let payload_len = usize::try_from(payload_len_u64).map_err(|_| {
        FramkeyError::invalid_data(format!(
            "CH347 ROM physical-backup payload length is too large: {payload_len_u64}"
        ))
    })?;
    let expected_hash = &image[28..60];
    let header_len_u32 = u32::from_le_bytes(image[60..64].try_into().expect("fixed header range"));
    let header_len = usize::try_from(header_len_u32).map_err(|_| {
        FramkeyError::invalid_data(format!(
            "CH347 ROM physical-backup header length is too large: {header_len_u32}"
        ))
    })?;
    if !(64..=image.len()).contains(&header_len) {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 ROM physical-backup header length {header_len} is invalid for {} byte image",
            image.len()
        ))
        .into());
    }
    let payload_end = header_len
        .checked_add(payload_len)
        .ok_or_else(|| FramkeyError::invalid_data("CH347 ROM physical-backup payload overflow"))?;
    if payload_end > image.len() {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 ROM physical-backup payload requires {payload_end} bytes, but image is {} bytes",
            image.len()
        ))
        .into());
    }
    let payload = &image[header_len..payload_end];
    let actual_hash = blake3::hash(payload);
    if actual_hash.as_bytes() != expected_hash {
        return Err(FramkeyError::invalid_data(
            "CH347 ROM physical-backup payload BLAKE3 does not match header",
        )
        .into());
    }
    if image[payload_end..].iter().any(|byte| *byte != 0xff) {
        return Err(FramkeyError::invalid_data(
            "CH347 ROM physical-backup padding is not erased 0xFF bytes",
        )
        .into());
    }

    Ok(ExtractedPhysicalBackup {
        bytes: payload.to_vec(),
        payload_size: payload.len(),
        payload_blake3: encode_hex(actual_hash.as_bytes()),
        storage_format: PHYSICAL_BACKUP_ROM_FORMAT,
        output_kind: "physical_backup_payload",
        output_prefix: "framkey-rom-backup",
        output_extension: "dat",
        layout_parsed: true,
        verified: true,
    })
}

fn extracted_full_chip_image(image: &[u8]) -> Result<ExtractedPhysicalBackup> {
    if image.is_empty() {
        return Err(FramkeyError::invalid_data("CH347 ROM image is empty").into());
    }
    Ok(ExtractedPhysicalBackup {
        bytes: image.to_vec(),
        payload_size: image.len(),
        payload_blake3: encode_hex(blake3::hash(image).as_bytes()),
        storage_format: FULL_CHIP_IMAGE_FORMAT,
        output_kind: "full_chip_image",
        output_prefix: "framkey-rom-image",
        output_extension: "bin",
        layout_parsed: false,
        verified: true,
    })
}

fn write_extracted_backup_output(
    output_dir: &Path,
    extracted: &ExtractedPhysicalBackup,
) -> Result<PathBuf> {
    validate_output_dir(output_dir)?;
    let timestamp = now_unix_ms();
    for index in 0..1000 {
        let suffix = if index == 0 {
            String::new()
        } else {
            format!("-{index}")
        };
        let candidate = output_dir.join(format!(
            "{}-{timestamp}{suffix}.{}",
            extracted.output_prefix, extracted.output_extension
        ));
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "failed to create CH347 ROM backup output {}",
                        candidate.display()
                    )
                });
            }
        };
        file.write_all(&extracted.bytes)?;
        file.flush()?;
        normalize_output_file_owner_and_permissions(&candidate, output_dir)?;
        return Ok(candidate);
    }
    Err(FramkeyError::invalid_data(format!(
        "failed to choose a unique CH347 ROM backup output name in {}",
        output_dir.display()
    ))
    .into())
}

pub fn error_response(error: &anyhow::Error) -> Ch347HelperResponse {
    Ch347HelperResponse::error(Ch347HelperError {
        code: classify_error(error).to_owned(),
        message: format_error_chain(error, 2_400),
    })
}

impl Ch347HelperWriteRequest {
    pub fn validate(&self) -> Result<()> {
        if self.operation != CH347_HELPER_OPERATION {
            return Err(FramkeyError::unsupported(format!(
                "CH347 helper only supports {CH347_HELPER_OPERATION}"
            ))
            .into());
        }
        validate_absolute_path("input path", &self.input_path)?;
        validate_absolute_path("flashrom path", &self.flashrom_path)?;
        if self.expected_size == 0 || self.expected_size > MAX_CH347_HELPER_IMAGE_BYTES {
            return Err(FramkeyError::invalid_data(format!(
                "CH347 helper input file size must be between 1 and {} bytes",
                MAX_CH347_HELPER_IMAGE_BYTES
            ))
            .into());
        }
        validate_blake3_hex(&self.expected_blake3)?;
        if let Some(chip) = &self.chip {
            validate_text("chip override", chip)?;
        }
        parse_spi_speed(self.spispeed.as_deref())?;
        Ok(())
    }
}

impl Ch347HelperReadRequest {
    pub fn validate(&self) -> Result<()> {
        if self.operation != CH347_HELPER_READ_OPERATION {
            return Err(FramkeyError::unsupported(format!(
                "CH347 helper only supports {CH347_HELPER_READ_OPERATION}"
            ))
            .into());
        }
        validate_output_dir(&self.output_dir)?;
        validate_absolute_path("flashrom path", &self.flashrom_path)?;
        if let Some(chip) = &self.chip {
            validate_text("chip override", chip)?;
        }
        parse_spi_speed(self.spispeed.as_deref())?;
        Ok(())
    }
}

pub fn parse_spi_speed(value: Option<&str>) -> Result<Option<Ch347SpiSpeed>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    Ch347SpiSpeed::from_flashrom_value(value)
        .map(Some)
        .ok_or_else(|| {
            FramkeyError::invalid_data(
                "CH347 SPI speed must be one of 60M, 30M, 15M, 7.5M, 3.75M, 1.875M, 937.5K, 468.75K",
            )
            .into()
        })
}

fn read_limited_file(path: &Path, max_bytes: usize) -> Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take((max_bytes + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 helper request exceeds {max_bytes} bytes"
        ))
        .into());
    }
    Ok(bytes)
}

fn validate_absolute_path(label: &str, path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() || !path.is_absolute() {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 helper {label} must be an absolute path"
        ))
        .into());
    }
    let text = path.display().to_string();
    if text.chars().any(char::is_control) {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 helper {label} must not contain control characters"
        ))
        .into());
    }
    Ok(())
}

fn validate_output_dir(path: &Path) -> Result<()> {
    validate_absolute_path("output directory", path)?;
    let metadata = fs::metadata(path).with_context(|| {
        format!(
            "failed to inspect CH347 helper output directory {}",
            path.display()
        )
    })?;
    if !metadata.is_dir() {
        return Err(FramkeyError::invalid_data(format!(
            "CH347 helper output directory is not a directory: {}",
            path.display()
        ))
        .into());
    }
    Ok(())
}

#[cfg(unix)]
fn normalize_output_file_owner_and_permissions(path: &Path, output_dir: &Path) -> Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let dir_metadata = fs::metadata(output_dir)?;
    let mut file_metadata = fs::metadata(path)?;
    if file_metadata.uid() != dir_metadata.uid() || file_metadata.gid() != dir_metadata.gid() {
        let _ = std::os::unix::fs::chown(path, Some(dir_metadata.uid()), Some(dir_metadata.gid()));
        file_metadata = fs::metadata(path)?;
    }
    if file_metadata.uid() == dir_metadata.uid() {
        let mut permissions = file_metadata.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn normalize_output_file_owner_and_permissions(_path: &Path, _output_dir: &Path) -> Result<()> {
    Ok(())
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn validate_text(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() || value.trim() != value || value.chars().any(char::is_control) {
        return Err(
            FramkeyError::invalid_data(format!("CH347 helper {label} is malformed")).into(),
        );
    }
    Ok(())
}

fn validate_blake3_hex(value: &str) -> Result<()> {
    if value.len() != 64 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(FramkeyError::invalid_data(
            "CH347 helper expected BLAKE3 must be 64 hex characters",
        )
        .into());
    }
    Ok(())
}

fn classify_error(error: &anyhow::Error) -> &'static str {
    let message = error.to_string();
    if message.contains("invalid data") {
        "INVALID_DATA"
    } else if message.contains("unsupported operation") || message.contains("flashrom") {
        "UNSUPPORTED_METHOD"
    } else {
        "IO_ERROR"
    }
}

fn format_error_chain(error: &anyhow::Error, max_chars: usize) -> String {
    let mut parts = Vec::new();
    for cause in error.chain() {
        let message = cause.to_string();
        if !parts.iter().any(|part| part == &message) {
            parts.push(message);
        }
    }
    let mut message = parts.join("; caused by: ");
    if message.chars().count() > max_chars {
        message = message.chars().take(max_chars).collect::<String>();
        message.push_str("...");
    }
    message
}

#[cfg(test)]
mod tests;
