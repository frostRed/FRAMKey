#[cfg(target_os = "macos")]
mod imp {
    use framkey_core::{FramkeyError, Result};
    use framkey_crypto::{SecretBytes, random_array};
    use localauthentication::{LAContext, LAError, LAPolicy};
    use security_framework::base::Error as SecurityError;
    use security_framework::passwords::{
        PasswordOptions, delete_generic_password, delete_generic_password_options,
        generic_password, set_generic_password_options,
    };
    use security_framework_sys::base::errSecItemNotFound;

    use crate::types::{KeychainAccessPolicy, MacKeychainItem, MacKeychainKek, mac_keychain_kek};

    const KEYCHAIN_BLOB_MAGIC: [u8; 8] = *b"FRKKEK\0\0";
    const KEYCHAIN_BLOB_VERSION_LOCAL_AUTH_DOMAIN_STATE: u16 = 3;
    const KEYCHAIN_BLOB_POLICY_BIOMETRY_CURRENT_SET: u16 = 1;
    const KEYCHAIN_BLOB_POLICY_DEVICE_OWNER_AUTHENTICATION: u16 = 2;
    const KEYCHAIN_BLOB_MIN_HEADER_LEN: usize = 8 + 2 + 2;
    const KEYCHAIN_BLOB_AUTH_STATE_HASH_OFFSET: usize = KEYCHAIN_BLOB_MIN_HEADER_LEN;
    const KEYCHAIN_BLOB_KEK_OFFSET: usize = KEYCHAIN_BLOB_AUTH_STATE_HASH_OFFSET + 32;
    const KEYCHAIN_BLOB_LEN: usize = KEYCHAIN_BLOB_KEK_OFFSET + 32;

    pub fn load_kek(item: &MacKeychainItem) -> Result<SecretBytes<32>> {
        Ok(load_existing_kek(item)?.kek)
    }

    pub fn load_existing_kek(item: &MacKeychainItem) -> Result<MacKeychainKek> {
        item.validate()?;
        let auth_state_hash =
            authorize_local_kek_access(KeychainAccessPolicy::LocalBiometryCurrentSet)?;
        let bytes = load_keychain_bytes(item)
            .map_err(|error| map_security_error("load macOS Keychain KEK", error))?;
        let loaded = secret_from_keychain_blob(bytes, &auth_state_hash)?;
        Ok(mac_keychain_kek(
            item.clone(),
            false,
            loaded.policy,
            loaded.kek,
        ))
    }

    pub fn ensure_kek(
        item: &MacKeychainItem,
        policy: KeychainAccessPolicy,
    ) -> Result<MacKeychainKek> {
        item.validate()?;
        let auth_state_hash = authorize_local_kek_access(policy)?;

        match load_keychain_bytes(item) {
            Ok(bytes) => {
                let loaded = secret_from_keychain_blob(bytes, &auth_state_hash)?;
                store_kek(item, policy, &auth_state_hash, &loaded.kek)?;
                Ok(mac_keychain_kek(item.clone(), false, policy, loaded.kek))
            }
            Err(error) if error.code() == errSecItemNotFound => {
                let kek = SecretBytes::new(random_array::<32>()?);
                store_kek(item, policy, &auth_state_hash, &kek)?;
                Ok(mac_keychain_kek(item.clone(), true, policy, kek))
            }
            Err(error) => Err(map_security_error("load macOS Keychain KEK", error)),
        }
    }

    pub fn reset_kek(
        item: &MacKeychainItem,
        policy: KeychainAccessPolicy,
    ) -> Result<MacKeychainKek> {
        item.validate()?;
        let auth_state_hash = authorize_local_kek_access(policy)?;
        delete_local_keychain_item(item)?;
        delete_classic_keychain_item(item)?;
        let kek = SecretBytes::new(random_array::<32>()?);
        store_kek(item, policy, &auth_state_hash, &kek)?;
        Ok(mac_keychain_kek(item.clone(), true, policy, kek))
    }

