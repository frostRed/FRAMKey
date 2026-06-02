use std::fmt;

use framkey_core::{FramkeyError, Result};
use framkey_crypto::SecretBytes;

use crate::platform;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacKeychainItem {
    pub service: String,
    pub account: String,
}

impl MacKeychainItem {
    pub fn new(service: impl Into<String>, account: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            account: account.into(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.service.trim().is_empty() {
            return Err(FramkeyError::invalid_data(
                "macOS Keychain service must not be blank",
            ));
        }
        if self.account.trim().is_empty() {
            return Err(FramkeyError::invalid_data(
                "macOS Keychain account must not be blank",
            ));
        }
        if self.service != self.service.trim() || self.account != self.account.trim() {
            return Err(FramkeyError::invalid_data(
                "macOS Keychain service/account must not have leading or trailing whitespace",
            ));
        }
        if self.service.chars().any(char::is_control) || self.account.chars().any(char::is_control)
        {
            return Err(FramkeyError::invalid_data(
                "macOS Keychain service/account must not contain control characters",
            ));
        }
        Ok(())
    }

    pub fn keychain_item_id(&self) -> String {
        format!("{}:{}", self.service, self.account)
    }

    pub fn device_binding_id(&self) -> [u8; 16] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"framkey:mac-keychain-device-binding:v1");
        hasher.update(self.service.as_bytes());
        hasher.update(&[0]);
        hasher.update(self.account.as_bytes());
        let hash = hasher.finalize();
        let mut output = [0_u8; 16];
        output.copy_from_slice(&hash.as_bytes()[..16]);
        output
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeychainAccessPolicy {
    LocalDeviceOwnerAuthentication,
}

impl KeychainAccessPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalDeviceOwnerAuthentication => "local_device_owner_authentication",
        }
    }
}

pub trait MacKeychain {
    fn load_kek(&self, item: &MacKeychainItem) -> Result<SecretBytes<32>>;
}

pub struct MacKeychainKek {
    pub item: MacKeychainItem,
    pub created: bool,
    pub access_policy: KeychainAccessPolicy,
    pub keychain_item_id: String,
    pub device_id: [u8; 16],
    pub kek_id: [u8; 16],
    pub kek: SecretBytes<32>,
}

impl fmt::Debug for MacKeychainKek {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MacKeychainKek")
            .field("item", &self.item)
            .field("created", &self.created)
            .field("access_policy", &self.access_policy)
            .field("keychain_item_id", &self.keychain_item_id)
            .field("device_id", &self.device_id)
            .field("kek_id", &self.kek_id)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default)]
pub struct SystemKeychain;

impl SystemKeychain {
    pub fn load_existing_kek(&self, item: &MacKeychainItem) -> Result<MacKeychainKek> {
        platform::load_existing_kek(item)
    }

    pub fn ensure_kek(
        &self,
        item: &MacKeychainItem,
        policy: KeychainAccessPolicy,
    ) -> Result<MacKeychainKek> {
        platform::ensure_kek(item, policy)
    }

    pub fn reset_kek(
        &self,
        item: &MacKeychainItem,
        policy: KeychainAccessPolicy,
    ) -> Result<MacKeychainKek> {
        platform::reset_kek(item, policy)
    }

    pub fn delete_kek(&self, item: &MacKeychainItem) -> Result<bool> {
        platform::delete_kek(item)
    }

    pub fn rebind_kek(
        &self,
        item: &MacKeychainItem,
        policy: KeychainAccessPolicy,
    ) -> Result<MacKeychainKek> {
        platform::rebind_kek(item, policy)
    }
}

impl MacKeychain for SystemKeychain {
    fn load_kek(&self, item: &MacKeychainItem) -> Result<SecretBytes<32>> {
        platform::load_kek(item)
    }
}

pub(crate) fn mac_keychain_kek(
    item: MacKeychainItem,
    created: bool,
    access_policy: KeychainAccessPolicy,
    kek: SecretBytes<32>,
) -> MacKeychainKek {
    let keychain_item_id = item.keychain_item_id();
    let device_id = item.device_binding_id();
    let kek_id = mac_keychain_kek_id(&kek);
    MacKeychainKek {
        item,
        created,
        access_policy,
        keychain_item_id,
        device_id,
        kek_id,
        kek,
    }
}

pub(crate) fn mac_keychain_kek_id(kek: &SecretBytes<32>) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"framkey:mac-keychain-kek-id:v1");
    hasher.update(kek.expose());
    let hash = hasher.finalize();
    let mut output = [0_u8; 16];
    output.copy_from_slice(&hash.as_bytes()[..16]);
    output
}
