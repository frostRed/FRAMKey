mod config;
mod constants;
mod error;
mod handler;
mod signer_helper;

use anyhow::Result;
use framkey_ipc::{
    IpcErrorCode, IpcRequest, IpcResponse, read_native_message, write_native_message,
};

use crate::config::NativeHostConfig;
use crate::handler::handle_request;

fn main() -> Result<()> {
    let config = NativeHostConfig::load()?;
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    while let Some(payload) = read_native_message(&mut stdin)? {
        let response = match serde_json::from_slice::<IpcRequest>(&payload) {
            Ok(request) => handle_request(&config, request),
            Err(error) => IpcResponse::error(
                "invalid",
                IpcErrorCode::UnsupportedMethod,
                format!("invalid native request JSON: {error}"),
            ),
        };
        let response_payload = serde_json::to_vec(&response)?;
        write_native_message(&mut stdout, &response_payload)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests;
