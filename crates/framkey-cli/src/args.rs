use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use framkey_ch347::Ch347SpiSpeed;
use framkey_gbxcart::GbaSaveType;
use framkey_keychain_macos::MacKeychainItem;
use framkey_vault::DEFAULT_FRAM_SAVE_IMAGE_SIZE;

use crate::constants::{DEFAULT_KEYCHAIN_ACCOUNT, DEFAULT_KEYCHAIN_SERVICE};

#[derive(Debug, Parser)]
#[command(name = "framkey")]
#[command(about = "FRAMKey development CLI")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Device {
        #[command(subcommand)]
        command: DeviceCommand,
    },
    Recovery {
        #[command(subcommand)]
        command: RecoveryCommand,
    },
    Signer {
        #[command(subcommand)]
        command: SignerCommand,
    },
    Vault {
        #[command(subcommand)]
        command: VaultCommand,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum DeviceCommand {
    Probe(DeviceTargetArgs),
    ReadSave(ReadSaveArgs),
    WriteSave(WriteSaveArgs),
    VerifySave(VerifySaveArgs),
}

#[derive(Debug, Subcommand)]
pub(crate) enum RecoveryCommand {
    Policy,
}

#[derive(Debug, Subcommand)]
pub(crate) enum SignerCommand {
    PersonalSign(SignerPersonalSignArgs),
}

#[derive(Debug, Subcommand)]
pub(crate) enum VaultCommand {
    BuildTestImage(BuildTestImageArgs),
    InitKeychainKek(KeychainKekArgs),
    RebindKeychainKek(KeychainKekArgs),
    TrustKeychainHelperAccess(TrustKeychainHelperAccessArgs),
    BuildKeychainEncryptedImage(BuildKeychainEncryptedImageArgs),
    RecoverKeychainEncryptedImage(RecoverKeychainEncryptedImageArgs),
    OpenKeychainEncryptedImage(OpenKeychainEncryptedImageArgs),
    GenerateDevKek,
    BuildDevEncryptedImage(BuildDevEncryptedImageArgs),
    OpenDevEncryptedImage(OpenDevEncryptedImageArgs),
    InspectImage(InspectImageArgs),
}

#[derive(Debug, Args)]
pub(crate) struct BuildTestImageArgs {
    #[arg(long)]
    pub(crate) out: PathBuf,

    #[arg(long, default_value_t = DEFAULT_FRAM_SAVE_IMAGE_SIZE)]
    pub(crate) image_size: usize,

    #[arg(long, default_value_t = 1)]
    pub(crate) generation: u64,

    #[arg(long, default_value = "FRAMKey hardware smoke test")]
    pub(crate) label: String,
}

#[derive(Debug, Args)]
pub(crate) struct BuildDevEncryptedImageArgs {
    #[arg(long)]
    pub(crate) out: PathBuf,

    #[arg(long, default_value_t = DEFAULT_FRAM_SAVE_IMAGE_SIZE)]
    pub(crate) image_size: usize,

    #[arg(long, default_value_t = 1)]
    pub(crate) generation: u64,

    #[arg(long, default_value = "FRAMKey dev encrypted vault")]
    pub(crate) label: String,

    #[arg(long)]
    pub(crate) dev_kek_hex: Option<String>,
}

#[derive(Debug, Args)]
pub(crate) struct KeychainKekArgs {
    #[command(flatten)]
    pub(crate) keychain: KeychainItemArgs,
}

#[derive(Debug, Args)]
pub(crate) struct TrustKeychainHelperAccessArgs {
    #[command(flatten)]
    pub(crate) keychain: KeychainItemArgs,

    #[command(flatten)]
    pub(crate) helper: SignerHelperArgs,
}

#[derive(Debug, Args)]
pub(crate) struct BuildKeychainEncryptedImageArgs {
    #[arg(long)]
    pub(crate) out: PathBuf,

    #[arg(long)]
    pub(crate) recovery_out_dir: Option<PathBuf>,

    #[arg(long, default_value_t = DEFAULT_FRAM_SAVE_IMAGE_SIZE)]
    pub(crate) image_size: usize,

    #[arg(long, default_value_t = 1)]
    pub(crate) generation: u64,

    #[command(flatten)]
    pub(crate) keychain: KeychainItemArgs,

    #[command(flatten)]
    pub(crate) helper: SignerHelperArgs,
}

#[derive(Debug, Args)]
pub(crate) struct RecoverKeychainEncryptedImageArgs {
    #[arg(long)]
    pub(crate) path: PathBuf,

    #[arg(long)]
    pub(crate) out: PathBuf,

    #[arg(long = "recovery-file", required = true)]
    pub(crate) recovery_files: Vec<PathBuf>,

    #[command(flatten)]
    pub(crate) keychain: KeychainItemArgs,

    #[command(flatten)]
    pub(crate) helper: SignerHelperArgs,
}

#[derive(Debug, Args)]
pub(crate) struct OpenKeychainEncryptedImageArgs {
    #[arg(long)]
    pub(crate) path: PathBuf,

    #[command(flatten)]
    pub(crate) keychain: KeychainItemArgs,

    #[command(flatten)]
    pub(crate) helper: SignerHelperArgs,
}

#[derive(Debug, Args)]
pub(crate) struct SignerPersonalSignArgs {
    #[command(flatten)]
    pub(crate) target: DeviceTargetArgs,

    #[arg(long)]
    pub(crate) message: String,

    #[command(flatten)]
    pub(crate) keychain: KeychainItemArgs,

    #[command(flatten)]
    pub(crate) helper: SignerHelperArgs,
}

#[derive(Debug, Args)]
pub(crate) struct SignerHelperArgs {
    #[arg(long = "signer-helper")]
    pub(crate) signer_helper: Option<PathBuf>,

    #[arg(long = "signer-helper-blake3")]
    pub(crate) signer_helper_blake3: Option<String>,

    #[arg(long = "allow-unsandboxed-signer-helper")]
    pub(crate) allow_unsandboxed_signer_helper: bool,

    #[arg(long = "use-sandbox-exec-signer-helper", hide = true)]
    pub(crate) use_sandbox_exec_signer_helper: bool,
}

#[derive(Debug, Args)]
pub(crate) struct KeychainItemArgs {
    #[arg(long = "keychain-service", default_value = DEFAULT_KEYCHAIN_SERVICE)]
    pub(crate) service: String,

    #[arg(long = "keychain-account", default_value = DEFAULT_KEYCHAIN_ACCOUNT)]
    pub(crate) account: String,
}

impl KeychainItemArgs {
    pub(crate) fn item(&self) -> MacKeychainItem {
        MacKeychainItem::new(self.service.clone(), self.account.clone())
    }
}

#[derive(Debug, Args)]
pub(crate) struct OpenDevEncryptedImageArgs {
    #[arg(long)]
    pub(crate) path: PathBuf,

    #[arg(long)]
    pub(crate) dev_kek_hex: Option<String>,
}

#[derive(Debug, Args)]
pub(crate) struct InspectImageArgs {
    #[arg(long)]
    pub(crate) path: PathBuf,
}

#[derive(Debug, Args)]
pub(crate) struct DeviceTargetArgs {
    #[arg(long, value_enum, default_value_t = DeviceTargetKind::File)]
    pub(crate) device: DeviceTargetKind,

    #[arg(long)]
    pub(crate) path: Option<PathBuf>,

    #[arg(long)]
    pub(crate) port: Option<String>,

    #[arg(long)]
    pub(crate) expected_save_size: Option<usize>,

    #[arg(long, value_enum)]
    pub(crate) save_type: Option<GbxCartSaveTypeArg>,

    #[arg(long)]
    pub(crate) chip: Option<String>,

    #[arg(long, value_enum)]
    pub(crate) spispeed: Option<Ch347SpiSpeedArg>,

    #[arg(long)]
    pub(crate) flashrom: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub(crate) struct ReadSaveArgs {
    #[command(flatten)]
    pub(crate) target: DeviceTargetArgs,

    #[arg(long)]
    pub(crate) out: PathBuf,
}

#[derive(Debug, Args)]
pub(crate) struct WriteSaveArgs {
    #[command(flatten)]
    pub(crate) target: DeviceTargetArgs,

    #[arg(long)]
    pub(crate) input: PathBuf,
}

#[derive(Debug, Args)]
pub(crate) struct VerifySaveArgs {
    #[command(flatten)]
    pub(crate) target: DeviceTargetArgs,

    #[arg(long)]
    pub(crate) blake3: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum DeviceTargetKind {
    Ch347,
    File,
    GbxCart,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Ch347SpiSpeedArg {
    #[value(name = "60M", alias = "60m")]
    M60,

    #[value(name = "30M", alias = "30m")]
    M30,

    #[value(name = "15M", alias = "15m")]
    M15,

    #[value(name = "7.5M", alias = "7.5m")]
    M7_5,

    #[value(name = "3.75M", alias = "3.75m")]
    M3_75,

    #[value(name = "1.875M", alias = "1.875m")]
    M1_875,

    #[value(name = "937.5K", alias = "937.5k")]
    K937_5,

    #[value(name = "468.75K", alias = "468.75k")]
    K468_75,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum GbxCartSaveTypeArg {
    #[value(name = "gba-eeprom-64k", alias = "gba-eeprom64k")]
    GbaEeprom64k,

    #[value(
        name = "gba-sram-fram-256k",
        alias = "gba-sram-256k",
        alias = "gba-fram-256k"
    )]
    GbaSramFram256k,

    #[value(
        name = "gba-sram-fram-512kbit",
        alias = "gba-sram-fram-512k",
        alias = "gba-sram-fram-64kib",
        alias = "gba-sram-512k",
        alias = "gba-fram-512k"
    )]
    GbaSramFram512Kbit,

    #[value(
        name = "gba-sram-fram-1mbit",
        alias = "gba-sram-fram-1m",
        alias = "gba-sram-fram-128k",
        alias = "gba-sram-1m",
        alias = "gba-fram-1m"
    )]
    GbaSramFram1Mbit,
}