    pub fn rebind_kek(
        item: &MacKeychainItem,
        policy: KeychainAccessPolicy,
    ) -> Result<MacKeychainKek> {
        item.validate()?;
        let auth_state_hash = authorize_local_kek_access(policy)?;
        let bytes = load_keychain_bytes(item)
            .map_err(|error| map_security_error("load macOS Keychain KEK", error))?;
        let loaded = secret_from_keychain_blob(bytes, &auth_state_hash)?;
        store_kek(item, policy, &auth_state_hash, &loaded.kek)?;
        Ok(mac_keychain_kek(item.clone(), false, policy, loaded.kek))
    }

    pub fn delete_kek(item: &MacKeychainItem) -> Result<bool> {
        item.validate()?;
        let local_deleted = delete_local_keychain_item(item)?;
        let classic_deleted = delete_classic_keychain_item(item)?;
        Ok(local_deleted || classic_deleted)
    }

    fn delete_local_keychain_item(item: &MacKeychainItem) -> Result<bool> {
        match delete_generic_password_options(local_keychain_options(item)) {
            Ok(()) => Ok(true),
            Err(error) if error.code() == errSecItemNotFound => Ok(false),
            Err(error) => Err(map_security_error("delete macOS local Keychain KEK", error)),
        }
    }

    fn delete_classic_keychain_item(item: &MacKeychainItem) -> Result<bool> {
        match delete_generic_password(&item.service, &item.account) {
            Ok(()) => Ok(true),
            Err(error) if error.code() == errSecItemNotFound => Ok(false),
            Err(error) => Err(map_security_error(
                "delete macOS classic Keychain KEK",
                error,
            )),
        }
    }

    fn load_keychain_bytes(item: &MacKeychainItem) -> std::result::Result<Vec<u8>, SecurityError> {
        generic_password(local_keychain_options(item))
    }

    struct ParsedKeychainBlob {
        policy: KeychainAccessPolicy,
        kek: SecretBytes<32>,
    }

    fn secret_from_keychain_blob(
        mut bytes: Vec<u8>,
        expected_auth_state_hash: &[u8; 32],
    ) -> Result<ParsedKeychainBlob> {
        let parsed = parse_keychain_blob(&bytes, expected_auth_state_hash);
        bytes.fill(0);
        parsed
    }

    fn store_kek(
        item: &MacKeychainItem,
        policy: KeychainAccessPolicy,
        auth_state_hash: &[u8; 32],
        kek: &SecretBytes<32>,
    ) -> Result<()> {
        let mut blob = keychain_blob(policy, auth_state_hash, kek);
        let result = set_generic_password_options(&blob, local_keychain_store_options(item))
            .map_err(|error| map_security_error("store macOS Keychain KEK", error));
        blob.fill(0);
        result
    }

    fn keychain_blob(
        policy: KeychainAccessPolicy,
        auth_state_hash: &[u8; 32],
        kek: &SecretBytes<32>,
    ) -> Vec<u8> {
        let policy_id = match policy {
            KeychainAccessPolicy::LocalDeviceOwnerAuthentication => {
                KEYCHAIN_BLOB_POLICY_DEVICE_OWNER_AUTHENTICATION
            }
            KeychainAccessPolicy::LocalBiometryCurrentSet => {
                KEYCHAIN_BLOB_POLICY_BIOMETRY_CURRENT_SET
            }
        };

        let mut blob = Vec::with_capacity(KEYCHAIN_BLOB_LEN);
        blob.extend_from_slice(&KEYCHAIN_BLOB_MAGIC);
        blob.extend_from_slice(&KEYCHAIN_BLOB_VERSION_LOCAL_AUTH_DOMAIN_STATE.to_le_bytes());
        blob.extend_from_slice(&policy_id.to_le_bytes());
        blob.extend_from_slice(auth_state_hash);
        blob.extend_from_slice(kek.expose());
        blob
    }

