mod args;
mod constants;
mod device;
mod recovery;
mod signer;
mod signer_helper;
mod vault;

use anyhow::Result;
use clap::Parser;

use crate::{
    args::{Cli, Commands},
    device::run_device,
    recovery::run_recovery,
    signer::run_signer,
    vault::run_vault,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Device { command } => run_device(command)?,
        Commands::Recovery { command } => run_recovery(command)?,
        Commands::Signer { command } => run_signer(command)?,
        Commands::Vault { command } => run_vault(command)?,
    }

    Ok(())
}
