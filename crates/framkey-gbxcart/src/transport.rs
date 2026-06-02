use std::{
    fmt,
    io::{Read, Write},
    thread,
    time::Duration,
};

use framkey_core::{FramkeyError, Result};
use framkey_device::{DeviceInfo, DeviceKind, SaveImage, VaultDevice};
use serialport::{ClearBuffer, SerialPort, SerialPortType};

use crate::{
    constants::*,
    types::{FirmwareInfo, GbaHeader, GbaSaveType, GbxCartConfig},
};

#[derive(Debug, Clone)]
pub struct GbxCartDevice {
    config: GbxCartConfig,
}

impl GbxCartDevice {
    pub fn new(config: GbxCartConfig) -> Self {
        Self { config }
    }

    fn connect(&self) -> Result<GbxCartSession> {
        let ports = candidate_ports(self.config.port_hint.as_deref())?;
        if ports.is_empty() {
            return Err(FramkeyError::unsupported(
                "no GBxCart serial ports found; pass --port if the adapter is not exposed as USB serial",
            ));
        }

        let mut last_error = None;
        for port_name in ports {
            match GbxCartSession::open(&port_name) {
                Ok(session) => return Ok(session),
                Err(error) => last_error = Some(error),
            }
        }

        Err(last_error.unwrap_or_else(|| {
            FramkeyError::unsupported("no usable GBxCart serial device was found")
        }))
    }

    fn selected_save_type(&self) -> Result<GbaSaveType> {
        let save_type = self.config.save_type.ok_or_else(|| {
            FramkeyError::unsupported(
                "GBxCart read/write requires an explicit --save-type; auto detection is not implemented yet",
            )
        })?;

        if let Some(expected) = self.config.expected_save_size
            && expected != save_type.save_size()
        {
            return Err(FramkeyError::invalid_data(format!(
                "expected save size {expected} does not match {} ({})",
                save_type.label(),
                save_type.save_size()
            )));
        }

        Ok(save_type)
    }
}

impl VaultDevice for GbxCartDevice {
    fn probe(&self) -> Result<DeviceInfo> {
        let mut session = self.connect()?;
        let header = session.read_gba_header().ok();
        let save_size = self
            .config
            .save_type
            .map(GbaSaveType::save_size)
            .or(self.config.expected_save_size);

        let label = match header {
            Some(header) => format!(
                "{}; GBA {} {} rev {}{}",
                session.label(),
                header.game_code,
                header.title,
                header.revision,
                if header.header_checksum_valid {
                    ""
                } else {
                    " (bad header checksum)"
                }
            ),
            None => session.label(),
        };

        session.cleanup_best_effort();

        Ok(DeviceInfo {
            kind: DeviceKind::GbxCart,
            label,
            save_size,
        })
    }

    fn read_save_image(&self) -> Result<SaveImage> {
        let save_type = self.selected_save_type()?;
        let mut session = self.connect()?;
        let result = session.read_save_image(save_type).map(SaveImage::new);
        session.cleanup_best_effort();
        result
    }

    fn write_save_image(&mut self, image: &SaveImage) -> Result<()> {
        let save_type = self.selected_save_type()?;
        if image.len() != save_type.save_size() {
            return Err(FramkeyError::invalid_data(format!(
                "{} requires {} bytes, got {}",
                save_type.label(),
                save_type.save_size(),
                image.len()
            )));
        }

        let mut session = self.connect()?;
        let result = (|| {
            session.write_save_image(save_type, image.as_bytes())?;
            let readback = session.read_save_image(save_type)?;

            if readback != image.as_bytes() {
                return Err(FramkeyError::invalid_data(
                    "GBxCart write completed but readback verification failed",
                ));
            }

            Ok(())
        })();
        session.cleanup_best_effort();
        result
    }
}

struct GbxCartSession {
    port_name: String,
    port: Box<dyn SerialPort>,
    firmware: FirmwareInfo,
}

impl GbxCartSession {
    fn open(port_name: &str) -> Result<Self> {
        let mut port = serialport::new(port_name, DEFAULT_BAUD)
            .timeout(SERIAL_TIMEOUT)
            .open()
            .map_err(|error| serial_error(format!("failed to open {port_name}"), error))?;
        clear_port(port.as_mut())?;

        let mut session = Self {
            port_name: port_name.to_owned(),
            port,
            firmware: FirmwareInfo {
                cfw_id: '\0',
                fw_version: 0,
                pcb_version: 0,
                firmware_timestamp: 0,
                official_fw_version: 0,
            },
        };

        session.firmware = session.query_firmware()?;
        session.validate_firmware()?;
        Ok(session)
    }

