use std::time::Duration;

use anyhow::{Context, Result};
use serde_json::{Value, json};
use tauri::{
    Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder, webview::PageLoadEvent,
};

use crate::*;

pub(crate) fn run_dapp_compatibility_check(
    app: &tauri::AppHandle,
    request: &NormalizedDappCompatibilityCheckRequest,
) -> Result<Value> {
    let dapp = app
        .get_webview_window("dapp")
        .ok_or_else(|| anyhow::anyhow!("dApp WebView is not open"))?;
    let mode_json =
        serde_json::to_string(request.mode).context("failed to encode compatibility check mode")?;
    let script = format!(
        r#"
(() => {{
  if (typeof window.framkeyRunProviderSmoke !== "function") {{
    throw new Error("FRAMKey provider smoke runner is not available on this page");
  }}
  Promise.resolve(window.framkeyRunProviderSmoke({{ mode: {mode_json} }}))
    .catch((error) => console.error("FRAMKey compatibility check failed", error));
}})();
"#
    );
    dapp.eval(&script)
        .context("failed to start dApp compatibility check")?;
    Ok(json!({
        "started": true,
        "mode": request.mode,
        "window": "dapp",
        "readOnly": request.mode == "read",
    }))
}

#[tauri::command]
pub(crate) fn open_dapp_webview(
    window: WebviewWindow,
    app: tauri::AppHandle,
    url: Option<String>,
) -> ProviderEnvelope {
    match ensure_trusted_window(&window)
        .and_then(|()| open_dapp_window(&app, url.as_deref()))
        .and_then(|()| {
            let state = app.state::<AppState>();
            Ok(json!({
                "opened": true,
                "state": state.dapp_session_snapshot()?,
            }))
        }) {
        Ok(result) => ProviderEnvelope::result("open_dapp_webview", result),
        Err(error) => ProviderEnvelope::error("open_dapp_webview", error_to_provider_error(error)),
    }
}

pub(crate) fn startup_dapp_target() -> Option<String> {
    startup_dapp_target_from_options(
        env_string("FRAMKEY_DESKTOP_START_URL"),
        env_string("FRAMKEY_DESKTOP_START_DAPP"),
        env_string("FRAMKEY_DESKTOP_DAPP_URL"),
        runtime_smoke_enabled(),
        remote_provider_smoke_mode().is_some(),
    )
}

pub(crate) fn startup_dapp_target_from_options(
    start_url: Option<String>,
    start_dapp: Option<String>,
    dapp_url: Option<String>,
    runtime_smoke: bool,
    remote_provider_smoke: bool,
) -> Option<String> {
    start_url
        .or(start_dapp)
        .or(dapp_url)
        .or_else(|| (runtime_smoke || remote_provider_smoke).then(|| "local".to_owned()))
}

pub(crate) fn ensure_main_window(app: &tauri::AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        debug_one_window("main_existing", &window);
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("FRAMKey")
        .inner_size(1160.0, 760.0)
        .min_inner_size(940.0, 620.0)
        .devtools(desktop_devtools_enabled())
        .build()?;
    let _ = window.show();
    let _ = window.set_focus();
    debug_one_window("main_built", &window);
    Ok(())
}

pub(crate) fn open_dapp_window(app: &tauri::AppHandle, target_url: Option<&str>) -> Result<()> {
    let target = dapp_webview_url(target_url)?;
    let target_state = dapp_session_target(target_url)?;
    let state = app.state::<AppState>();
    state.remember_dapp_open_request(target_state)?;
    if let Some(window) = app.get_webview_window("dapp") {
        match target {
            WebviewUrl::External(url) | WebviewUrl::CustomProtocol(url) => {
                window.navigate(url)?;
                let _ = window.show();
                let _ = window.set_focus();
                debug_one_window("dapp_navigated", &window);
            }
            WebviewUrl::App(_) => {
                window.destroy()?;
                build_dapp_window(app, target)?;
            }
            _ => anyhow::bail!("unsupported dApp WebView URL target"),
        }
        return Ok(());
    }

    build_dapp_window(app, target)
}

