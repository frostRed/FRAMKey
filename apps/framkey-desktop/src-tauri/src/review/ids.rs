use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use framkey_crypto::encode_hex;

pub(crate) fn new_broker_session_id() -> String {
    match new_random_hex::<8>() {
        Ok(random) => format!("broker-{random}"),
        Err(_) => format!("broker-{}", now_unix_ms()),
    }
}

pub(crate) fn new_decision_token() -> Result<String> {
    new_random_hex::<16>()
}

pub(crate) fn new_random_hex<const N: usize>() -> Result<String> {
    let mut bytes = [0_u8; N];
    getrandom::fill(&mut bytes)
        .map_err(|error| anyhow::anyhow!("failed to create review decision token: {error:?}"))?;
    Ok(encode_hex(&bytes))
}

pub(crate) fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().try_into().unwrap_or(u64::MAX))
        .unwrap_or(0)
}