impl From<GbxCartSaveTypeArg> for GbaSaveType {
    fn from(value: GbxCartSaveTypeArg) -> Self {
        match value {
            GbxCartSaveTypeArg::GbaEeprom64k => Self::Eeprom64k,
            GbxCartSaveTypeArg::GbaSramFram256k => Self::SramFram256k,
            GbxCartSaveTypeArg::GbaSramFram512Kbit => Self::SramFram512Kbit,
            GbxCartSaveTypeArg::GbaSramFram1Mbit => Self::SramFram1Mbit,
        }
    }
}

impl From<Ch347SpiSpeedArg> for Ch347SpiSpeed {
    fn from(value: Ch347SpiSpeedArg) -> Self {
        match value {
            Ch347SpiSpeedArg::M60 => Self::M60,
            Ch347SpiSpeedArg::M30 => Self::M30,
            Ch347SpiSpeedArg::M15 => Self::M15,
            Ch347SpiSpeedArg::M7_5 => Self::M7_5,
            Ch347SpiSpeedArg::M3_75 => Self::M3_75,
            Ch347SpiSpeedArg::M1_875 => Self::M1_875,
            Ch347SpiSpeedArg::K937_5 => Self::K937_5,
            Ch347SpiSpeedArg::K468_75 => Self::K468_75,
        }
    }
}
