use anyhow::Result;
use framkey_core::Generation;
use framkey_crypto::{SecretBytes, decode_hex_array, encode_hex, random_array};
use framkey_device::SaveImage;
use framkey_ipc::{SignerBuildKeychainVaultResponse, SignerRecoverKeychainVaultResponse};
use framkey_keychain_macos::SystemKeychain;
use framkey_vault::{
    build_dev_encrypted_save_image, build_test_save_image, inspect_save_image,
    open_dev_encrypted_save_image,
};
use serde_json::json;

use crate::{
    args::VaultCommand,
    constants::DEFAULT_KEYCHAIN_ACCESS_POLICY,
    files::write_new_file,
    recovery::{
        read_encrypted_vault_backup_from_bundle, read_recovery_backup_files,
        write_recovery_backup_pack,
    },
    signer_helper::{
        helper_build_keychain_vault, helper_open_keychain_vault, helper_recover_keychain_vault,
        signer_helper_report,
    },
};

pub(crate) fn run_vault(command: VaultCommand) -> Result<()> {
    match command {
        VaultCommand::BuildTestImage(args) => {
            let image = SaveImage::new(build_test_save_image(
                args.image_size,
                Generation(args.generation),
                &args.label,
            )?);
            write_new_file(&args.out, image.as_bytes())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": "build_test_image",
                    "out": args.out.display().to_string(),
                    "save_size": image.len(),
                    "blake3": image.blake3_hash().to_string(),
                }))?
            );
        }
        VaultCommand::InitKeychainKek(args) => {
            let item = args.keychain.item();
            let keychain = SystemKeychain;
            let loaded = keychain.ensure_kek(&item, DEFAULT_KEYCHAIN_ACCESS_POLICY)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": "init_keychain_kek",
                    "created": loaded.created,
                    "keychain_service": loaded.item.service,
                    "keychain_account": loaded.item.account,
                    "keychain_item_id": loaded.keychain_item_id,
                    "device_id": encode_hex(&loaded.device_id),
                    "kek_id": encode_hex(&loaded.kek_id),
                    "access_policy": loaded.access_policy.as_str(),
                }))?
            );
        }
        VaultCommand::RebindKeychainKek(args) => {
            let item = args.keychain.item();
            let keychain = SystemKeychain;
            let loaded = keychain.rebind_kek(&item, DEFAULT_KEYCHAIN_ACCESS_POLICY)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": "rebind_keychain_kek",
                    "keychain_service": loaded.item.service,
                    "keychain_account": loaded.item.account,
                    "keychain_item_id": loaded.keychain_item_id,
                    "device_id": encode_hex(&loaded.device_id),
                    "kek_id": encode_hex(&loaded.kek_id),
                    "access_policy": loaded.access_policy.as_str(),
                    "wallet_secret_touched": false,
                    "vault_image_touched": false,
                }))?
            );
        }
        VaultCommand::BuildKeychainEncryptedImage(args) => {
            let (response, helper_execution) = helper_build_keychain_vault(
                &args.helper,
                &args.keychain,
                args.image_size,
                Generation(args.generation),
                args.recovery_out_dir.is_some(),
            )?;

            let SignerBuildKeychainVaultResponse {
                save_image,
                keychain_service,
                keychain_account,
                keychain_item_id,
                keychain_access_policy,
                device_id,
                kek_id,
                created_keychain_kek,
                metadata,
                recovery_backup_pack,
            } = response;

            let image = SaveImage::new(save_image);
            write_new_file(&args.out, image.as_bytes())?;
            let recovery_backups = match (&args.recovery_out_dir, recovery_backup_pack) {
                (Some(out_dir), Some(pack)) => Some(write_recovery_backup_pack(
                    out_dir,
                    &pack,
                    image.as_bytes(),
                )?),
                (Some(_out_dir), None) => {
                    anyhow::bail!("signer helper did not return requested recovery backups")
                }
                (None, Some(_pack)) => {
                    anyhow::bail!("signer helper returned recovery backups without an output dir")
                }
                (None, None) => None,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": "build_keychain_encrypted_image",
                    "out": args.out.display().to_string(),
                    "save_size": image.len(),
                    "blake3": image.blake3_hash().to_string(),
                    "metadata": metadata,
                    "keychain_service": keychain_service,
                    "keychain_account": keychain_account,
                    "keychain_item_id": keychain_item_id,
                    "device_id": device_id,
                    "kek_id": kek_id,
                    "created_keychain_kek": created_keychain_kek,
                    "access_policy": keychain_access_policy,
                    "plaintext_secret_process": "framkey-signer-helper",
                    "recovery_backups": recovery_backups,
                    "signer_helper": signer_helper_report(&helper_execution),
                }))?
            );
        }
        VaultCommand::RecoverKeychainEncryptedImage(args) => {
            let image = read_encrypted_vault_backup_from_bundle(&args.path)?;
            let recovery_files = read_recovery_backup_files(&args.recovery_files)?;
            let recovery_file_count = recovery_files.len();
            let (response, helper_execution) =
                helper_recover_keychain_vault(&args.helper, &args.keychain, image, recovery_files)?;

            let SignerRecoverKeychainVaultResponse {
                save_image,
                keychain_service,
                keychain_account,
                keychain_item_id,
                keychain_access_policy,
                device_id,
                kek_id,
                created_keychain_kek,
                metadata,
                recovery_share_file_count,
            } = response;

            let image = SaveImage::new(save_image);
            write_new_file(&args.out, image.as_bytes())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": "recover_keychain_encrypted_image",
                    "path": args.path.display().to_string(),
                    "out": args.out.display().to_string(),
                    "save_size": image.len(),
                    "blake3": image.blake3_hash().to_string(),
                    "metadata": metadata,
                    "keychain_service": keychain_service,
                    "keychain_account": keychain_account,
                    "keychain_item_id": keychain_item_id,
                    "device_id": device_id,
                    "kek_id": kek_id,
                    "created_keychain_kek": created_keychain_kek,
                    "access_policy": keychain_access_policy,
                    "recovery_file_count": recovery_file_count,
                    "recovery_share_file_count": recovery_share_file_count,
                    "wallet_secret_touched": false,
                    "recovery_share_bytes_printed": false,
                    "plaintext_secret_process": "not_required_for_rewrap",
                    "signer_helper": signer_helper_report(&helper_execution),
                }))?
            );
        }
        VaultCommand::OpenKeychainEncryptedImage(args) => {
            let image = std::fs::read(&args.path)?;
            let (response, helper_execution) =
                helper_open_keychain_vault(&args.helper, &args.keychain, image)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": "open_keychain_encrypted_image",
                    "path": args.path.display().to_string(),
                    "address": response.address,
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
        }
        VaultCommand::GenerateDevKek => {
            let kek = random_array::<32>()?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": "generate_dev_kek",
                    "dev_only": true,
                    "dev_kek_hex": encode_hex(&kek),
                }))?
            );
        }
        VaultCommand::BuildDevEncryptedImage(args) => {
            let dev_kek = load_dev_kek(args.dev_kek_hex.as_deref())?;
            let built = build_dev_encrypted_save_image(
                args.image_size,
                Generation(args.generation),
                &args.label,
                &dev_kek,
            )?;
            let image = SaveImage::new(built.save_image);
            write_new_file(&args.out, image.as_bytes())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": "build_dev_encrypted_image",
                    "out": args.out.display().to_string(),
                    "save_size": image.len(),
                    "blake3": image.blake3_hash().to_string(),
                    "metadata": built.metadata,
                    "dev_only": true,
                }))?
            );
        }
        VaultCommand::OpenDevEncryptedImage(args) => {
            let dev_kek = load_dev_kek(args.dev_kek_hex.as_deref())?;
            let image = std::fs::read(&args.path)?;
            let metadata = open_dev_encrypted_save_image(&image, &dev_kek)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "operation": "open_dev_encrypted_image",
                    "path": args.path.display().to_string(),
                    "metadata": metadata,
                    "dev_only": true,
                }))?
            );
        }
        VaultCommand::InspectImage(args) => {
            let image = std::fs::read(&args.path)?;
            let inspection = inspect_save_image(&image)?;
            println!("{}", serde_json::to_string_pretty(&inspection)?);
        }
    }

    Ok(())
}

fn load_dev_kek(arg: Option<&str>) -> Result<SecretBytes<32>> {
    let value = match arg {
        Some(value) => value.to_owned(),
        None => std::env::var("FRAMKEY_DEV_KEK_HEX").map_err(|_| {
            anyhow::anyhow!("pass --dev-kek-hex or set FRAMKEY_DEV_KEK_HEX for dev/test vaults")
        })?,
    };

    Ok(SecretBytes::new(decode_hex_array::<32>(&value)?))
}