    fn label(&self) -> String {
        format!(
            "GBxCart RW {} firmware {} on {}",
            self.firmware.pcb_label(),
            self.firmware.firmware_label(),
            self.port_name
        )
    }

    fn validate_firmware(&self) -> Result<()> {
        if self.firmware.cfw_id != 'L' {
            return Err(FramkeyError::unsupported(format!(
                "unsupported GBxCart firmware id {}; Lesserkuma firmware is required",
                self.firmware.cfw_id
            )));
        }

        if self.firmware.fw_version == 0 {
            return Err(FramkeyError::unsupported(
                "official GBxCart firmware is not supported by the native transport yet",
            ));
        }

        if self.firmware.pcb_version < 5 && self.firmware.fw_version != 1 {
            return Err(FramkeyError::unsupported(format!(
                "GBxCart RW {} reported firmware {}; this first transport supports v1.3/L1-class behavior",
                self.firmware.pcb_label(),
                self.firmware.firmware_label()
            )));
        }

        Ok(())
    }

    fn query_firmware(&mut self) -> Result<FirmwareInfo> {
        self.write_byte(CMD_OFW_PCB_VER)?;
        let pcb_version = self.read_byte()?;

        self.write_byte(CMD_OFW_FW_VER)?;
        let official_fw_version = self.read_byte()?;
        if pcb_version < 5 && official_fw_version > 0 {
            return Err(FramkeyError::unsupported(
                "legacy official GBxCart firmware is not supported by the native transport yet",
            ));
        }

        self.write_byte(CMD_QUERY_FW_INFO)?;
        let size = self.read_byte()? as usize;
        if size != 8 {
            return Err(FramkeyError::invalid_data(format!(
                "unexpected firmware info size {size}; expected 8"
            )));
        }

        let info = self.read_exact_vec(8)?;
        let cfw_id = char::from(info[0]);
        let fw_version = u16::from_be_bytes([info[1], info[2]]);
        let pcb_version_from_fw = info[3];
        let firmware_timestamp = u32::from_be_bytes([info[4], info[5], info[6], info[7]]);

        if pcb_version_from_fw != pcb_version {
            return Err(FramkeyError::invalid_data(format!(
                "GBxCart PCB version mismatch: command reported {pcb_version}, firmware info reported {pcb_version_from_fw}"
            )));
        }

        Ok(FirmwareInfo {
            cfw_id,
            fw_version,
            pcb_version,
            firmware_timestamp,
            official_fw_version,
        })
    }

    fn read_gba_header(&mut self) -> Result<GbaHeader> {
        self.enter_agb_mode()?;
        let header = self.read_rom(0, GBA_HEADER_SIZE, EEPROM_TRANSFER_CHUNK)?;
        GbaHeader::parse(&header)
    }

    fn read_save_image(&mut self, save_type: GbaSaveType) -> Result<Vec<u8>> {
        self.enter_agb_mode()?;
        match save_type {
            GbaSaveType::Eeprom64k => self.read_eeprom64k(),
            GbaSaveType::SramFram256k => self.read_sram_fram_256k(),
            GbaSaveType::SramFram512Kbit => self.read_sram_fram_512kbit(),
            GbaSaveType::SramFram1Mbit => self.read_sram_fram_1mbit(),
        }
    }

    fn write_save_image(&mut self, save_type: GbaSaveType, bytes: &[u8]) -> Result<()> {
        self.enter_agb_mode()?;
        match save_type {
            GbaSaveType::Eeprom64k => self.write_eeprom64k(bytes),
            GbaSaveType::SramFram256k => self.write_sram_fram_256k(bytes),
            GbaSaveType::SramFram512Kbit => self.write_sram_fram_512kbit(bytes),
            GbaSaveType::SramFram1Mbit => self.write_sram_fram_1mbit(bytes),
        }
    }