    fn keychain_blob_policy(bytes: &[u8]) -> Result<KeychainAccessPolicy> {
        if bytes.len() < KEYCHAIN_BLOB_MIN_HEADER_LEN {
            return Err(FramkeyError::invalid_data(format!(
                "macOS Keychain KEK blob is too short: {} bytes",
                bytes.len(),
            )));
        }
        if bytes[0..8] != KEYCHAIN_BLOB_MAGIC {
            return Err(FramkeyError::invalid_data(
                "macOS Keychain KEK blob magic mismatch",
            ));
        }

        let version = u16::from_le_bytes([bytes[8], bytes[9]]);
        if version != KEYCHAIN_BLOB_VERSION_LOCAL_AUTH_DOMAIN_STATE {
            return Err(FramkeyError::unsupported(format!(
                "macOS Keychain KEK blob version {version}; delete and recreate the local Keychain KEK"
            )));
        }
        if bytes.len() != KEYCHAIN_BLOB_LEN {
            return Err(FramkeyError::invalid_data(format!(
                "macOS Keychain KEK blob must be {KEYCHAIN_BLOB_LEN} bytes, got {}",
                bytes.len(),
            )));
        }

        let policy_id = u16::from_le_bytes([bytes[10], bytes[11]]);
        match policy_id {
            KEYCHAIN_BLOB_POLICY_BIOMETRY_CURRENT_SET => {
                Ok(KeychainAccessPolicy::LocalBiometryCurrentSet)
            }
            KEYCHAIN_BLOB_POLICY_DEVICE_OWNER_AUTHENTICATION => {
                Ok(KeychainAccessPolicy::LocalDeviceOwnerAuthentication)
            }
            _ => Err(FramkeyError::unsupported(format!(
                "macOS Keychain KEK policy id {policy_id}"
            ))),
        }
    }

    fn parse_keychain_blob(
        bytes: &[u8],
        expected_auth_state_hash: &[u8; 32],
    ) -> Result<ParsedKeychainBlob> {
        let policy = keychain_blob_policy(bytes)?;
        if &bytes[KEYCHAIN_BLOB_AUTH_STATE_HASH_OFFSET..KEYCHAIN_BLOB_KEK_OFFSET]
            != expected_auth_state_hash
        {
            return Err(FramkeyError::unsupported(
                "macOS local unlock binding changed; recover the vault to rebind this Mac",
            ));
        }
        Ok(ParsedKeychainBlob {
            policy,
            kek: SecretBytes::<32>::from_slice(
                &bytes[KEYCHAIN_BLOB_KEK_OFFSET..KEYCHAIN_BLOB_LEN],
            )?,
        })
    }

    fn local_keychain_options(item: &MacKeychainItem) -> PasswordOptions {
        let mut options = PasswordOptions::new_generic_password(&item.service, &item.account);
        options.set_access_synchronized(Some(false));
        options
    }

    fn local_keychain_store_options(item: &MacKeychainItem) -> PasswordOptions {
        let mut options = local_keychain_options(item);
        options.set_label("FRAMKey local KEK");
        options.set_description("Local FRAMKey Keychain encryption key");
        options
    }

    fn authorize_local_kek_access(policy: KeychainAccessPolicy) -> Result<[u8; 32]> {
        authorize_local_kek_access_inner(policy, true)
    }

