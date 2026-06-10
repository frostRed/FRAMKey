use std::{env, io::Write, path::PathBuf};

use anyhow::{Context, Result};
use framkey_ch347_helper::{
    Ch347HelperResponse, error_response, execute_request, read_request_file, response_json_bytes,
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
    let payload = response_json_bytes(&response)?;
    std::io::stdout().write_all(&payload)?;
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
}

impl HelperArgs {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut request = None;
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--request" => {
                    request = Some(PathBuf::from(
                        args.next().context("--request requires a path")?,
                    ));
                }
                "--response" => {
                    anyhow::bail!(
                        "--response is no longer supported; CH347 helper responses are written to stdout"
                    );
                }
                "--help" | "-h" => {
                    anyhow::bail!("usage: framkey-ch347-helper --request <path>");
                }
                _ => anyhow::bail!("unsupported argument {arg}"),
            }
        }
        Ok(Self {
            request: request.context("--request is required")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_args_reject_response_path() {
        let error = HelperArgs::parse([
            "--request".to_owned(),
            "/tmp/request.json".to_owned(),
            "--response".to_owned(),
            "/tmp/response.json".to_owned(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("--response is no longer supported"));
    }

    #[test]
    fn helper_args_accept_request_only() {
        let args =
            HelperArgs::parse(["--request".to_owned(), "/tmp/request.json".to_owned()]).unwrap();

        assert_eq!(args.request, PathBuf::from("/tmp/request.json"));
    }
}
