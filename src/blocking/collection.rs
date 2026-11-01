use crate::{Error, blocking::Item};
use std::collections::HashMap;
use zbus::OwnedObjectPath;

/// Collection struct.
/// Should always be created from the [`SecretService`](crate::blocking::SecretService) entry point,
/// whether through a new collection or a collection search
pub struct Collection<'a> {
    /// The dbus object path of the collection.
    pub collection_path: OwnedObjectPath,
    inner: crate::Collection<'a>,
}

impl<'a> From<crate::Collection<'a>> for Collection<'a> {
    fn from(inner: crate::Collection<'a>) -> Self {
        Self {
            collection_path: inner.collection_path.clone(),
            inner,
        }
    }
}

impl<'a> Collection<'a> {
    /// Whether the collection is locked.
    pub fn is_locked(&self) -> Result<bool, Error> {
        zruntime::block_on(self.inner.is_locked())
    }

    /// Return [Error::Locked] if the collection is locked.
    pub fn ensure_unlocked(&self) -> Result<(), Error> {
        zruntime::block_on(self.inner.ensure_unlocked())
    }

    /// Unlock the collection.
    pub fn unlock(&self) -> Result<(), Error> {
        zruntime::block_on(self.inner.unlock())
    }

    /// Lock the collection.
    pub fn lock(&self) -> Result<(), Error> {
        zruntime::block_on(self.inner.lock())
    }

    /// Deletes dbus object, but struct instance still exists (current implementation)
    pub fn delete(&self) -> Result<(), Error> {
        zruntime::block_on(self.inner.delete())
    }

    /// Get all items in the collection.
    pub fn get_all_items(&'a self) -> Result<Vec<Item<'a>>, Error> {
        Ok(zruntime::block_on(self.inner.get_all_items())?
            .into_iter()
            .map(Item::from)
            .collect())
    }

    /// Search the collection's items by attributes.
    pub fn search_items(&'a self, attributes: HashMap<&str, &str>) -> Result<Vec<Item<'a>>, Error> {
        Ok(zruntime::block_on(self.inner.search_items(attributes))?
            .into_iter()
            .map(Item::from)
            .collect())
    }

    /// Get the collection's label.
    pub fn get_label(&self) -> Result<String, Error> {
        zruntime::block_on(self.inner.get_label())
    }

    /// Set the collection's label.
    pub fn set_label(&self, new_label: &str) -> Result<(), Error> {
        zruntime::block_on(self.inner.set_label(new_label))
    }

    /// Create a new item in the collection, replacing any item with the same attributes if `replace` is true.
    pub fn create_item(
        &'a self,
        label: &str,
        attributes: HashMap<&str, &str>,
        secret: &[u8],
        replace: bool,
        content_type: &str,
    ) -> Result<Item<'a>, Error> {
        Ok(Item::from(zruntime::block_on(self.inner.create_item(
            label,
            attributes,
            secret,
            replace,
            content_type,
        ))?))
    }
}

#[cfg(test)]
mod test {
    use crate::blocking::*;

    #[test]
    fn should_create_collection_struct() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let _ = ss.get_default_collection().unwrap();
        // tested under SecretService struct
    }

