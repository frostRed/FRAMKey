use framkey_core::{FramkeyError, Result};

pub fn random_array<const N: usize>() -> Result<[u8; N]> {
    let mut bytes = [0_u8; N];
    getrandom::fill(&mut bytes)
        .map_err(|error| FramkeyError::Io(std::io::Error::other(error.to_string())))?;
    Ok(bytes)
}
