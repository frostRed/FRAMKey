use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

fn main() {
    prepare_signer_helper_sidecar();
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "framkey_provider_request",
            "framkey_review_queue",
            "framkey_transaction_activity",
            "framkey_recovery_state",
            "framkey_clear_recovery_state",
            "framkey_decide_review_request",
            "framkey_dismiss_review_request",
            "framkey_clear_review_queue",
            "framkey_account_permissions",
            "framkey_revoke_account_permission",
            "framkey_provider_telemetry",
            "framkey_provider_events",
            "framkey_clear_provider_events",
            "framkey_smoke_event",
            "framkey_status",
            "framkey_wallet_assets",
            "framkey_create_keychain_vault",
            "framkey_validate_recovery_set",
            "framkey_recover_keychain_vault",
            "framkey_pick_vault_backup_file",
            "framkey_reveal_path",
            "open_dapp_webview",
        ]),
    ))
    .expect("failed to build FRAMKey Tauri app");
}

fn prepare_signer_helper_sidecar() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let target = env::var("TARGET").expect("TARGET");
    println!("cargo:rustc-env=FRAMKEY_BUILD_TARGET={target}");
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_owned());
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("desktop crate lives under apps/framkey-desktop/src-tauri");
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                repo_root.join(path)
            }
        })
        .unwrap_or_else(|| repo_root.join("target"));
    let helper_name = format!("framkey-signer-helper{}", env::consts::EXE_SUFFIX);
    let source = target_dir.join(&profile).join(&helper_name);
    let sidecar_dir = manifest_dir.join("binaries");
    let sidecar = sidecar_dir.join(format!(
        "framkey-signer-helper-{target}{}",
        env::consts::EXE_SUFFIX
    ));

    println!("cargo:rerun-if-changed={}", source.display());
    if let Err(error) = copy_if_present(&source, &sidecar_dir, &sidecar) {
        println!(
            "cargo:warning=failed to prepare signer helper sidecar from {}: {error}",
            source.display()
        );
    }
}

fn copy_if_present(source: &Path, sidecar_dir: &Path, sidecar: &Path) -> io::Result<()> {
    if !source.exists() {
        println!(
            "cargo:warning=signer helper sidecar source {} is missing; run `cargo build -p framkey-signer-helper` before bundling the desktop app",
            source.display()
        );
        return Ok(());
    }
    fs::create_dir_all(sidecar_dir)?;
    fs::copy(source, sidecar)?;
    let permissions = fs::metadata(source)?.permissions();
    fs::set_permissions(sidecar, permissions)?;
    Ok(())
}
