use crate::Error;
use std::collections::HashMap;
use zbus::OwnedObjectPath;

/// A Secret Service item.
pub struct Item<'a> {
    /// The dbus object path of the item.
    pub item_path: OwnedObjectPath,
    pub(crate) inner: crate::Item<'a>,
}

impl<'a> From<crate::Item<'a>> for Item<'a> {
    fn from(inner: crate::Item<'a>) -> Self {
        Self {
            item_path: inner.item_path.clone(),
            inner,
        }
    }
}

impl Item<'_> {
    /// Whether the item is locked.
    pub fn is_locked(&self) -> Result<bool, Error> {
        zruntime::block_on(self.inner.is_locked())
    }

    /// Return [Error::Locked] if the item is locked.
    pub fn ensure_unlocked(&self) -> Result<(), Error> {
        zruntime::block_on(self.inner.ensure_unlocked())
    }

    /// Unlock the item.
    pub fn unlock(&self) -> Result<(), Error> {
        zruntime::block_on(self.inner.unlock())
    }

    /// Lock the item.
    pub fn lock(&self) -> Result<(), Error> {
        zruntime::block_on(self.inner.lock())
    }

    /// Get the item's attributes.
    pub fn get_attributes(&self) -> Result<HashMap<String, String>, Error> {
        zruntime::block_on(self.inner.get_attributes())
    }

    /// Replace the item's attributes.
    pub fn set_attributes(&self, attributes: HashMap<&str, &str>) -> Result<(), Error> {
        zruntime::block_on(self.inner.set_attributes(attributes))
    }

    /// Get the item's label.
    pub fn get_label(&self) -> Result<String, Error> {
        zruntime::block_on(self.inner.get_label())
    }

    /// Set the item's label.
    pub fn set_label(&self, new_label: &str) -> Result<(), Error> {
        zruntime::block_on(self.inner.set_label(new_label))
    }

    /// Deletes dbus object, but struct instance still exists (current implementation)
    pub fn delete(&self) -> Result<(), Error> {
        zruntime::block_on(self.inner.delete())
    }

    /// Get the item's secret, decrypted with the session key when one
    /// is in use.
    pub fn get_secret(&self) -> Result<Vec<u8>, Error> {
        zruntime::block_on(self.inner.get_secret())
    }

    /// Get the MIME type of the item's secret.
    pub fn get_secret_content_type(&self) -> Result<String, Error> {
        zruntime::block_on(self.inner.get_secret_content_type())
    }

    /// Replace the item's secret and its MIME type.
    pub fn set_secret(&self, secret: &[u8], content_type: &str) -> Result<(), Error> {
        zruntime::block_on(self.inner.set_secret(secret, content_type))
    }

    /// Get the time the item was created, in seconds since the Unix epoch.
    pub fn get_created(&self) -> Result<u64, Error> {
        zruntime::block_on(self.inner.get_created())
    }

    /// Get the time the item was last modified, in seconds since the Unix epoch.
    pub fn get_modified(&self) -> Result<u64, Error> {
        zruntime::block_on(self.inner.get_modified())
    }
}

impl Eq for Item<'_> {}
impl PartialEq for Item<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.item_path == other.item_path
    }
}

#[cfg(test)]
mod test {
    use crate::blocking::*;

    fn create_test_default_item<'a>(collection: &'a Collection<'_>) -> Item<'a> {
        collection
            .create_item("Test", HashMap::new(), b"test", false, "text/plain")
            .unwrap()
    }

    #[test]
    fn should_create_and_delete_item() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = create_test_default_item(&collection);