pub(crate) fn navigate_dapp_window(
    app: &tauri::AppHandle,
    action: DappNavigationAction,
) -> Result<Value> {
    if action == DappNavigationAction::Home {
        open_dapp_window(app, Some("local"))?;
    } else {
        let dapp = app
            .get_webview_window("dapp")
            .ok_or_else(|| anyhow::anyhow!("dApp WebView is not open"))?;
        dapp.eval(action.script())
            .with_context(|| format!("failed to request dApp {} navigation", action.as_str()))?;
        let state = app.state::<AppState>();
        state.remember_dapp_navigation_action(action)?;
    }
    let state = app.state::<AppState>();
    Ok(json!({
        "action": action.as_str(),
        "state": state.dapp_session_snapshot()?,
    }))
}

pub(crate) fn build_dapp_window(app: &tauri::AppHandle, target: WebviewUrl) -> Result<()> {
    let initialization_script = provider_initialization_script();
    let navigation_app = app.clone();
    let load_app = app.clone();
    WebviewWindowBuilder::new(app, "dapp", target)
        .title("FRAMKey dApp WebView")
        .inner_size(1080.0, 720.0)
        .min_inner_size(760.0, 520.0)
        .devtools(desktop_devtools_enabled())
        .initialization_script(&initialization_script)
        .on_navigation(move |url| {
            let state = navigation_app.state::<AppState>();
            if let Err(error) = state.remember_dapp_navigation_url(url.as_str()) {
                eprintln!("framkey_dapp_navigation_state_error {error}");
            }
            if provider_event_stderr_enabled() || window_smoke_enabled() {
                let url = sanitize_provider_event_url(url.as_str())
                    .unwrap_or_else(|_| "<malformed>".to_owned());
                eprintln!("framkey_dapp_navigation url={url}");
            }
            true
        })
        .on_page_load(move |window, payload| {
            let event = match payload.event() {
                PageLoadEvent::Started => "started",
                PageLoadEvent::Finished => "finished",
            };
            let state = load_app.state::<AppState>();
            if let Err(error) = state.remember_dapp_page_load(event, payload.url().as_str()) {
                eprintln!("framkey_dapp_page_load_state_error {error}");
            }
            if provider_event_stderr_enabled() || window_smoke_enabled() {
                let url = sanitize_provider_event_url(payload.url().as_str())
                    .unwrap_or_else(|_| "<malformed>".to_owned());
                eprintln!(
                    "framkey_dapp_page_load window={} event={} url={}",
                    window.label(),
                    event,
                    url,
                );
            }
        })
        .build()
        .map(|window| {
            let _ = window.show();
            let _ = window.set_focus();
            debug_one_window("dapp_built", &window);
        })?;
    Ok(())
}

pub(crate) fn schedule_delayed_window_smoke(app: tauri::AppHandle) {
    if !window_smoke_enabled() {
        return;
    }
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1_000));
        debug_window_state(&app, "delayed_1s");
        std::thread::sleep(Duration::from_millis(2_000));
        debug_window_state(&app, "delayed_3s");
    });
}

pub(crate) fn debug_window_state(app: &tauri::AppHandle, label: &str) {
    if !window_smoke_enabled() {
        return;
    }
    let labels = ["main", "dapp"];
    for window_label in labels {
        match app.get_webview_window(window_label) {
            Some(window) => debug_one_window(&format!("{label}_{window_label}"), &window),
            None => eprintln!("framkey_window_smoke label={label} window={window_label} missing"),
        }
    }
}

pub(crate) fn debug_one_window(label: &str, window: &WebviewWindow) {
    if !window_smoke_enabled() {
        return;
    }
    let visible = window.is_visible().ok();
    let focused = window.is_focused().ok();
    let minimized = window.is_minimized().ok();
    let inner_size = window
        .inner_size()
        .ok()
        .map(|size| format!("{}x{}", size.width, size.height))
        .unwrap_or_else(|| "unknown".to_owned());
    let position = window
        .outer_position()
        .ok()
        .map(|position| format!("{},{}", position.x, position.y))
        .unwrap_or_else(|| "unknown".to_owned());
    eprintln!(
        "framkey_window_smoke label={label} window={} visible={visible:?} focused={focused:?} minimized={minimized:?} inner_size={inner_size} position={position}",
        window.label(),
    );
}

pub(crate) fn window_smoke_enabled() -> bool {
    std::env::var_os("FRAMKEY_DESKTOP_WINDOW_SMOKE").is_some() || runtime_smoke_enabled()
}