    fn enter_agb_mode(&mut self) -> Result<()> {
        self.write_byte(CMD_SET_MODE_AGB)?;
        self.write_byte(CMD_SET_VOLTAGE_3_3V)?;
        self.set_variable_u8(VAR_AGB_READ_METHOD, 0)?;
        self.set_variable_u8(VAR_CART_MODE, CART_MODE_AGB)?;
        self.set_variable_u32(VAR_ADDRESS, 0)?;

        if self.firmware.fw_version >= 8 {
            self.write_ack(&[CMD_DISABLE_PULLUPS])?;
        }

        self.cart_power_on()?;
        Ok(())
    }

    fn cart_power_on(&mut self) -> Result<()> {
        if !matches!(self.firmware.pcb_version, 5 | 6 | 101) {
            return Ok(());
        }

        self.write_byte(CMD_OFW_QUERY_CART_PWR)?;
        if self.read_byte()? == 0 {
            self.write_byte(CMD_OFW_CART_PWR_ON)?;
            thread::sleep(Duration::from_millis(100));
            clear_port(self.port.as_mut())?;
        }

        Ok(())
    }

    fn read_eeprom64k(&mut self) -> Result<Vec<u8>> {
        let mut image = Vec::with_capacity(EEPROM64K_SIZE);

        for offset in (0..EEPROM64K_SIZE).step_by(EEPROM64K_READ_BLOCK) {
            let address = eeprom_address(offset)?;
            let chunk = self.read_ram_with_command(
                address,
                EEPROM64K_READ_BLOCK,
                &[CMD_AGB_CART_READ_EEPROM, 2],
                EEPROM_TRANSFER_CHUNK,
            )?;
            image.extend_from_slice(&chunk);
        }

        Ok(image)
    }

    fn write_eeprom64k(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() != EEPROM64K_SIZE {
            return Err(FramkeyError::invalid_data(format!(
                "GBA EEPROM64K write requires {EEPROM64K_SIZE} bytes, got {}",
                bytes.len()
            )));
        }

        for (offset, chunk) in bytes.chunks(EEPROM64K_WRITE_BLOCK).enumerate() {
            let byte_offset = offset * EEPROM64K_WRITE_BLOCK;
            let address = eeprom_address(byte_offset)?;
            self.write_ram_with_command(address, chunk, &[CMD_AGB_CART_WRITE_EEPROM, 2])?;
        }

        Ok(())
    }

    fn read_sram_fram_256k(&mut self) -> Result<Vec<u8>> {
        self.read_sram_fram_linear(SRAM_FRAM_256K_SIZE)
    }

    fn write_sram_fram_256k(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() != SRAM_FRAM_256K_SIZE {
            return Err(FramkeyError::invalid_data(format!(
                "GBA SRAM/FRAM 256K write requires {SRAM_FRAM_256K_SIZE} bytes, got {}",
                bytes.len()
            )));
        }

        self.write_sram_fram_linear(bytes)
    }

    fn read_sram_fram_512kbit(&mut self) -> Result<Vec<u8>> {
        self.read_sram_fram_linear(SRAM_FRAM_512KBIT_SIZE)
    }

    fn write_sram_fram_512kbit(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() != SRAM_FRAM_512KBIT_SIZE {
            return Err(FramkeyError::invalid_data(format!(
                "GBA SRAM/FRAM 512 Kbit write requires {SRAM_FRAM_512KBIT_SIZE} bytes, got {}",
                bytes.len()
            )));
        }

        self.write_sram_fram_linear(bytes)
    }

    fn read_sram_fram_1mbit(&mut self) -> Result<Vec<u8>> {
        let mut image = Vec::with_capacity(SRAM_FRAM_1MBIT_SIZE);

        for bank in 0..(SRAM_FRAM_1MBIT_SIZE / SRAM_FRAM_BANK_SIZE) {
            self.select_sram_fram_bank(bank as u16)?;
            let bank_image = self.read_sram_fram_linear(SRAM_FRAM_BANK_SIZE)?;
            image.extend_from_slice(&bank_image);
        }

        Ok(image)
    }

