mod handler;
mod io;
mod metadata;
mod recovery;
mod validation;

use framkey_ipc::{IpcError, SignerHelperResponse};

fn main() {
    if let Err(error) = handler::run() {
        let response = SignerHelperResponse::error(IpcError {
            code: io::classify_error(&error),
            message: error.to_string(),
        });
        let _ = io::write_json_response(&response);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests;