    #[test]
    fn should_check_if_collection_locked() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        assert!(!collection.is_locked().unwrap());
    }

    #[test]
    fn missing_collection_has_no_lock_state() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss
            .get_collection_by_path(
                OwnedObjectPath::try_from("/org/freedesktop/secrets/collection/missing").unwrap(),
            )
            .unwrap();
        assert!(collection.is_locked().is_err());
    }

    #[test]
    fn missing_collection_cannot_be_confirmed_unlocked() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss
            .get_collection_by_path(
                OwnedObjectPath::try_from("/org/freedesktop/secrets/collection/missing").unwrap(),
            )
            .unwrap();
        assert!(collection.ensure_unlocked().is_err());
    }

    #[test]
    fn missing_collection_cannot_be_deleted() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss
            .get_collection_by_path(
                OwnedObjectPath::try_from("/org/freedesktop/secrets/collection/missing").unwrap(),
            )
            .unwrap();
        assert!(collection.delete().is_err());
    }

    #[test]
    fn missing_collection_cannot_be_renamed() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss
            .get_collection_by_path(
                OwnedObjectPath::try_from("/org/freedesktop/secrets/collection/missing").unwrap(),
            )
            .unwrap();
        assert!(collection.set_label("test").is_err());
    }

    #[test]
    #[ignore] // should unignore this test this manually, otherwise will constantly prompt during tests.
    fn should_lock_and_unlock() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let locked = collection.is_locked().unwrap();
        if locked {
            collection.unlock().unwrap();
            collection.ensure_unlocked().unwrap();
            assert!(!collection.is_locked().unwrap());
            collection.lock().unwrap();
            assert!(collection.is_locked().unwrap());
        } else {
            collection.lock().unwrap();
            assert!(collection.is_locked().unwrap());
            collection.unlock().unwrap();
            collection.ensure_unlocked().unwrap();
            assert!(!collection.is_locked().unwrap());
        }
    }

    #[test]
    #[ignore]
    fn should_delete_collection() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collections = ss.get_all_collections().unwrap();
        let count_before = collections.len();
        for collection in collections {
            let collection_path = &*collection.collection_path;
            if collection_path.contains("Test") {
                collection.unlock().unwrap();
                collection.delete().unwrap();
            }
        }
        //double check after
        let collections = ss.get_all_collections().unwrap();
        assert!(
            collections.len() < count_before,
            "collections before delete {count_before}",
        )
    }

    #[test]
    fn should_get_all_items() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let item = collection
            .create_item(
                "all_items_test",
                HashMap::new(),
                b"test",
                false,
                "text/plain",
            )
            .unwrap();
        let items = collection.get_all_items().unwrap();
        assert!(items.iter().any(|found| found.item_path == item.item_path));
        item.delete().unwrap();
    }

    #[test]
    fn should_search_items() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();

        // Create an item
        let item = collection
            .create_item(
                "test",
                HashMap::from([("test_attributes_in_collection", "test")]),
                b"test_secret",
                false,
                "text/plain",
            )
            .unwrap();

        // handle empty vec search
        collection.search_items(HashMap::new()).unwrap();

        // handle no result
        let bad_search = collection
            .search_items(HashMap::from([("test_bad", "test")]))
            .unwrap();
        assert_eq!(bad_search.len(), 0);

        // handle correct search for item and compare
        let search_item = collection
            .search_items(HashMap::from([("test_attributes_in_collection", "test")]))
            .unwrap();

        assert_eq!(item.item_path, search_item[0].item_path);
        item.delete().unwrap();
    }

    #[test]
    #[ignore]
    fn should_get_and_set_collection_label() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let label = collection.get_label().unwrap();
        assert_eq!(label, "Login");

        // Set label to test and check
        collection.unlock().unwrap();
        collection.set_label("Test").unwrap();
        let label = collection.get_label().unwrap();
        assert_eq!(label, "Test");

        // Reset label to original and test
        collection.unlock().unwrap();
        collection.set_label("Login").unwrap();
        let label = collection.get_label().unwrap();
        assert_eq!(label, "Login");

        collection.lock().unwrap();
    }

    #[test]
    fn should_get_collection_by_path() {
        let path: OwnedObjectPath;
        let label: String;
        // get the default collection on one blocking call, remember its path and label
        {
            let ss = SecretService::connect(EncryptionType::Plain).unwrap();
            let collection = ss.get_default_collection().unwrap();
            label = collection.get_label().unwrap();
            path = collection.collection_path.clone();
        }
        // get collection by path on another blocking call, check that the label is the same
        {
            let ss = SecretService::connect(EncryptionType::Plain).unwrap();
            let collection_prime = ss.get_collection_by_path(path).unwrap();
            let label_prime = collection_prime.get_label().unwrap();
            assert_eq!(label, label_prime);
        }
    }

    #[test]
    fn should_get_collection_label_from_service() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let async_service =
            zruntime::block_on(crate::SecretService::connect(EncryptionType::Plain)).unwrap();
        let async_collection = zruntime::block_on(
            async_service.get_collection_by_path(collection.collection_path.clone()),
        )
        .unwrap();
        assert_eq!(
            collection.get_label().unwrap(),
            zruntime::block_on(async_collection.get_label()).unwrap()
        );
    }
}
