use std::time::Duration;

use framkey_keychain_macos::KeychainAccessPolicy;

pub(crate) const DEFAULT_KEYCHAIN_SERVICE: &str = "io.framkey.local-kek";
pub(crate) const DEFAULT_KEYCHAIN_ACCOUNT: &str = "default";
pub(crate) const FRAMKEY_SIGNER_HELPER_BLAKE3_ENV: &str = "FRAMKEY_SIGNER_HELPER_BLAKE3";
pub(crate) const MACOS_NO_NETWORK_SANDBOX_PROFILE: &str =
    "(version 1) (allow default) (deny network*)";
pub(crate) const SIGNER_HELPER_TIMEOUT: Duration = Duration::from_secs(45);
pub(crate) const DEFAULT_KEYCHAIN_ACCESS_POLICY: KeychainAccessPolicy =
    KeychainAccessPolicy::LocalDeviceOwnerAuthentication;