    fn write_sram_fram_1mbit(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() != SRAM_FRAM_1MBIT_SIZE {
            return Err(FramkeyError::invalid_data(format!(
                "GBA SRAM/FRAM 1 Mbit write requires {SRAM_FRAM_1MBIT_SIZE} bytes, got {}",
                bytes.len()
            )));
        }

        if !sram_fram_1mbit_banks_match(bytes) {
            let current = self.read_sram_fram_1mbit()?;
            if sram_fram_1mbit_banks_match(&current) {
                return Err(FramkeyError::invalid_data(
                    "refusing non-mirrored 1 Mbit SRAM/FRAM write: current bank reads are mirrored, so bank switching is not proven for this cartridge",
                ));
            }
        }

        for (bank, chunk) in bytes.chunks(SRAM_FRAM_BANK_SIZE).enumerate() {
            self.select_sram_fram_bank(bank as u16)?;
            self.write_sram_fram_linear(chunk)?;
        }

        Ok(())
    }

    fn read_sram_fram_linear(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut image = Vec::with_capacity(size);

        for offset in (0..size).step_by(SRAM_FRAM_READ_BLOCK) {
            let chunk = self.read_ram_with_command(
                offset as u32,
                SRAM_FRAM_READ_BLOCK,
                &[CMD_AGB_CART_READ_SRAM],
                SRAM_FRAM_READ_BLOCK,
            )?;
            image.extend_from_slice(&chunk);
        }

        Ok(image)
    }

    fn write_sram_fram_linear(&mut self, bytes: &[u8]) -> Result<()> {
        for (index, chunk) in bytes.chunks(SRAM_FRAM_WRITE_BLOCK).enumerate() {
            let offset = index * SRAM_FRAM_WRITE_BLOCK;
            self.write_ram_with_command(offset as u32, chunk, &[CMD_AGB_CART_WRITE_SRAM])?;
        }

        Ok(())
    }

    fn select_sram_fram_bank(&mut self, bank: u16) -> Result<()> {
        self.cart_write_agb(SRAM_FRAM_1MBIT_BANK_SELECT_ADDR, bank)?;
        thread::sleep(Duration::from_millis(50));
        Ok(())
    }

    fn cart_write_agb(&mut self, address: u32, value: u16) -> Result<()> {
        let mut command = Vec::with_capacity(7);
        command.push(CMD_AGB_CART_WRITE);
        command.extend_from_slice(&(address / 2).to_be_bytes());
        command.extend_from_slice(&value.to_be_bytes());
        self.write_bytes(&command)
    }

    fn read_rom(&mut self, address: u32, length: usize, max_chunk: usize) -> Result<Vec<u8>> {
        let mut remaining = length;
        let mut output = Vec::with_capacity(length);
        let mut transfer_size = remaining.min(max_chunk);

        self.set_variable_u16(VAR_TRANSFER_SIZE, transfer_size)?;
        self.set_variable_u32(VAR_ADDRESS, address / 2)?;

        while remaining > 0 {
            let chunk_size = remaining.min(max_chunk);
            if chunk_size != transfer_size {
                self.set_variable_u16(VAR_TRANSFER_SIZE, chunk_size)?;
                transfer_size = chunk_size;
            }

            self.write_byte(CMD_AGB_CART_READ)?;
            let chunk = self.read_exact_vec(chunk_size)?;
            output.extend_from_slice(&chunk);
            remaining -= chunk_size;
        }

        Ok(output)
    }

    fn read_ram_with_command(
        &mut self,
        address: u32,
        length: usize,
        command: &[u8],
        max_chunk: usize,
    ) -> Result<Vec<u8>> {
        let mut remaining = length;
        let mut output = Vec::with_capacity(length);
        let mut transfer_size = remaining.min(max_chunk);

        self.set_variable_u16(VAR_TRANSFER_SIZE, transfer_size)?;
        self.set_variable_u32(VAR_ADDRESS, address)?;

        while remaining > 0 {
            let chunk_size = remaining.min(max_chunk);
            if chunk_size != transfer_size {
                self.set_variable_u16(VAR_TRANSFER_SIZE, chunk_size)?;
                transfer_size = chunk_size;
            }

            self.write_bytes(command)?;
            let chunk = self.read_exact_vec(chunk_size)?;
            output.extend_from_slice(&chunk);
            remaining -= chunk_size;
        }

        Ok(output)
    }

    fn write_ram_with_command(&mut self, address: u32, bytes: &[u8], command: &[u8]) -> Result<()> {
        self.set_variable_u16(VAR_TRANSFER_SIZE, bytes.len())?;
        self.set_variable_u32(VAR_ADDRESS, address)?;
        self.write_bytes(command)?;
        self.write_bytes(bytes)?;

        let ack = self.read_byte()?;
        if !matches!(ack, 0x01 | 0x03) {
            return Err(FramkeyError::invalid_data(format!(
                "GBxCart write returned unexpected ACK 0x{ack:02x}"
            )));
        }

        Ok(())
    }

