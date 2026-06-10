use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use crate::*;
use anyhow::{Context, Result};
use framkey_crypto::encode_hex;
use framkey_device::SaveImage;
use framkey_evm::EvmTransaction;
use framkey_ipc::{
    MAX_SIGNER_HELPER_SAVE_IMAGE_BYTES, MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES, SignerBtcPsbt,
    SignerBuildKeychainVaultRequest, SignerBuildKeychainVaultResponse, SignerEvmTransaction,
    SignerHelperRequest, SignerHelperResponse, SignerHelperResult,
    SignerKeychainAccessProbeRequest, SignerKeychainAccessProbeResponse,
    SignerOpenKeychainVaultRequest, SignerOpenKeychainVaultResponse, SignerPersonalSignRequest,
    SignerPersonalSignResponse, SignerRecoverKeychainVaultRequest,
    SignerRecoverKeychainVaultResponse, SignerSignBtcPsbtRequest, SignerSignBtcPsbtResponse,
    SignerSignTransactionRequest, SignerSignTransactionResponse, SignerSignTypedDataRequest,
    SignerSignTypedDataResponse, SignerValidateRecoveryFilesRequest,
    SignerValidateRecoveryFilesResponse,
};
use framkey_recovery::{
    RecoveryBackupBundle, RecoveryBackupFile, RecoveryBackupPack, parse_recovery_backup_bundle,
    recovery_backup_file_name,
};
use framkey_vault::inspect_save_image;
use serde_json::{Value, json};

pub(crate) fn load_keychain_account(config: &DesktopConfig) -> Result<DesktopAccount> {
    let started_at = Instant::now();
    eprintln!("framkey_account_connect stage=read_vault_start");
    let save_image = read_configured_save_image(config)?;
    let read_elapsed = started_at.elapsed();
    eprintln!(
        "framkey_account_connect stage=read_vault_done duration_ms={}",
        read_elapsed.as_millis()
    );
    let unlock_started_at = Instant::now();
    eprintln!("framkey_account_connect stage=local_unlock_start");
    let opened = open_keychain_vault_with_helper(config, save_image)?;
    eprintln!(
        "framkey_account_connect stage=local_unlock_done duration_ms={}",
        unlock_started_at.elapsed().as_millis()
    );
    let address = opened
        .address
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Keychain vault did not expose an EVM address"))?;
    let accounts = desktop_accounts_from_signer(config, &address, &opened.accounts);
    let helper_report = helper_report(&config.helper)?;
    eprintln!(
        "framkey_account_connect stage=complete duration_ms={}",
        started_at.elapsed().as_millis()
    );

    Ok(DesktopAccount {
        address,
        accounts,
        wallet: json!({
            "kind": "keychain_vault",
            "mock": false,
        }),
        metadata: json!(opened.metadata),
        keychain: Some(json!({
            "service": opened.keychain_service,
            "account": opened.keychain_account,
            "accessPolicy": opened.keychain_access_policy,
            "itemId": opened.keychain_item_id,
            "deviceId": opened.device_id,
            "kekId": opened.kek_id,
        })),
        helper_report: Some(helper_report),
    })
}

pub(crate) fn read_configured_save_image(config: &DesktopConfig) -> Result<Vec<u8>> {
    let device = config.device.open_device();
    let save_image = device.read_save_image()?.as_bytes().to_vec();
    if config.wallet == DesktopWalletConfig::KeychainVault {
        let checkpoint = inspect_keychain_vault_checkpoint(&save_image).with_context(|| {
            format!(
                "configured save image from {} is not a valid FRAMKey Keychain vault image",
                config.device.describe()
            )
        })?;
        enforce_configured_vault_high_water(&checkpoint)?;
    } else {
        inspect_save_image(&save_image).with_context(|| {
            format!(
                "configured save image from {} is not a valid FRAMKey vault image",
                config.device.describe()
            )
        })?;
    }
    Ok(save_image)
}

pub(crate) fn write_configured_save_image(config: &DesktopConfig, image: &SaveImage) -> Result<()> {
    let mut device = config.device.open_device();
    device.write_save_image(image)?;
    Ok(())
}

