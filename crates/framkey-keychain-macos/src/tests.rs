use super::*;

#[test]
fn item_binding_ids_are_stable_and_distinct() {
    let item = MacKeychainItem::new("io.framkey.kek", "default");
    let same = MacKeychainItem::new("io.framkey.kek", "default");
    let other = MacKeychainItem::new("io.framkey.kek", "other");

    assert_eq!(item.keychain_item_id(), "io.framkey.kek:default");
    assert_eq!(item.device_binding_id(), same.device_binding_id());
    assert_ne!(item.device_binding_id(), other.device_binding_id());
}

#[test]
fn item_validation_rejects_empty_or_nul_values() {
    assert!(MacKeychainItem::new("", "default").validate().is_err());
    assert!(
        MacKeychainItem::new("io.framkey.kek", "")
            .validate()
            .is_err()
    );
    assert!(
        MacKeychainItem::new("io.framkey.kek", "bad\0account")
            .validate()
            .is_err()
    );
}
