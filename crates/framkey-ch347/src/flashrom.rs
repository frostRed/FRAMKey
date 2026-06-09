use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use framkey_core::{FramkeyError, Result};

use crate::Ch347Config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlashromProbeReport {
    pub(crate) chip_name: Option<String>,
    pub(crate) size_bytes: Option<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct FlashromTool {
    program: PathBuf,
    programmer_arg: String,
    chip: Option<String>,
}

impl FlashromTool {
    pub(crate) fn new(config: &Ch347Config) -> Result<Self> {
        Ok(Self {
            program: config.flashrom_program(),
            programmer_arg: config.programmer_arg(),
            chip: config.chip_name()?.map(str::to_owned),
        })
    }

    pub(crate) fn probe(&self) -> Result<FlashromProbeReport> {
        let output = self.run("probe", self.args_for(FlashromOperation::Probe))?;
        Ok(parse_flashrom_probe_output(&output))
    }

    pub(crate) fn read_to(&self, out: &Path) -> Result<()> {
        self.run("read", self.args_for(FlashromOperation::Read(out)))?;
        Ok(())
    }

    pub(crate) fn write_from(&self, input: &Path) -> Result<()> {
        self.run("write", self.args_for(FlashromOperation::Write(input)))?;
        Ok(())
    }

    pub(crate) fn args_for(&self, operation: FlashromOperation<'_>) -> Vec<OsString> {
        let mut args = vec![OsString::from("-p"), OsString::from(&self.programmer_arg)];

        if let Some(chip) = &self.chip {
            args.push(OsString::from("-c"));
            args.push(OsString::from(chip));
        }

        match operation {
            FlashromOperation::Probe => {}
            FlashromOperation::Read(path) => {
                args.push(OsString::from("-r"));
                args.push(path.as_os_str().to_os_string());
            }
            FlashromOperation::Write(path) => {
                args.push(OsString::from("-w"));
                args.push(path.as_os_str().to_os_string());
            }
        }

        args
    }

    fn run(&self, action: &str, args: Vec<OsString>) -> Result<Output> {
        let output = Command::new(&self.program)
            .args(&args)
            .output()
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    FramkeyError::unsupported(format!(
                        "flashrom was not found; install flashrom 1.4 or newer with CH347 support, or pass --flashrom <path> ({error})"
                    ))
                } else {
                    FramkeyError::unsupported(format!(
                        "failed to start flashrom for CH347 {action}: {error}"
                    ))
                }
            })?;

        if output.status.success() {
            return Ok(output);
        }

        Err(FramkeyError::unsupported(format!(
            "flashrom CH347 {action} failed with status {}; {}",
            output.status,
            output_excerpt(&output)
        )))
    }
}

pub(crate) fn parse_flashrom_probe_output(output: &Output) -> FlashromProbeReport {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");
    let chip_name = parse_flashrom_chip_name(&combined);
    let size_bytes = parse_flashrom_chip_size_bytes(&combined);
    FlashromProbeReport {
        chip_name,
        size_bytes,
    }
}

fn parse_flashrom_chip_name(text: &str) -> Option<String> {
    let line = text.lines().find(|line| line.contains("Found "))?;
    let start = line.find('"')?;
    let rest = &line[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

fn parse_flashrom_chip_size_bytes(text: &str) -> Option<usize> {
    for line in text.lines().filter(|line| line.contains("Found ")) {
        let mut rest = line;
        while let Some(start) = rest.find('(') {
            rest = &rest[start + 1..];
            let Some(end) = rest.find(')') else {
                break;
            };
            let segment = &rest[..end];
            if let Some(size) = parse_flashrom_size_segment(segment) {
                return Some(size);
            }
            rest = &rest[end + 1..];
        }
    }
    None
}

fn parse_flashrom_size_segment(segment: &str) -> Option<usize> {
    let first = segment.split(',').next()?.trim();
    let mut parts = first.split_whitespace();
    let value = parts.next()?.parse::<usize>().ok()?;
    let unit = parts.next()?.trim().to_ascii_lowercase();
    match unit.as_str() {
        "b" | "byte" | "bytes" => Some(value),
        "kb" | "kib" => value.checked_mul(1024),
        "mb" | "mib" => value.checked_mul(1024 * 1024),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum FlashromOperation<'a> {
    Probe,
    Read(&'a Path),
    Write(&'a Path),
}

fn output_excerpt(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("stdout: {}; stderr: {}", stdout.trim(), stderr.trim());
    let mut excerpt = combined.chars().take(1_200).collect::<String>();
    if combined.chars().count() > 1_200 {
        excerpt.push_str("...");
    }
    excerpt
}
