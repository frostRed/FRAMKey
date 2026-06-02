use anyhow::Result;
use framkey_evm::{decode_signature_hex, recover_personal_signer};
use serde_json::json;

use crate::{
    args::SignerCommand,
    device::open_device,
    signer_helper::{helper_personal_sign, signer_helper_report},
};

pub(crate) fn run_signer(command: SignerCommand) -> Result<()> {
    match command {
        SignerCommand::PersonalSign(args) => {
            let device = open_device(&args.target)?;
            let image = device.read_save_image()?;
            let (response, helper_execution) = helper_personal_sign(
                &args.helper,
                &args.keychain,
                image.as_bytes().to_vec(),
                args.message.as_bytes().to_vec(),
            )?;
            let signature = decode_signature_hex(&response.signature)?;
            let recovered = recover_personal_signer(args.message.as_bytes(), &signature)?;
            let recovered_matches = recovered
                .to_string()
                .eq_ignore_ascii_case(&response.address);

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": "signer_personal_sign",
                    "message_len": args.message.len(),
                    "address": response.address,
                    "message_hash": response.message_hash,
                    "signature": response.signature,
                    "recovered_address": recovered.to_string(),
                    "recovered_address_matches": recovered_matches,
                    "metadata": response.metadata,
                    "keychain_service": response.keychain_service,
                    "keychain_account": response.keychain_account,
                    "keychain_item_id": response.keychain_item_id,
                    "access_policy": response.keychain_access_policy,
                    "device_id": response.device_id,
                    "kek_id": response.kek_id,
                    "plaintext_secret_process": "framkey-signer-helper",
                    "signer_helper": signer_helper_report(&helper_execution),
                }))?
            );

            if !recovered_matches {
                anyhow::bail!("helper signature recovery did not match helper address");
            }
        }
    }

    Ok(())
}