    fn authorize_local_kek_access_inner(
        policy: KeychainAccessPolicy,
        allow_biometry_lockout_recovery: bool,
    ) -> Result<[u8; 32]> {
        let la_policy = match policy {
            KeychainAccessPolicy::LocalDeviceOwnerAuthentication => {
                LAPolicy::DeviceOwnerAuthentication
            }
            KeychainAccessPolicy::LocalBiometryCurrentSet => {
                LAPolicy::DeviceOwnerAuthenticationWithBiometrics
            }
        };

        let context = LAContext::new().map_err(|error| {
            map_local_auth_error("create macOS LocalAuthentication context", error)
        })?;
        match context.can_evaluate_policy(la_policy) {
            Ok(true) => {}
            Ok(false) => {
                return Err(FramkeyError::unsupported(format!(
                    "{} is not available for FRAMKey local KEK access",
                    local_auth_label(policy)
                )));
            }
            Err(error) => {
                if should_recover_biometry_lockout(policy, allow_biometry_lockout_recovery, &error)
                {
                    recover_biometry_lockout()?;
                    return authorize_local_kek_access_inner(policy, false);
                }
                return Err(map_local_auth_error(
                    "check macOS LocalAuthentication availability",
                    error,
                ));
            }
        }

        let authorized = match context.evaluate_policy(la_policy, "Unlock FRAMKey local KEK.") {
            Ok(authorized) => authorized,
            Err(error)
                if should_recover_biometry_lockout(
                    policy,
                    allow_biometry_lockout_recovery,
                    &error,
                ) =>
            {
                recover_biometry_lockout()?;
                return authorize_local_kek_access_inner(policy, false);
            }
            Err(error) => {
                return Err(map_local_auth_error(
                    "authorize FRAMKey local KEK access",
                    error,
                ));
            }
        };
        if !authorized {
            return Err(FramkeyError::unsupported(format!(
                "{} authorization failed for FRAMKey local KEK access",
                local_auth_label(policy)
            )));
        }

        let domain_state = context
            .evaluated_policy_domain_state()
            .map_err(|error| {
                map_local_auth_error("read macOS LocalAuthentication domain state", error)
            })?
            .ok_or_else(|| {
                FramkeyError::unsupported(
                    "macOS LocalAuthentication domain state is unavailable for FRAMKey local KEK access",
                )
            })?;

        let mut hasher = blake3::Hasher::new();
        hasher.update(b"framkey:macos-local-auth-domain-state:v1");
        hasher.update(policy.as_str().as_bytes());
        hasher.update(&[0]);
        hasher.update(&domain_state);
        let hash = hasher.finalize();
        let mut output = [0_u8; 32];
        output.copy_from_slice(hash.as_bytes());
        Ok(output)
    }

    fn should_recover_biometry_lockout(
        policy: KeychainAccessPolicy,
        allow_recovery: bool,
        error: &LAError,
    ) -> bool {
        allow_recovery
            && policy == KeychainAccessPolicy::LocalBiometryCurrentSet
            && matches!(error, LAError::BiometryLockout(_))
    }

    fn recover_biometry_lockout() -> Result<()> {
        let context = LAContext::new().map_err(|error| {
            map_local_auth_error(
                "create macOS LocalAuthentication lockout-recovery context",
                error,
            )
        })?;
        match context.can_evaluate_policy(LAPolicy::DeviceOwnerAuthentication) {
            Ok(true) => {}
            Ok(false) => {
                return Err(FramkeyError::unsupported(
                    "macOS device-owner authentication is not available to recover Touch ID lockout for FRAMKey local KEK access",
                ));
            }
            Err(error) => {
                return Err(map_local_auth_error(
                    "check macOS device-owner authentication for Touch ID lockout recovery",
                    error,
                ));
            }
        }

        let authorized = context
            .evaluate_policy(
                LAPolicy::DeviceOwnerAuthentication,
                "Recover Touch ID for FRAMKey local KEK access.",
            )
            .map_err(|error| {
                map_local_auth_error(
                    "recover Touch ID lockout with macOS device-owner authentication",
                    error,
                )
            })?;
        if !authorized {
            return Err(FramkeyError::unsupported(
                "macOS device-owner authentication did not recover Touch ID lockout for FRAMKey local KEK access",
            ));
        }
        Ok(())
    }

