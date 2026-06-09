use anyhow::Result;
use framkey_btc::{USER_VISIBLE_BTC_NETWORKS, p2wpkh_account_from_secret, sign_p2wpkh_psbt};
use framkey_core::{FramkeyError, Generation, Result as FramkeyResult, WalletType};
use framkey_crypto::encode_hex;
use framkey_evm::{
    EvmTransaction, address_from_secret, personal_sign, sign_transaction, sign_typed_data_v4,
};
use framkey_ipc::{
    SignerBuildKeychainVaultResponse, SignerChainAccount, SignerHelperRequest,
    SignerHelperResponse, SignerHelperResult, SignerKeychainAccessProbeResponse,
    SignerOpenKeychainVaultResponse, SignerPersonalSignResponse,
    SignerRecoverKeychainVaultResponse, SignerSignBtcPsbtResponse, SignerSignTransactionResponse,
    SignerSignTypedDataResponse,
};
use framkey_keychain_macos::{KeychainAccessPolicy, MacKeychainItem, SystemKeychain};
use framkey_vault::{
    build_keychain_encrypted_save_image, build_keychain_encrypted_save_image_with_recovery,
    rewrap_keychain_vault_with_recovery, with_keychain_wallet_secret,
};

use crate::{
    io::{read_limited_stdin, write_json_response},
    metadata::{encrypted_metadata_to_ipc, metadata_to_ipc},
    recovery::validate_recovery_files_drill,
    validation::{
        parse_expected_address, transaction_kind_name, validate_expected_address,
        validate_personal_sign_message, validate_recovery_files, validate_save_image_size,
        validate_sign_btc_psbt_request, validate_sign_transaction_request,
        validate_typed_data_request,
    },
};

const DEFAULT_KEYCHAIN_ACCESS_POLICY: KeychainAccessPolicy =
    KeychainAccessPolicy::LocalDeviceOwnerAuthentication;