        item.delete().unwrap();
        // Random operation to prove that path no longer exists
        if item.get_label().is_ok() {
            panic!("item still existed");
        }
    }

    #[test]
    fn should_check_if_item_locked() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = create_test_default_item(&collection);

        assert!(!item.is_locked().unwrap());
        item.delete().unwrap();
    }

    #[test]
    fn deleted_item_does_not_report_a_lock_state() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = create_test_default_item(&collection);
        item.delete().unwrap();
        assert!(item.is_locked().is_err());
    }

    #[test]
    fn deleted_item_cannot_be_confirmed_unlocked() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = create_test_default_item(&collection);
        item.delete().unwrap();
        assert!(item.ensure_unlocked().is_err());
    }

    #[test]
    #[ignore]
    fn should_lock_and_unlock() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = create_test_default_item(&collection);

        let locked = item.is_locked().unwrap();
        if locked {
            item.unlock().unwrap();
            item.ensure_unlocked().unwrap();
            assert!(!item.is_locked().unwrap());
            item.lock().unwrap();
            assert!(item.is_locked().unwrap());
        } else {
            item.lock().unwrap();
            assert!(item.is_locked().unwrap());
            item.unlock().unwrap();
            item.ensure_unlocked().unwrap();
            assert!(!item.is_locked().unwrap());
        }
        item.delete().unwrap();
    }

    #[test]
    fn should_get_and_set_item_label() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = create_test_default_item(&collection);

        // Set label to test and check
        item.set_label("Tester").unwrap();
        let label = item.get_label().unwrap();
        assert_eq!(label, "Tester");
        item.delete().unwrap();
    }

    #[test]
    fn should_get_item_by_path() {
        let path: OwnedObjectPath;
        // first create the item on one blocking call, set its label, and get its path
        {
            let ss = SecretService::connect(EncryptionType::Plain).unwrap();
            let collection = ss.get_default_collection().unwrap();
            let item = create_test_default_item(&collection);
            item.set_label("Tester").unwrap();
            path = item.item_path.clone();
        }
        // now get the item by path on another blocking call, check its label, and delete it
        {
            let ss = SecretService::connect(EncryptionType::Plain).unwrap();
            let item = ss.get_item_by_path(path).unwrap();
            let label = item.get_label().unwrap();
            assert_eq!(label, "Tester");
            item.delete().unwrap();
        }
    }

    #[test]
    fn should_create_with_item_attributes() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = collection
            .create_item(
                "Test",
                HashMap::from([("test_attributes_in_item", "test")]),
                b"test",
                false,
                "text/plain",
            )
            .unwrap();

        let attributes = item.get_attributes().unwrap();

        // We do not compare exact attributes, since the secret service provider could add its own
        // at any time. Instead, we only check that the ones we provided are returned back.
        assert_eq!(
            attributes
                .get("test_attributes_in_item")
                .map(String::as_str),
            Some("test")
        );

        item.delete().unwrap();
    }

    #[test]
    fn should_get_and_set_item_attributes() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = create_test_default_item(&collection);

        // Also test empty array handling
        item.set_attributes(HashMap::new()).unwrap();
        item.set_attributes(HashMap::from([("test_attributes_in_item_get", "test")]))
            .unwrap();

        let attributes = item.get_attributes().unwrap();

        // We do not compare exact attributes, since the secret service provider could add its own
        // at any time. Instead, we only check that the ones we provided are returned back.
        assert_eq!(
            attributes
                .get("test_attributes_in_item_get")
                .map(String::as_str),
            Some("test")
        );

        item.delete().unwrap();
    }

    #[test]
    fn should_get_modified_created_props() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = create_test_default_item(&collection);

        item.set_label("Tester").unwrap();
        assert!(item.get_created().unwrap() > 1);
        assert!(item.get_modified().unwrap() > 1);
        item.delete().unwrap();
    }

    #[test]
    fn should_create_and_get_secret() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = create_test_default_item(&collection);

        let secret = item.get_secret().unwrap();
        item.delete().unwrap();
        assert_eq!(secret, b"test");
    }

    #[test]
    fn should_create_and_get_secret_encrypted() {
        let ss = SecretService::connect(EncryptionType::Dh).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = create_test_default_item(&collection);

        let secret = item.get_secret().unwrap();
        item.delete().unwrap();
        assert_eq!(secret, b"test");
    }

    #[test]
    fn should_get_secret_content_type() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = create_test_default_item(&collection);

        let content_type = item.get_secret_content_type().unwrap();
        item.delete().unwrap();
        assert_eq!(content_type, "text/plain".to_owned());
    }

    #[test]
    fn should_set_secret() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = create_test_default_item(&collection);

        item.set_secret(b"new_test", "text/plain").unwrap();
        let secret = item.get_secret().unwrap();
        item.delete().unwrap();
        assert_eq!(secret, b"new_test");
    }

    #[test]
    fn should_create_encrypted_item() {
        let ss = SecretService::connect(EncryptionType::Dh).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = collection
            .create_item(
                "Test",
                HashMap::new(),
                b"test_encrypted",
                false,
                "text/plain",
            )
            .expect("Error on item creation");
        let secret = item.get_secret().unwrap();
        item.delete().unwrap();
        assert_eq!(secret, b"test_encrypted");
    }

    #[test]
    fn should_create_encrypted_item_from_empty_secret() {
        let ss = SecretService::connect(EncryptionType::Dh).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = collection
            .create_item("Test", HashMap::new(), b"", false, "text/plain")
            .expect("Error on item creation");
        let secret = item.get_secret().unwrap();
        item.delete().unwrap();
        assert_eq!(secret, b"");
    }

    #[test]
    fn should_get_encrypted_secret_across_dbus_connections() {
        {
            let ss = SecretService::connect(EncryptionType::Dh).unwrap();
            let collection = ss.get_default_collection().unwrap();
            let item = collection
                .create_item(
                    "Test",
                    HashMap::from([("test_attributes_in_item_encrypt", "test")]),
                    b"test_encrypted",
                    false,
                    "text/plain",
                )
                .expect("Error on item creation");
            let secret = item.get_secret().unwrap();
            assert_eq!(secret, b"test_encrypted");
        }
        {
            let ss = SecretService::connect(EncryptionType::Dh).unwrap();
            let collection = ss.get_default_collection().unwrap();
            let search_item = collection
                .search_items(HashMap::from([("test_attributes_in_item_encrypt", "test")]))
                .unwrap();
            let item = search_item.first().unwrap();
            assert_eq!(item.get_secret().unwrap(), b"test_encrypted");
            item.delete().unwrap();
        }
    }
}