    fn local_auth_label(policy: KeychainAccessPolicy) -> &'static str {
        match policy {
            KeychainAccessPolicy::LocalDeviceOwnerAuthentication => {
                "macOS device-owner authentication"
            }
            KeychainAccessPolicy::LocalBiometryCurrentSet => "Touch ID",
        }
    }

    fn map_security_error(operation: &str, error: SecurityError) -> FramkeyError {
        FramkeyError::unsupported(format!(
            "{operation} failed with Security.framework status {}",
            error.code()
        ))
    }

    fn map_local_auth_error(operation: &str, error: LAError) -> FramkeyError {
        FramkeyError::unsupported(format!(
            "{operation} failed: macOS LocalAuthentication failed ({error})"
        ))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn local_auth_blob_round_trips_policy_state_and_secret() {
            let secret = SecretBytes::<32>::from_slice(&[7_u8; 32]).unwrap();
            let auth_state_hash = [3_u8; 32];
            let blob = keychain_blob(
                KeychainAccessPolicy::LocalBiometryCurrentSet,
                &auth_state_hash,
                &secret,
            );

            assert_eq!(blob.len(), KEYCHAIN_BLOB_LEN);
            let parsed = parse_keychain_blob(&blob, &auth_state_hash).unwrap();

            assert_eq!(parsed.policy, KeychainAccessPolicy::LocalBiometryCurrentSet);
            assert_eq!(parsed.kek.expose(), secret.expose());
        }

        #[test]
        fn legacy_local_auth_blob_version_is_not_accepted() {
            let secret = SecretBytes::<32>::from_slice(&[11_u8; 32]).unwrap();
            let auth_state_hash = [5_u8; 32];
            let mut blob = keychain_blob(
                KeychainAccessPolicy::LocalBiometryCurrentSet,
                &auth_state_hash,
                &secret,
            );
            blob[8..10].copy_from_slice(&1_u16.to_le_bytes());

            assert!(parse_keychain_blob(&blob, &auth_state_hash).is_err());
        }

        #[test]
        fn os_access_control_blob_version_is_not_accepted() {
            let secret = SecretBytes::<32>::from_slice(&[13_u8; 32]).unwrap();
            let auth_state_hash = [6_u8; 32];
            let mut blob = keychain_blob(
                KeychainAccessPolicy::LocalBiometryCurrentSet,
                &auth_state_hash,
                &secret,
            );
            blob[8..10].copy_from_slice(&2_u16.to_le_bytes());

            assert!(parse_keychain_blob(&blob, &auth_state_hash).is_err());
        }

        #[test]
        fn changed_local_auth_domain_state_is_not_accepted() {
            let secret = SecretBytes::<32>::from_slice(&[17_u8; 32]).unwrap();
            let auth_state_hash = [8_u8; 32];
            let changed_auth_state_hash = [9_u8; 32];
            let blob = keychain_blob(
                KeychainAccessPolicy::LocalBiometryCurrentSet,
                &auth_state_hash,
                &secret,
            );

            let error = match parse_keychain_blob(&blob, &changed_auth_state_hash) {
                Ok(_) => panic!("changed local auth domain state was accepted"),
                Err(error) => error,
            };
            assert!(
                error
                    .to_string()
                    .contains("macOS local unlock binding changed")
            );
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use framkey_core::{FramkeyError, Result};
    use framkey_crypto::{SecretBytes, random_array};

    use crate::types::{KeychainAccessPolicy, MacKeychainItem, MacKeychainKek};

    pub fn load_kek(_item: &MacKeychainItem) -> Result<SecretBytes<32>> {
        Err(FramkeyError::unsupported(
            "macOS Keychain bridge is only available on macOS",
        ))
    }

    pub fn load_existing_kek(_item: &MacKeychainItem) -> Result<MacKeychainKek> {
        Err(FramkeyError::unsupported(
            "macOS Keychain bridge is only available on macOS",
        ))
    }

    pub fn ensure_kek(
        _item: &MacKeychainItem,
        _policy: KeychainAccessPolicy,
    ) -> Result<MacKeychainKek> {
        Err(FramkeyError::unsupported(
            "macOS Keychain bridge is only available on macOS",
        ))
    }

    pub fn reset_kek(
        _item: &MacKeychainItem,
        _policy: KeychainAccessPolicy,
    ) -> Result<MacKeychainKek> {
        Err(FramkeyError::unsupported(
            "macOS Keychain bridge is only available on macOS",
        ))
    }

    pub fn delete_kek(_item: &MacKeychainItem) -> Result<bool> {
        Err(FramkeyError::unsupported(
            "macOS Keychain bridge is only available on macOS",
        ))
    }

    pub fn rebind_kek(
        _item: &MacKeychainItem,
        _policy: KeychainAccessPolicy,
    ) -> Result<MacKeychainKek> {
        Err(FramkeyError::unsupported(
            "macOS Keychain bridge is only available on macOS",
        ))
    }
}

#[cfg(target_os = "macos")]
pub use self::imp::*;

#[cfg(not(target_os = "macos"))]
pub use self::imp::*;
