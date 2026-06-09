use std::time::Duration;

pub(crate) const DEFAULT_HOST_NAME: &str = "dev.framkey.native_host";
pub(crate) const DEFAULT_KEYCHAIN_SERVICE: &str = "io.framkey.local-kek";
pub(crate) const DEFAULT_KEYCHAIN_ACCOUNT: &str = "default";
pub(crate) const DEFAULT_CHAIN_ID: &str = "0x1";
pub(crate) const MACOS_NO_NETWORK_SANDBOX_PROFILE: &str =
    "(version 1) (allow default) (deny network*)";
pub(crate) const SIGNER_HELPER_TIMEOUT: Duration = Duration::from_secs(45);
