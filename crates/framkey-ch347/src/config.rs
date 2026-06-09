use std::{fmt, path::PathBuf};

use framkey_core::{FramkeyError, Result};

pub(crate) const DEFAULT_FLASHROM_PROGRAMMER: &str = "ch347_spi";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ch347Config {
    pub chip: Option<String>,
    pub flashrom_path: Option<PathBuf>,
    pub spi_speed: Option<Ch347SpiSpeed>,
    pub expected_size: Option<usize>,
}

impl Ch347Config {
    pub(crate) fn validate(&self) -> Result<()> {
        if let Some(chip) = &self.chip {
            validated_chip_name(chip)?;
        }
        Ok(())
    }

    pub(crate) fn chip_name(&self) -> Result<Option<&str>> {
        self.chip.as_deref().map(validated_chip_name).transpose()
    }

    pub(crate) fn flashrom_program(&self) -> PathBuf {
        self.flashrom_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("flashrom"))
    }

    pub(crate) fn programmer_arg(&self) -> String {
        match self.spi_speed {
            Some(speed) => format!(
                "{}:spispeed={}",
                DEFAULT_FLASHROM_PROGRAMMER,
                speed.as_flashrom_value()
            ),
            None => format!(
                "{}:spispeed={}",
                DEFAULT_FLASHROM_PROGRAMMER,
                Ch347SpiSpeed::M15.as_flashrom_value()
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ch347SpiSpeed {
    M60,
    M30,
    M15,
    M7_5,
    M3_75,
    M1_875,
    K937_5,
    K468_75,
}

impl Ch347SpiSpeed {
    pub fn as_flashrom_value(self) -> &'static str {
        match self {
            Self::M60 => "60M",
            Self::M30 => "30M",
            Self::M15 => "15M",
            Self::M7_5 => "7.5M",
            Self::M3_75 => "3.75M",
            Self::M1_875 => "1.875M",
            Self::K937_5 => "937.5K",
            Self::K468_75 => "468.75K",
        }
    }

    pub fn from_flashrom_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "60m" => Some(Self::M60),
            "30m" => Some(Self::M30),
            "15m" => Some(Self::M15),
            "7.5m" => Some(Self::M7_5),
            "3.75m" => Some(Self::M3_75),
            "1.875m" => Some(Self::M1_875),
            "937.5k" => Some(Self::K937_5),
            "468.75k" => Some(Self::K468_75),
            _ => None,
        }
    }
}

impl fmt::Display for Ch347SpiSpeed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_flashrom_value())
    }
}

fn validated_chip_name(input: &str) -> Result<&str> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(FramkeyError::invalid_data(
            "CH347 flashrom chip name must not be empty",
        ));
    }

    if trimmed.len() != input.len() {
        return Err(FramkeyError::invalid_data(
            "CH347 flashrom chip name must not contain leading or trailing whitespace",
        ));
    }

    if trimmed.chars().any(char::is_control) {
        return Err(FramkeyError::invalid_data(
            "CH347 flashrom chip name must not contain control characters",
        ));
    }

    Ok(trimmed)
}
