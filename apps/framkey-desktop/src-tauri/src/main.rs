mod btc;
mod ch347_helper;
mod chains;
mod commands;
mod config;
mod constants;
mod dapp;
mod paths;
mod provider;
mod recovery_ops;
mod review;
mod rollback;
mod session;
mod signer_runtime;
mod state;
mod transactions;
mod wallet;

pub(crate) use btc::*;
pub(crate) use ch347_helper::*;
pub(crate) use chains::*;
pub(crate) use commands::*;
pub(crate) use config::*;
pub(crate) use constants::*;
pub(crate) use dapp::*;
pub(crate) use paths::*;
pub(crate) use provider::*;
pub(crate) use recovery_ops::*;
pub(crate) use review::{
    ReviewDecision, ReviewQueue, ReviewRequest, ReviewStatus, dangerous_method_kind,
    network_switch_authorization,
};
pub(crate) use rollback::*;
pub(crate) use session::*;
pub(crate) use signer_runtime::*;
pub(crate) use state::*;
pub(crate) use transactions::*;
pub(crate) use wallet::*;

#[cfg(test)]
mod tests;

fn main() {
    tauri::Builder::default()
        .manage(AppState::load())
        .invoke_handler(tauri::generate_handler![
            framkey_provider_request,
            framkey_review_queue,
            framkey_transaction_activity,
            framkey_clear_transaction_activity,
            framkey_decide_review_request,
            framkey_dismiss_review_request,
            framkey_clear_review_queue,
            framkey_account_permissions,
            framkey_revoke_account_permission,
            framkey_disconnect_account,
            framkey_provider_telemetry,
            framkey_provider_events,
            framkey_clear_provider_events,
            framkey_run_dapp_compatibility_check,
            framkey_dapp_session,
            framkey_navigate_dapp,
            framkey_smoke_event,
            framkey_status,
            framkey_rpc_health,
            framkey_authorize_keychain_helper,
            framkey_wallet_assets,
            framkey_btc_balance,
            framkey_send_native_transfer,
            framkey_send_token_transfer,
            framkey_send_btc_transfer,
            framkey_switch_session_chain,
            framkey_recovery_state,
            framkey_clear_recovery_state,
            framkey_recovery_smoke_pack,
            framkey_create_keychain_vault,
            framkey_validate_recovery_set,
            framkey_recover_keychain_vault,
            framkey_pick_vault_backup_file,
            framkey_pick_physical_backup_file,
            framkey_pick_physical_backup_out_dir,
            framkey_write_ch347_backup,
            framkey_read_ch347_backup,
            framkey_pick_recovery_files,
            framkey_pick_recovery_out_dir,
            framkey_reveal_path,
            open_dapp_webview,
        ])
        .setup(|app| {
            app.set_activation_policy(tauri::ActivationPolicy::Regular);
            debug_window_state(app.handle(), "setup_start");
            ensure_main_window(app.handle())?;
            debug_window_state(app.handle(), "after_main");
            if let Some(startup_dapp) = startup_dapp_target() {
                open_dapp_window(app.handle(), Some(startup_dapp.as_str()))?;
                debug_window_state(app.handle(), "after_dapp");
            } else {
                debug_window_state(app.handle(), "dapp_not_started");
            }
            schedule_delayed_window_smoke(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run FRAMKey desktop app");
}