pub(crate) fn runtime_smoke_enabled() -> bool {
    std::env::var_os("FRAMKEY_DESKTOP_AUTOSMOKE").is_some()
}

pub(crate) fn trusted_autosmoke_enabled() -> bool {
    runtime_smoke_enabled() || env_truthy("FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE")
}

pub(crate) fn recovery_autosmoke_enabled() -> bool {
    env_truthy("FRAMKEY_DESKTOP_RECOVERY_AUTOSMOKE")
}

pub(crate) fn desktop_devtools_enabled() -> bool {
    desktop_devtools_enabled_from_value(env_string("FRAMKEY_DESKTOP_DEVTOOLS").as_deref())
}

pub(crate) fn desktop_devtools_enabled_from_value(value: Option<&str>) -> bool {
    cfg!(debug_assertions) && matches!(value.map(str::trim), Some("1" | "true" | "yes" | "on"))
}

pub(crate) fn wallet_send_autosmoke_enabled() -> bool {
    env_truthy("FRAMKEY_DESKTOP_WALLET_SEND_AUTOSMOKE")
}

pub(crate) fn trusted_autosmoke_duration_ms() -> u64 {
    pub(crate) const RUNTIME_SMOKE_DURATION_MS: u64 = 20_000;
    pub(crate) const TRUSTED_SMOKE_DURATION_MS: u64 = 45_000;
    pub(crate) const MIN_DURATION_MS: u64 = 1_000;
    pub(crate) const MAX_DURATION_MS: u64 = 300_000;

    env_string("FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE_DURATION_MS")
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|duration| (MIN_DURATION_MS..=MAX_DURATION_MS).contains(duration))
        .unwrap_or_else(|| {
            if runtime_smoke_enabled() {
                RUNTIME_SMOKE_DURATION_MS
            } else {
                TRUSTED_SMOKE_DURATION_MS
            }
        })
}

pub(crate) fn remote_provider_smoke_mode() -> Option<&'static str> {
    let value = env_string("FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE")?;
    match value.trim().to_ascii_lowercase().as_str() {
        "0" | "false" | "no" | "off" => None,
        "interactive" | "full" | "write" | "sign" => Some("interactive"),
        _ => Some("read"),
    }
}

pub(crate) fn remote_provider_smoke_chain_id() -> Option<String> {
    env_string("FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE_CHAIN_ID")
        .or_else(|| env_string("FRAMKEY_DESKTOP_REMOTE_PROVIDER_SMOKE_SWITCH_CHAIN_ID"))
        .and_then(|value| normalize_chain_id(value.trim()).ok())
}

pub(crate) fn provider_event_stderr_enabled() -> bool {
    std::env::var_os("FRAMKEY_DESKTOP_PROVIDER_TELEMETRY_STDERR").is_some()
        || runtime_smoke_enabled()
}

pub(crate) fn print_provider_event_if_enabled(event: &ProviderEvent) {
    if !provider_event_stderr_enabled() {
        return;
    }
    eprintln!(
        "framkey_provider_event seq={} kind={} status={} window={} origin={} url={} method={} result={} error={} duration_ms={}",
        event.sequence,
        event.kind,
        event.status,
        event.window.as_deref().unwrap_or("-"),
        event.origin.as_deref().unwrap_or("-"),
        event.url.as_deref().unwrap_or("-"),
        event.method.as_deref().unwrap_or("-"),
        event.result_kind.as_deref().unwrap_or("-"),
        provider_event_error_label(event),
        event
            .duration_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_owned()),
    );
}

pub(crate) fn provider_event_error_label(event: &ProviderEvent) -> String {
    match (event.error_code, event.error_message.as_deref()) {
        (Some(code), Some(message)) => format!("{code}:{message}"),
        (Some(code), None) => code.to_string(),
        (None, Some(message)) => message.to_owned(),
        (None, None) => "-".to_owned(),
    }
}

