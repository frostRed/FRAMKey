use std::{env, path::PathBuf};

use anyhow::{Context, Result};
use framkey_ch347_helper::{
    Ch347HelperResponse, error_response, execute_request, read_request_file, write_response_file,
};

fn main() {
    if let Err(error) = run_cli() {
        eprintln!("framkey-ch347-helper failed: {error}");
        std::process::exit(1);
    }
}

fn run_cli() -> Result<()> {
    let args = HelperArgs::parse(env::args().skip(1))?;
    let response = match read_request_file(&args.request).and_then(execute_request) {
        Ok(result) => Ch347HelperResponse::ok(result),
        Err(error) => error_response(&error),
    };
    write_response_file(&args.response, &response)?;
    match response {
        Ch347HelperResponse::Ok { .. } => Ok(()),
        Ch347HelperResponse::Error { error } => {
            anyhow::bail!("CH347 helper failed: {}: {}", error.code, error.message)
        }
    }
}

#[derive(Debug, Clone)]
struct HelperArgs {
    request: PathBuf,
    response: PathBuf,
}

impl HelperArgs {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut request = None;
        let mut response = None;
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--request" => {
                    request = Some(PathBuf::from(
                        args.next().context("--request requires a path")?,
                    ));
                }
                "--response" => {
                    response = Some(PathBuf::from(
                        args.next().context("--response requires a path")?,
                    ));
                }
                "--help" | "-h" => {
                    anyhow::bail!("usage: framkey-ch347-helper --request <path> --response <path>");
                }
                _ => anyhow::bail!("unsupported argument {arg}"),
            }
        }
        Ok(Self {
            request: request.context("--request is required")?,
            response: response.context("--response is required")?,
        })
    }
}