pub(crate) fn run() -> Result<()> {
    let input = read_limited_stdin()?;
    let request: SignerHelperRequest = serde_json::from_slice(&input)?;
    let response = match request {
        SignerHelperRequest::KeychainAccessProbe(request) => {
            let item = MacKeychainItem::new(request.keychain_service, request.keychain_account);
            let keychain = SystemKeychain;
            let loaded = keychain.load_existing_kek(&item)?;
            SignerHelperResponse::ok(SignerHelperResult::KeychainAccessProbe(
                SignerKeychainAccessProbeResponse {
                    keychain_service: loaded.item.service,
                    keychain_account: loaded.item.account,
                    keychain_item_id: loaded.keychain_item_id,
                    keychain_access_policy: loaded.access_policy.as_str().to_owned(),
                    device_id: encode_hex(&loaded.device_id),
                    kek_id: encode_hex(&loaded.kek_id),
                    card_touched: false,
                    vault_image_touched: false,
                    wallet_secret_touched: false,
                },
            ))
        }
        SignerHelperRequest::BuildKeychainVault(request) => {
            validate_save_image_size(request.image_size)?;
            let item = MacKeychainItem::new(request.keychain_service, request.keychain_account);
            let keychain = SystemKeychain;
            let loaded = keychain.reset_kek(&item, DEFAULT_KEYCHAIN_ACCESS_POLICY)?;
            let built = if request.recovery_backups {
                build_keychain_encrypted_save_image_with_recovery(
                    request.image_size,
                    Generation(request.generation),
                    &loaded.keychain_item_id,
                    loaded.device_id,
                    &loaded.kek,
                )?
            } else {
                build_keychain_encrypted_save_image(
                    request.image_size,
                    Generation(request.generation),
                    &loaded.keychain_item_id,
                    loaded.device_id,
                    &loaded.kek,
                )?
            };
            SignerHelperResponse::ok(SignerHelperResult::BuildKeychainVault(
                SignerBuildKeychainVaultResponse {
                    save_image: built.save_image,
                    keychain_service: loaded.item.service,
                    keychain_account: loaded.item.account,
                    keychain_item_id: loaded.keychain_item_id,
                    keychain_access_policy: loaded.access_policy.as_str().to_owned(),
                    device_id: encode_hex(&loaded.device_id),
                    kek_id: encode_hex(&loaded.kek_id),
                    created_keychain_kek: loaded.created,
                    metadata: encrypted_metadata_to_ipc(built.metadata),
                    recovery_backup_pack: built.recovery_backup_pack,
                },
            ))
        }
        SignerHelperRequest::RecoverKeychainVault(request) => {
            validate_save_image_size(request.save_image.len())?;
            validate_recovery_files(request.recovery_files.len())?;
            let item = MacKeychainItem::new(request.keychain_service, request.keychain_account);
            let keychain = SystemKeychain;
            let loaded = keychain.reset_kek(&item, DEFAULT_KEYCHAIN_ACCESS_POLICY)?;
            let recovered = rewrap_keychain_vault_with_recovery(
                &request.save_image,
                &request.recovery_files,
                &loaded.keychain_item_id,
                loaded.device_id,
                &loaded.kek,
            )?;
            SignerHelperResponse::ok(SignerHelperResult::RecoverKeychainVault(
                SignerRecoverKeychainVaultResponse {
                    save_image: recovered.save_image,
                    keychain_service: loaded.item.service,
                    keychain_account: loaded.item.account,
                    keychain_item_id: loaded.keychain_item_id,
                    keychain_access_policy: loaded.access_policy.as_str().to_owned(),
                    device_id: encode_hex(&loaded.device_id),
                    kek_id: encode_hex(&loaded.kek_id),
                    created_keychain_kek: loaded.created,
                    metadata: metadata_to_ipc(recovered.metadata, None),
                    recovery_share_file_count: request.recovery_files.len(),
                },
            ))
        }
        SignerHelperRequest::ValidateRecoveryFiles(request) => {
            validate_recovery_files(request.recovery_files.len())?;
            SignerHelperResponse::ok(SignerHelperResult::ValidateRecoveryFiles(
                validate_recovery_files_drill(&request.recovery_files)?,
            ))
        }
        SignerHelperRequest::OpenKeychainVault(request) => {
            validate_save_image_size(request.save_image.len())?;
            let item = MacKeychainItem::new(request.keychain_service, request.keychain_account);
            let keychain = SystemKeychain;
            let loaded = keychain.load_existing_kek(&item)?;
            let (metadata, (wallet_secret_hash, address, accounts)) = with_keychain_wallet_secret(
                &request.save_image,
                &loaded.keychain_item_id,
                loaded.device_id,
                &loaded.kek,
                |metadata, wallet_secret| {
                    let wallet_secret_hash =
                        encode_hex(blake3::hash(wallet_secret.expose()).as_bytes());
                    let accounts = chain_accounts_from_secret(metadata.wallet_type, wallet_secret)?;
                    let address = accounts
                        .iter()
                        .find(|account| account.family == "evm")
                        .map(|account| account.address.clone());
                    Ok((wallet_secret_hash, address, accounts))
                },
            )?;
            SignerHelperResponse::ok(SignerHelperResult::OpenKeychainVault(
                SignerOpenKeychainVaultResponse {
                    keychain_service: loaded.item.service,
                    keychain_account: loaded.item.account,
                    keychain_item_id: loaded.keychain_item_id,
                    keychain_access_policy: loaded.access_policy.as_str().to_owned(),
                    device_id: encode_hex(&loaded.device_id),
                    kek_id: encode_hex(&loaded.kek_id),
                    metadata: metadata_to_ipc(metadata, Some(wallet_secret_hash)),
                    address,
                    accounts,
                },
            ))
        }
        SignerHelperRequest::PersonalSign(request) => {
            validate_save_image_size(request.save_image.len())?;
            validate_personal_sign_message(&request.message)?;
            let expected_address = parse_expected_address(request.expected_address.as_deref())?;
            let item = MacKeychainItem::new(request.keychain_service, request.keychain_account);
            let keychain = SystemKeychain;
            let loaded = keychain.load_existing_kek(&item)?;
            let (metadata, signed) = with_keychain_wallet_secret(
                &request.save_image,
                &loaded.keychain_item_id,
                loaded.device_id,
                &loaded.kek,
                |metadata, wallet_secret| {
                    if !metadata.wallet_type.supports_evm_eoa() {
                        return Err(FramkeyError::unsupported(
                            "signer helper only supports EVM-capable secp256k1 vaults",
                        ));
                    }
                    let address = address_from_secret(wallet_secret)?;
                    validate_expected_address(address, expected_address)?;
                    personal_sign(wallet_secret, &request.message)
                },
            )?;

            SignerHelperResponse::ok(SignerHelperResult::PersonalSign(
                SignerPersonalSignResponse {
                    keychain_service: loaded.item.service,
                    keychain_account: loaded.item.account,
                    keychain_item_id: loaded.keychain_item_id,
                    keychain_access_policy: loaded.access_policy.as_str().to_owned(),
                    device_id: encode_hex(&loaded.device_id),
                    kek_id: encode_hex(&loaded.kek_id),
                    metadata: metadata_to_ipc(metadata, None),
                    address: signed.address.to_string(),
                    message_hash: signed.message_hash_hex(),
                    signature: signed.signature_hex(),
                },
            ))
        }
        SignerHelperRequest::SignTypedData(request) => {
            validate_save_image_size(request.save_image.len())?;
            validate_typed_data_request(&request.typed_data)?;
            let expected_address = parse_expected_address(request.expected_address.as_deref())?;
            let item = MacKeychainItem::new(request.keychain_service, request.keychain_account);
            let keychain = SystemKeychain;
            let loaded = keychain.load_existing_kek(&item)?;
            let (metadata, signed) = with_keychain_wallet_secret(
                &request.save_image,
                &loaded.keychain_item_id,
                loaded.device_id,
                &loaded.kek,
                |metadata, wallet_secret| {
                    if !metadata.wallet_type.supports_evm_eoa() {
                        return Err(FramkeyError::unsupported(
                            "signer helper only supports EVM-capable secp256k1 vaults",
                        ));
                    }
                    let address = address_from_secret(wallet_secret)?;
                    validate_expected_address(address, expected_address)?;
                    sign_typed_data_v4(wallet_secret, &request.typed_data)
                },
            )?;

            SignerHelperResponse::ok(SignerHelperResult::SignTypedData(
                SignerSignTypedDataResponse {
                    keychain_service: loaded.item.service,
                    keychain_account: loaded.item.account,
                    keychain_item_id: loaded.keychain_item_id,
                    keychain_access_policy: loaded.access_policy.as_str().to_owned(),
                    device_id: encode_hex(&loaded.device_id),
                    kek_id: encode_hex(&loaded.kek_id),
                    metadata: metadata_to_ipc(metadata, None),
                    address: signed.address.to_string(),
                    typed_data_hash: signed.typed_data_hash_hex(),
                    signature: signed.signature_hex(),
                },
            ))
        }
        SignerHelperRequest::SignTransaction(request) => {
            validate_save_image_size(request.save_image.len())?;
            validate_sign_transaction_request(&request.transaction)?;
            let expected_address = parse_expected_address(request.expected_address.as_deref())?;
            let item = MacKeychainItem::new(request.keychain_service, request.keychain_account);
            let keychain = SystemKeychain;
            let loaded = keychain.load_existing_kek(&item)?;
            let transaction = EvmTransaction {
                chain_id: request.transaction.chain_id,
                nonce: request.transaction.nonce,
                gas_limit: request.transaction.gas_limit,
                to: request.transaction.to,
                value: request.transaction.value,
                data: request.transaction.data,
                gas_price: request.transaction.gas_price,
                max_fee_per_gas: request.transaction.max_fee_per_gas,
                max_priority_fee_per_gas: request.transaction.max_priority_fee_per_gas,
            };
            let (metadata, signed) = with_keychain_wallet_secret(
                &request.save_image,
                &loaded.keychain_item_id,
                loaded.device_id,
                &loaded.kek,
                |metadata, wallet_secret| {
                    if !metadata.wallet_type.supports_evm_eoa() {
                        return Err(FramkeyError::unsupported(
                            "signer helper only supports EVM-capable secp256k1 vaults",
                        ));
                    }
                    let address = address_from_secret(wallet_secret)?;
                    validate_expected_address(address, expected_address)?;
                    sign_transaction(wallet_secret, &transaction)
                },
            )?;

            SignerHelperResponse::ok(SignerHelperResult::SignTransaction(
                SignerSignTransactionResponse {
                    keychain_service: loaded.item.service,
                    keychain_account: loaded.item.account,
                    keychain_item_id: loaded.keychain_item_id,
                    keychain_access_policy: loaded.access_policy.as_str().to_owned(),
                    device_id: encode_hex(&loaded.device_id),
                    kek_id: encode_hex(&loaded.kek_id),
                    metadata: metadata_to_ipc(metadata, None),
                    address: signed.address.to_string(),
                    transaction_kind: transaction_kind_name(signed.kind).to_owned(),
                    transaction_hash: signed.transaction_hash_hex(),
                    raw_transaction: signed.raw_transaction_hex(),
                },
            ))
        }
        SignerHelperRequest::SignBtcPsbt(request) => {
            validate_save_image_size(request.save_image.len())?;
            let network = validate_sign_btc_psbt_request(&request.psbt, &request.expected_address)?;
            let item = MacKeychainItem::new(request.keychain_service, request.keychain_account);
            let keychain = SystemKeychain;
            let loaded = keychain.load_existing_kek(&item)?;
            let (metadata, signed) = with_keychain_wallet_secret(
                &request.save_image,
                &loaded.keychain_item_id,
                loaded.device_id,
                &loaded.kek,
                |metadata, wallet_secret| {
                    if !metadata.wallet_type.supports_btc_single_key() {
                        return Err(FramkeyError::unsupported(
                            "signer helper only supports BTC signing for secp256k1 single-key vaults",
                        ));
                    }
                    sign_p2wpkh_psbt(
                        wallet_secret,
                        network,
                        &request.expected_address,
                        &request.psbt.bytes,
                    )
                },
            )?;

            SignerHelperResponse::ok(SignerHelperResult::SignBtcPsbt(SignerSignBtcPsbtResponse {
                keychain_service: loaded.item.service,
                keychain_account: loaded.item.account,
                keychain_item_id: loaded.keychain_item_id,
                keychain_access_policy: loaded.access_policy.as_str().to_owned(),
                device_id: encode_hex(&loaded.device_id),
                kek_id: encode_hex(&loaded.kek_id),
                metadata: metadata_to_ipc(metadata, None),
                network: signed.network.id().to_owned(),
                address: signed.address,
                transaction_id: signed.transaction_id,
                raw_transaction: signed.raw_transaction,
                vbytes: signed.vbytes,
            }))
        }
    };

    write_json_response(&response)?;
    Ok(())
}

pub(crate) fn chain_accounts_from_secret(
    wallet_type: WalletType,
    wallet_secret: &framkey_crypto::SecretBytes<32>,
) -> FramkeyResult<Vec<SignerChainAccount>> {
    if !wallet_type.uses_secp256k1_secret() {
        return Ok(Vec::new());
    }

    let evm_address = address_from_secret(wallet_secret)?;
    let btc_accounts = if wallet_type.supports_btc_single_key() {
        USER_VISIBLE_BTC_NETWORKS
            .iter()
            .map(|network| p2wpkh_account_from_secret(wallet_secret, *network))
            .collect::<FramkeyResult<Vec<_>>>()?
    } else {
        Vec::new()
    };
    let mut accounts = vec![SignerChainAccount {
        family: "evm".to_owned(),
        network: "evm-active-chain".to_owned(),
        address: evm_address.to_string(),
        address_type: "eoa".to_owned(),
        key_role: "secp256k1_signer".to_owned(),
    }];
    for account in btc_accounts {
        accounts.push(SignerChainAccount {
            family: "btc".to_owned(),
            network: account.network.id().to_owned(),
            address: account.address,
            address_type: account.address_type,
            key_role: account.script_policy,
        });
    }
    Ok(accounts)
}