pub(crate) fn provider_initialization_script() -> String {
    let mut script = include_str!("provider-injection.js").to_owned();
    if runtime_smoke_enabled() {
        script.push_str("\nwindow.__FRAMKEY_AUTOSMOKE__ = true;\n");
    }
    if let Some(mode) = remote_provider_smoke_mode() {
        let mode_json = serde_json::to_string(mode).expect("remote provider smoke mode serializes");
        script.push_str("\nwindow.__FRAMKEY_REMOTE_PROVIDER_SMOKE__ = ");
        script.push_str(&mode_json);
        script.push_str(";\n");
        if let Some(chain_id) = remote_provider_smoke_chain_id() {
            let chain_id_json =
                serde_json::to_string(&chain_id).expect("remote provider smoke chain serializes");
            script.push_str("window.__FRAMKEY_REMOTE_PROVIDER_SMOKE_CHAIN_ID__ = ");
            script.push_str(&chain_id_json);
            script.push_str(";\n");
        }
    }
    script
}

pub(crate) fn dapp_webview_url(target_url: Option<&str>) -> Result<WebviewUrl> {
    let Some(target_url) = target_url.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(WebviewUrl::App("dapp.html".into()));
    };
    if target_url == "local" || target_url == "framkey://local-dapp" {
        return Ok(WebviewUrl::App("dapp.html".into()));
    }
    if target_url == "uniswap" {
        return Ok(WebviewUrl::External(
            Url::parse(UNISWAP_URL).expect("constant Uniswap URL is valid"),
        ));
    }
    if target_url == "aave" {
        return Ok(WebviewUrl::External(
            Url::parse(AAVE_URL).expect("constant Aave URL is valid"),
        ));
    }
    if target_url.len() > 2048 || target_url.chars().any(char::is_control) {
        anyhow::bail!("dApp URL is malformed");
    }
    let url = Url::parse(target_url).context("invalid dApp URL")?;
    match url.scheme() {
        "http" | "https" => Ok(WebviewUrl::External(url)),
        _ => anyhow::bail!("dApp URL must use http or https"),
    }
}

pub(crate) fn dapp_session_target(target_url: Option<&str>) -> Result<DappSessionTarget> {
    let Some(target_url) = target_url.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(DappSessionTarget {
            label: "Local Test".to_owned(),
            url: Some(LOCAL_DAPP_URL.to_owned()),
            origin: Some(LOCAL_DAPP_ORIGIN.to_owned()),
        });
    };
    if target_url == "local" || target_url == "framkey://local-dapp" {
        return Ok(DappSessionTarget {
            label: "Local Test".to_owned(),
            url: Some(LOCAL_DAPP_URL.to_owned()),
            origin: Some(LOCAL_DAPP_ORIGIN.to_owned()),
        });
    }
    if target_url == "uniswap" {
        let location = dapp_session_location(UNISWAP_URL)?;
        return Ok(DappSessionTarget {
            label: "Uniswap".to_owned(),
            url: location.url,
            origin: location.origin,
        });
    }
    if target_url == "aave" {
        let location = dapp_session_location(AAVE_URL)?;
        return Ok(DappSessionTarget {
            label: "Aave".to_owned(),
            url: location.url,
            origin: location.origin,
        });
    }
    let location = dapp_session_location(target_url)?;
    let label = location
        .origin
        .as_deref()
        .and_then(|origin| Url::parse(origin).ok())
        .and_then(|url| url.host_str().map(str::to_owned))
        .or_else(|| location.origin.clone())
        .unwrap_or_else(|| "Custom dApp".to_owned());
    Ok(DappSessionTarget {
        label,
        url: location.url,
        origin: location.origin,
    })
}

pub(crate) fn dapp_session_location(value: &str) -> Result<DappSessionLocation> {
    let sanitized = sanitize_provider_event_url(value)?;
    let origin = Url::parse(&sanitized)
        .ok()
        .and_then(|url| dapp_origin_from_url(&url));
    Ok(DappSessionLocation {
        url: Some(sanitized),
        origin,
    })
}

pub(crate) fn dapp_origin_from_url(url: &Url) -> Option<String> {
    let scheme = url.scheme();
    match scheme {
        "http" | "https" | "tauri" => {
            let host = url.host_str()?;
            let host = match url.port() {
                Some(port) => format!("{host}:{port}"),
                None => host.to_owned(),
            };
            Some(format!("{scheme}://{host}"))
        }
        "framkey" => Some("framkey://local-dapp".to_owned()),
        _ => None,
    }
}

pub(crate) fn ensure_trusted_window(window: &WebviewWindow) -> Result<()> {
    if window.label() != "main" {
        anyhow::bail!(
            "review broker command is restricted to trusted main window, got {}",
            window.label()
        );
    }
    Ok(())
}