pub(crate) fn authorize_keychain_helper_access(config: &DesktopConfig) -> Result<Value> {
    let started = Instant::now();
    let helper = helper_report(&config.helper)?;
    let helper_cdhash = signer_helper_cdhash(&config.helper.path).ok();
    let partition_list = helper_cdhash
        .as_ref()
        .map(|cdhash| format!("cdhash:{cdhash}"));
    let probe = keychain_access_probe_with_helper(config)?;
    Ok(json!({
        "operation": "keychain_helper_access_probe",
        "status": "authorized",
        "durationMs": duration_ms(started.elapsed()),
        "helper": helper,
        "helperCdhash": helper_cdhash,
        "partitionList": partition_list,
        "keychain": {
            "service": probe.keychain_service,
            "account": probe.keychain_account,
            "accessPolicy": probe.keychain_access_policy,
            "itemId": probe.keychain_item_id,
            "deviceId": probe.device_id,
            "kekId": probe.kek_id,
        },
        "cardTouched": probe.card_touched,
        "vaultImageTouched": probe.vault_image_touched,
        "walletSecretTouched": probe.wallet_secret_touched,
        "passwordCapturedByFramkey": false,
    }))
}

pub(crate) fn keychain_access_probe_with_helper(
    config: &DesktopConfig,
) -> Result<SignerKeychainAccessProbeResponse> {
    let response = run_signer_helper(
        &config.helper,
        &SignerHelperRequest::KeychainAccessProbe(SignerKeychainAccessProbeRequest {
            keychain_service: config.keychain_service.clone(),
            keychain_account: config.keychain_account.clone(),
        }),
    )?;

    match response.into_result() {
        Ok(SignerHelperResult::KeychainAccessProbe(result)) => Ok(result),
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

pub(crate) fn build_keychain_vault_with_helper(
    config: &DesktopConfig,
    image_size: usize,
    generation: u64,
) -> Result<SignerBuildKeychainVaultResponse> {
    let response = run_signer_helper(
        &config.helper,
        &SignerHelperRequest::BuildKeychainVault(SignerBuildKeychainVaultRequest {
            image_size,
            generation,
            keychain_service: config.keychain_service.clone(),
            keychain_account: config.keychain_account.clone(),
            recovery_backups: true,
        }),
    )?;

    match response.into_result() {
        Ok(SignerHelperResult::BuildKeychainVault(result)) => Ok(result),
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

pub(crate) fn recover_keychain_vault_with_helper(
    config: &DesktopConfig,
    save_image: Vec<u8>,
    recovery_files: Vec<RecoveryBackupFile>,
) -> Result<SignerRecoverKeychainVaultResponse> {
    let response = run_signer_helper(
        &config.helper,
        &SignerHelperRequest::RecoverKeychainVault(SignerRecoverKeychainVaultRequest {
            save_image,
            keychain_service: config.keychain_service.clone(),
            keychain_account: config.keychain_account.clone(),
            recovery_files,
        }),
    )?;

    match response.into_result() {
        Ok(SignerHelperResult::RecoverKeychainVault(result)) => Ok(result),
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

pub(crate) fn validate_recovery_files_with_helper(
    config: &DesktopConfig,
    recovery_files: Vec<RecoveryBackupFile>,
) -> Result<SignerValidateRecoveryFilesResponse> {
    let response = run_signer_helper(
        &config.helper,
        &SignerHelperRequest::ValidateRecoveryFiles(SignerValidateRecoveryFilesRequest {
            recovery_files,
        }),
    )?;

    match response.into_result() {
        Ok(SignerHelperResult::ValidateRecoveryFiles(result)) => Ok(result),
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

pub(crate) fn open_keychain_vault_with_helper(
    config: &DesktopConfig,
    save_image: Vec<u8>,
) -> Result<SignerOpenKeychainVaultResponse> {
    let response = run_signer_helper(
        &config.helper,
        &SignerHelperRequest::OpenKeychainVault(SignerOpenKeychainVaultRequest {
            save_image,
            keychain_service: config.keychain_service.clone(),
            keychain_account: config.keychain_account.clone(),
        }),
    )?;

    match response.into_result() {
        Ok(SignerHelperResult::OpenKeychainVault(result)) => {
            remember_vault_generation_from_signer(
                &result.metadata,
                &result.keychain_item_id,
                &result.device_id,
            )?;
            Ok(result)
        }
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

pub(crate) fn personal_sign_with_helper(
    config: &DesktopConfig,
    save_image: Vec<u8>,
    message: Vec<u8>,
    expected_address: Option<String>,
) -> Result<SignerPersonalSignResponse> {
    let response = run_signer_helper(
        &config.helper,
        &SignerHelperRequest::PersonalSign(SignerPersonalSignRequest {
            save_image,
            keychain_service: config.keychain_service.clone(),
            keychain_account: config.keychain_account.clone(),
            message,
            expected_address,
        }),
    )?;

    match response.into_result() {
        Ok(SignerHelperResult::PersonalSign(result)) => {
            remember_vault_generation_from_signer(
                &result.metadata,
                &result.keychain_item_id,
                &result.device_id,
            )?;
            Ok(result)
        }
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

pub(crate) fn sign_typed_data_with_helper(
    config: &DesktopConfig,
    save_image: Vec<u8>,
    typed_data: Value,
    expected_address: Option<String>,
) -> Result<SignerSignTypedDataResponse> {
    let response = run_signer_helper(
        &config.helper,
        &SignerHelperRequest::SignTypedData(SignerSignTypedDataRequest {
            save_image,
            keychain_service: config.keychain_service.clone(),
            keychain_account: config.keychain_account.clone(),
            typed_data,
            expected_address,
        }),
    )?;

    match response.into_result() {
        Ok(SignerHelperResult::SignTypedData(result)) => {
            remember_vault_generation_from_signer(
                &result.metadata,
                &result.keychain_item_id,
                &result.device_id,
            )?;
            Ok(result)
        }
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

pub(crate) fn sign_transaction_with_helper(
    config: &DesktopConfig,
    save_image: Vec<u8>,
    transaction: EvmTransaction,
    expected_address: Option<String>,
) -> Result<SignerSignTransactionResponse> {
    let response = run_signer_helper(
        &config.helper,
        &SignerHelperRequest::SignTransaction(SignerSignTransactionRequest {
            save_image,
            keychain_service: config.keychain_service.clone(),
            keychain_account: config.keychain_account.clone(),
            transaction: signer_evm_transaction(transaction),
            expected_address,
        }),
    )?;

    match response.into_result() {
        Ok(SignerHelperResult::SignTransaction(result)) => {
            remember_vault_generation_from_signer(
                &result.metadata,
                &result.keychain_item_id,
                &result.device_id,
            )?;
            Ok(result)
        }
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

pub(crate) fn sign_btc_psbt_with_helper(
    config: &DesktopConfig,
    save_image: Vec<u8>,
    network: &str,
    psbt_bytes: Vec<u8>,
    expected_address: String,
) -> Result<SignerSignBtcPsbtResponse> {
    let response = run_signer_helper(
        &config.helper,
        &SignerHelperRequest::SignBtcPsbt(SignerSignBtcPsbtRequest {
            save_image,
            keychain_service: config.keychain_service.clone(),
            keychain_account: config.keychain_account.clone(),
            psbt: SignerBtcPsbt {
                network: network.to_owned(),
                bytes: psbt_bytes,
            },
            expected_address,
        }),
    )?;

    match response.into_result() {
        Ok(SignerHelperResult::SignBtcPsbt(result)) => {
            remember_vault_generation_from_signer(
                &result.metadata,
                &result.keychain_item_id,
                &result.device_id,
            )?;
            Ok(result)
        }
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

pub(crate) fn remember_vault_generation_from_signer(
    metadata: &framkey_ipc::SignerVaultMetadata,
    keychain_item_id: &str,
    device_id: &str,
) -> Result<()> {
    remember_configured_vault_generation(checkpoint_from_signer_metadata(
        metadata,
        keychain_item_id,
        device_id,
    )?)
}

pub(crate) fn signer_evm_transaction(transaction: EvmTransaction) -> SignerEvmTransaction {
    SignerEvmTransaction {
        chain_id: transaction.chain_id,
        nonce: transaction.nonce,
        gas_limit: transaction.gas_limit,
        to: transaction.to,
        value: transaction.value,
        data: transaction.data,
        gas_price: transaction.gas_price,
        max_fee_per_gas: transaction.max_fee_per_gas,
        max_priority_fee_per_gas: transaction.max_priority_fee_per_gas,
    }
}

pub(crate) fn run_signer_helper(
    helper: &SignerHelperConfig,
    request: &SignerHelperRequest,
) -> Result<SignerHelperResponse> {
    verify_helper_hash(helper)?;

    let mut command = signer_helper_command(helper);
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start signer helper {}", helper.path.display()))?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("signer helper stdin was not available"))?;
        serde_json::to_writer(&mut stdin, request)?;
        stdin.write_all(b"\n")?;
    }

    let output = wait_for_signer_helper_output(child, SIGNER_HELPER_TIMEOUT)?;
    if output.stdout.is_empty() && !output.status.success() {
        anyhow::bail!(
            "signer helper exited with {} before returning JSON; stderr: {}",
            output.status,
            signer_helper_stderr_summary(&output.stderr)
        );
    }
    let response: SignerHelperResponse =
        serde_json::from_slice(&output.stdout).map_err(|error| {
            anyhow::anyhow!(
                "failed to parse signer helper response: {error}; stderr: {}",
                signer_helper_stderr_summary(&output.stderr)
            )
        })?;

    if !output.status.success() {
        match &response {
            SignerHelperResponse::Error { error } => {
                anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message);
            }
            SignerHelperResponse::Ok { .. } => {
                anyhow::bail!("signer helper exited with {}", output.status);
            }
        }
    }

    Ok(response)
}

pub(crate) fn wait_for_signer_helper_output(mut child: Child, timeout: Duration) -> Result<Output> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("signer helper stdout was not available"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("signer helper stderr was not available"))?;
    let stdout_reader = std::thread::spawn(move || {
        let mut stdout = stdout;
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut stderr = stderr;
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });

    let started_at = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let stdout = join_output_reader(stdout_reader, "stdout")?;
            let stderr = join_output_reader(stderr_reader, "stderr")?;
            return Ok(Output {
                status,
                stdout,
                stderr,
            });
        }
        if started_at.elapsed() >= timeout {
            let _ = child.kill();
            let _status = child.wait()?;
            let stdout = join_output_reader(stdout_reader, "stdout")?;
            let stderr = join_output_reader(stderr_reader, "stderr")?;
            anyhow::bail!(
                "signer helper timed out after {} ms; stdout: {} bytes; stderr: {}",
                timeout.as_millis(),
                stdout.len(),
                signer_helper_stderr_summary(&stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn join_output_reader(
    handle: std::thread::JoinHandle<std::io::Result<Vec<u8>>>,
    stream: &str,
) -> Result<Vec<u8>> {
    handle
        .join()
        .map_err(|_| anyhow::anyhow!("signer helper {stream} reader panicked"))?
        .map_err(|error| anyhow::anyhow!("failed to read signer helper {stream}: {error}"))
}

pub(crate) fn signer_helper_stderr_summary(stderr: &[u8]) -> String {
    if stderr.is_empty() {
        "empty".to_owned()
    } else {
        format!("{} bytes redacted", stderr.len())
    }
}

pub(crate) fn signer_helper_command(helper: &SignerHelperConfig) -> Command {
    match helper.sandbox {
        SignerHelperSandbox::MacosProcessIdentity => Command::new(&helper.path),
        SignerHelperSandbox::MacosSandboxExecNoNetwork => {
            let mut command = Command::new("/usr/bin/sandbox-exec");
            command
                .arg("-p")
                .arg(MACOS_NO_NETWORK_SANDBOX_PROFILE)
                .arg(&helper.path);
            command
        }
        SignerHelperSandbox::DisabledByConfig => Command::new(&helper.path),
        #[cfg(not(target_os = "macos"))]
        SignerHelperSandbox::UnsupportedPlatform => Command::new(&helper.path),
    }
}

pub(crate) fn verify_helper_hash(helper: &SignerHelperConfig) -> Result<()> {
    if let Some(expected) = &helper.expected_blake3 {
        let actual = hash_file_blake3(&helper.path)?;
        if expected != &actual {
            anyhow::bail!(
                "signer helper BLAKE3 mismatch: expected {}, got {} for {}",
                expected,
                actual,
                helper.path.display()
            );
        }
    }
    Ok(())
}

pub(crate) fn helper_report(helper: &SignerHelperConfig) -> Result<Value> {
    Ok(json!({
        "path": helper.path.display().to_string(),
        "blake3": hash_file_blake3(&helper.path)?,
        "sandbox": helper.sandbox.as_str(),
    }))
}

pub(crate) fn signer_helper_status_value(helper: &SignerHelperConfig) -> Value {
    let exists = helper.path.exists();
    let blake3 = if exists {
        hash_file_blake3(&helper.path).ok()
    } else {
        None
    };
    let hash_matches = match (&helper.expected_blake3, &blake3) {
        (Some(expected), Some(actual)) => Some(expected == actual),
        (Some(_), None) => Some(false),
        (None, _) => None,
    };
    let readiness = match (exists, hash_matches) {
        (false, _) => "missing",
        (true, Some(false)) => "hash_mismatch",
        (true, _) => "ready",
    };
    json!({
        "path": helper.path.display().to_string(),
        "exists": exists,
        "ready": readiness == "ready",
        "readiness": readiness,
        "location": signer_helper_location(&helper.path),
        "sandbox": helper.sandbox.as_str(),
        "hashPinned": helper.expected_blake3.is_some(),
        "hashMatches": hash_matches,
        "blake3": blake3,
    })
}

pub(crate) fn signer_helper_location(path: &Path) -> &'static str {
    let path_text = path.display().to_string();
    if path_text.contains(".app/Contents/MacOS") || path_text.contains(".app/Contents/Resources") {
        return "bundled_app";
    }
    if path_text.contains("/src-tauri/binaries/") {
        return "sidecar_source";
    }
    if path_text.contains("/target/debug/") || path_text.contains("/target/release/") {
        return "cargo_target";
    }
    "custom"
}

pub(crate) fn hash_file_blake3(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(encode_hex(hasher.finalize().as_bytes()))
}

pub(crate) fn signer_helper_cdhash(path: &Path) -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/bin/codesign")
            .arg("-dv")
            .arg("--verbose=4")
            .arg(path)
            .output()
            .map_err(|error| anyhow::anyhow!("failed to run /usr/bin/codesign: {error}"))?;
        if !output.status.success() {
            anyhow::bail!(
                "codesign failed for {} with {}; stderr: {}",
                path.display(),
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let cdhash = parse_codesign_cdhash(&stderr).ok_or_else(|| {
            anyhow::anyhow!(
                "codesign output for {} did not contain CDHash",
                path.display()
            )
        })?;
        Ok(cdhash.to_owned())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        anyhow::bail!("signer-helper CDHash is only available on macOS");
    }
}

pub(crate) fn parse_codesign_cdhash(output: &str) -> Option<&str> {
    output.lines().find_map(|line| {
        let cdhash = line.trim().strip_prefix("CDHash=")?.trim();
        if cdhash.len() == 40
            && cdhash
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            Some(cdhash)
        } else {
            None
        }
    })
}

pub(crate) fn recovery_smoke_encrypted_vault_backup(
    pack: &RecoveryBackupPack,
    generation: u64,
) -> Vec<u8> {
    format!(
        concat!(
            "FRAMKey development recovery smoke encrypted vault placeholder\n",
            "This is not a real wallet vault image and contains no wallet secret.\n",
            "backup_set_id={}\n",
            "wallet_id={}\n",
            "generation={}\n"
        ),
        pack.manifest.backup_set_id, pack.manifest.wallet_id, generation
    )
    .into_bytes()
}

pub(crate) fn write_recovery_backup_pack(
    out_dir: &Path,
    pack: &RecoveryBackupPack,
    encrypted_vault_backup: Option<&[u8]>,
) -> Result<Value> {
    validate_recovery_backup_pack_targets(pack)?;
    create_private_dir_all(out_dir)
        .with_context(|| format!("failed to create recovery directory {}", out_dir.display()))?;

    let mut created_paths = Vec::new();
    match write_recovery_backup_pack_files(
        out_dir,
        pack,
        encrypted_vault_backup,
        &mut created_paths,
    ) {
        Ok(summary) => Ok(summary),
        Err(error) => match cleanup_paths(&created_paths) {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(anyhow::anyhow!(
                "{error}; failed to remove partial recovery files: {cleanup_error}"
            )),
        },
    }
}

pub(crate) fn validate_recovery_backup_pack_targets(pack: &RecoveryBackupPack) -> Result<()> {
    if pack.files.len() != 4 {
        anyhow::bail!(
            "signer helper returned {} recovery backup files, but this desktop build requires exactly four; rebuild the bundled signer helper to match the desktop app",
            pack.files.len()
        );
    }

    let mut names = BTreeSet::new();
    for file in &pack.files {
        let name = recovery_backup_file_name(file);
        if !names.insert(name.clone()) {
            anyhow::bail!(
                "signer helper returned a recovery pack that maps multiple backup files to {name}; rebuild the bundled signer helper to match the desktop app"
            );
        }
    }

    Ok(())
}

pub(crate) fn recovery_backup_set_out_dir(
    parent_dir: &Path,
    pack: &RecoveryBackupPack,
) -> Result<PathBuf> {
    create_private_dir_all(parent_dir).with_context(|| {
        format!(
            "failed to create recovery parent directory {}",
            parent_dir.display()
        )
    })?;

    let prefix = recovery_backup_set_dir_name(pack);
    for index in 0..1000 {
        let dir_name = if index == 0 {
            prefix.clone()
        } else {
            format!("{prefix}-{}", index + 1)
        };
        let candidate = parent_dir.join(dir_name);
        match create_private_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "failed to create recovery backup folder {}: {error}",
                    candidate.display()
                ));
            }
        }
    }

    anyhow::bail!(
        "failed to choose a new recovery backup folder under {}",
        parent_dir.display()
    )
}

pub(crate) fn recovery_backup_set_dir_name(pack: &RecoveryBackupPack) -> String {
    let backup_set_id = pack
        .manifest
        .backup_set_id
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .take(12)
        .collect::<String>()
        .to_ascii_lowercase();
    let backup_set_id = if backup_set_id.is_empty() {
        "unknown".to_owned()
    } else {
        backup_set_id
    };
    format!(
        "framkey-backup-g{}-{}",
        pack.manifest.generation, backup_set_id
    )
}

pub(crate) fn write_recovery_backup_pack_files(
    out_dir: &Path,
    pack: &RecoveryBackupPack,
    encrypted_vault_backup: Option<&[u8]>,
    created_paths: &mut Vec<PathBuf>,
) -> Result<Value> {
    let encrypted_vault_backup = encrypted_vault_backup
        .ok_or_else(|| anyhow::anyhow!("encrypted vault backup bytes are required"))?;
    let mut artifacts = Vec::new();
    for file in &pack.files {
        let path = out_dir.join(recovery_backup_file_name(file));
        let bundle = RecoveryBackupBundle::new(file.clone(), encrypted_vault_backup);
        let bytes = serde_json::to_vec(&bundle)?;
        write_new_file_tracked(&path, &bytes, created_paths)?;
        artifacts.push(RecoveryBackupArtifactSummary {
            kind: "bundle",
            path: path.display().to_string(),
            blake3: encode_hex(blake3::hash(&bytes).as_bytes()),
            group: Some(file.group_kind.as_str().to_owned()),
            member: Some(file.member_label.clone()),
            destination: recovery_backup_destination(file.group_kind.as_str(), &file.member_label)
                .to_owned(),
            contains_secret_bytes: true,
        });
    }

    let files = artifacts
        .iter()
        .map(recovery_artifact_summary_json)
        .collect::<Vec<_>>();

    Ok(json!({
        "outDir": out_dir.display().to_string(),
        "backupSetId": pack.manifest.backup_set_id,
        "policyId": pack.manifest.policy_id,
        "walletId": pack.manifest.wallet_id,
        "generation": pack.manifest.generation,
        "shareFileCount": pack.files.len(),
        "backupFileCount": pack.files.len(),
        "bundleFileCount": pack.files.len(),
        "embeddedVaultBackupCount": pack.files.len(),
        "files": files,
        "cloudAloneRecovers": false,
    }))
}

pub(crate) fn cleanup_recovery_backup_pack(
    out_dir: &Path,
    pack: &RecoveryBackupPack,
) -> Result<()> {
    let paths = pack
        .files
        .iter()
        .map(|file| out_dir.join(recovery_backup_file_name(file)))
        .collect::<Vec<_>>();
    cleanup_paths(&paths)
}

pub(crate) fn cleanup_paths(paths: &[PathBuf]) -> Result<()> {
    let mut first_error = None;
    for path in paths {
        if let Err(error) = std::fs::remove_file(path)
            && error.kind() != std::io::ErrorKind::NotFound
            && first_error.is_none()
        {
            first_error = Some(anyhow::anyhow!(
                "failed to remove {}: {error}",
                path.display()
            ));
        }
    }

    if let Some(error) = first_error {
        Err(error)
    } else {
        Ok(())
    }
}

pub(crate) fn write_new_file_tracked(
    path: &Path,
    bytes: &[u8],
    created_paths: &mut Vec<PathBuf>,
) -> Result<()> {
    write_new_file(path, bytes)?;
    created_paths.push(path.to_path_buf());
    Ok(())
}

pub(crate) fn write_new_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(PRIVATE_FILE_MODE);
    let mut file = options
        .open(path)
        .map_err(|error| anyhow::anyhow!("failed to create {}: {error}", path.display()))?;
    set_private_file_permissions(path)?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}

pub(crate) fn recovery_artifact_summary_json(artifact: &RecoveryBackupArtifactSummary) -> Value {
    let mut value = json!({
        "kind": artifact.kind,
        "path": artifact.path,
        "blake3": artifact.blake3,
        "destination": artifact.destination,
    });
    if let Value::Object(ref mut object) = value {
        if let Some(group) = &artifact.group {
            object.insert("group".to_owned(), json!(group));
        }
        if let Some(member) = &artifact.member {
            object.insert("member".to_owned(), json!(member));
        }
        if artifact.kind == "share" || artifact.kind == "bundle" {
            object.insert("shareBytesPrinted".to_owned(), json!(false));
            if artifact.kind == "bundle" {
                object.insert("encryptedVaultData".to_owned(), json!("embedded"));
            }
        } else {
            object.insert(
                "containsSecretBytes".to_owned(),
                json!(artifact.contains_secret_bytes),
            );
        }
    }
    value
}

pub(crate) fn recovery_backup_destination(group: &str, member: &str) -> &'static str {
    let member = member.to_ascii_lowercase();
    if member.contains("icloud") {
        "Upload to iCloud Drive"
    } else if member.contains("google") {
        "Upload to Google Drive"
    } else if member.contains("local") {
        "Copy to local physical storage"
    } else if member.contains("off-site") {
        "Store off-site away from this Mac and GBA card"
    } else if group == "cloud" {
        "Upload to the named cloud provider"
    } else {
        "Store according to this share label"
    }
}

pub(crate) fn recovery_out_dir_path(value: &str) -> Result<PathBuf> {
    let value = value.trim();
    if value.is_empty() || value.chars().any(char::is_control) {
        anyhow::bail!("recovery output directory is malformed");
    }
    user_path(value)
}

pub(crate) fn user_path(value: &str) -> Result<PathBuf> {
    if value == "~" || value.starts_with("~/") {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| anyhow::anyhow!("HOME is required to expand user path"))?;
        if value == "~" {
            return Ok(PathBuf::from(home));
        }
        return Ok(PathBuf::from(home).join(&value[2..]));
    }
    Ok(PathBuf::from(value))
}

pub(crate) fn read_recovery_backup_files(paths: &[PathBuf]) -> Result<Vec<RecoveryBackupFile>> {
    if paths.is_empty() {
        anyhow::bail!("at least one recovery backup file is required");
    }
    if paths.len() > 4 {
        anyhow::bail!("standard recovery accepts at most four backup files");
    }

    paths
        .iter()
        .map(|path| {
            let bytes = std::fs::read(path)
                .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path.display()))?;
            parse_recovery_backup_bundle(&bytes)
                .map(|bundle| bundle.recovery_file)
                .map_err(|error| {
                    anyhow::anyhow!(
                        "failed to parse recovery backup {}: {error}",
                        path.display()
                    )
                })
        })
        .collect()
}

pub(crate) fn read_encrypted_vault_backup_from_bundle(path: &Path) -> Result<Vec<u8>> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("failed to read recovery backup {}", path.display()))?;
    let bundle = parse_recovery_backup_bundle(&bytes).map_err(|error| {
        anyhow::anyhow!(
            "failed to parse recovery backup {}: {error}",
            path.display()
        )
    })?;
    bundle.encrypted_vault_backup_bytes().map_err(|error| {
        anyhow::anyhow!(
            "failed to read encrypted vault data from {}: {error}",
            path.display()
        )
    })
}

pub(crate) fn validate_vault_image_size(size: usize) -> Result<()> {
    if size < MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES {
        anyhow::bail!(
            "configured save target is too small for a vault: {} bytes, minimum {} bytes",
            size,
            MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES
        );
    }
    if size > MAX_SIGNER_HELPER_SAVE_IMAGE_BYTES {
        anyhow::bail!(
            "configured save target is too large for current signer helper limits: {} bytes, maximum {} bytes",
            size,
            MAX_SIGNER_HELPER_SAVE_IMAGE_BYTES
        );
    }
    Ok(())
}