    fn set_variable_u8(&mut self, key: u32, value: u32) -> Result<()> {
        self.set_variable(1, key, value)
    }

    fn set_variable_u16(&mut self, key: u32, value: usize) -> Result<()> {
        if value > u16::MAX as usize {
            return Err(FramkeyError::invalid_data(format!(
                "firmware variable value {value} does not fit in u16"
            )));
        }
        self.set_variable(2, key, value as u32)
    }

    fn set_variable_u32(&mut self, key: u32, value: u32) -> Result<()> {
        self.set_variable(4, key, value)
    }

    fn set_variable(&mut self, size: u8, key: u32, value: u32) -> Result<()> {
        let mut command = Vec::with_capacity(10);
        command.push(CMD_SET_VARIABLE);
        command.push(size);
        command.extend_from_slice(&key.to_be_bytes());
        command.extend_from_slice(&value.to_be_bytes());
        self.write_bytes(&command)
    }

    fn write_ack(&mut self, bytes: &[u8]) -> Result<()> {
        self.write_bytes(bytes)?;
        let ack = self.read_byte()?;
        if !matches!(ack, 0x01 | 0x03) {
            return Err(FramkeyError::invalid_data(format!(
                "GBxCart command returned unexpected ACK 0x{ack:02x}"
            )));
        }
        Ok(())
    }

    fn write_byte(&mut self, byte: u8) -> Result<()> {
        self.write_bytes(&[byte])
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.port.write_all(bytes)?;
        self.port.flush()?;
        thread::sleep(MACOS_WRITE_DELAY);
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8> {
        let mut byte = [0_u8; 1];
        self.port.read_exact(&mut byte)?;
        Ok(byte[0])
    }

    fn read_exact_vec(&mut self, length: usize) -> Result<Vec<u8>> {
        let mut bytes = vec![0_u8; length];
        self.port.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    fn cleanup_best_effort(&mut self) {
        let _ = self.write_byte(CMD_SET_ADDR_AS_INPUTS);
        let _ = clear_port(self.port.as_mut());
    }
}

fn eeprom_address(byte_offset: usize) -> Result<u32> {
    u32::try_from(byte_offset / 8)
        .map_err(|_| FramkeyError::invalid_data("EEPROM byte offset does not fit in u32"))
}

pub(crate) fn candidate_ports(port_hint: Option<&str>) -> Result<Vec<String>> {
    if let Some(port_hint) = port_hint {
        if port_hint.trim().is_empty() {
            return Err(FramkeyError::invalid_data(
                "GBxCart port hint must not be empty",
            ));
        }
        return Ok(vec![port_hint.to_owned()]);
    }

    let mut ports: Vec<String> = serialport::available_ports()
        .map_err(|error| serial_error("failed to enumerate serial ports", error))?
        .into_iter()
        .filter_map(|port| match port.port_type {
            SerialPortType::UsbPort(usb) if usb.vid == GBXCART_VID && usb.pid == GBXCART_PID => {
                Some(port.port_name)
            }
            _ => None,
        })
        .collect();

    ports.sort_by(|left, right| port_preference(left).cmp(&port_preference(right)));
    Ok(ports)
}

fn port_preference(name: &str) -> (u8, &str) {
    let prefix_rank = if name.contains("/dev/cu.") { 0 } else { 1 };
    (prefix_rank, name)
}

fn clear_port(port: &mut dyn SerialPort) -> Result<()> {
    port.clear(ClearBuffer::All)
        .map_err(|error| serial_error("failed to clear serial buffers", error))
}

fn serial_error(context: impl fmt::Display, error: serialport::Error) -> FramkeyError {
    FramkeyError::Io(std::io::Error::other(format!("{context}: {error}")))
}

pub(crate) fn sram_fram_1mbit_banks_match(bytes: &[u8]) -> bool {
    bytes.len() == SRAM_FRAM_1MBIT_SIZE
        && bytes[..SRAM_FRAM_BANK_SIZE] == bytes[SRAM_FRAM_BANK_SIZE..]
}
