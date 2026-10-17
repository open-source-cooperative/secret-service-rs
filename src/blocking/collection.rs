use crate::{Error, blocking::Item};
use std::collections::HashMap;
use zbus::OwnedObjectPath;

pub struct Collection<'a> {
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
    pub fn is_locked(&self) -> Result<bool, Error> {
        zruntime::block_on(self.inner.is_locked())
    }

    pub fn ensure_unlocked(&self) -> Result<(), Error> {
        zruntime::block_on(self.inner.ensure_unlocked())
    }

    pub fn unlock(&self) -> Result<(), Error> {
        zruntime::block_on(self.inner.unlock())
    }

    pub fn lock(&self) -> Result<(), Error> {
        zruntime::block_on(self.inner.lock())
    }

    pub fn delete(&self) -> Result<(), Error> {
        zruntime::block_on(self.inner.delete())
    }

    pub fn get_all_items(&'a self) -> Result<Vec<Item<'a>>, Error> {
        Ok(zruntime::block_on(self.inner.get_all_items())?
            .into_iter()
            .map(Item::from)
            .collect())
    }

    pub fn search_items(&'a self, attributes: HashMap<&str, &str>) -> Result<Vec<Item<'a>>, Error> {
        Ok(zruntime::block_on(self.inner.search_items(attributes))?
            .into_iter()
            .map(Item::from)
            .collect())
    }

    pub fn get_label(&self) -> Result<String, Error> {
        zruntime::block_on(self.inner.get_label())
    }

    pub fn set_label(&self, new_label: &str) -> Result<(), Error> {
        zruntime::block_on(self.inner.set_label(new_label))
    }

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
    }

    #[test]
    fn should_check_if_collection_locked() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();
        let _ = collection.is_locked().unwrap();
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
        collection.get_all_items().unwrap();
    }

    #[test]
    fn should_search_items() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collection = ss.get_default_collection().unwrap();

        let item = collection
            .create_item(
                "test",
                HashMap::from([("test_attributes_in_collection", "test")]),
                b"test_secret",
                false,
                "text/plain",
            )
            .unwrap();

        collection.search_items(HashMap::new()).unwrap();

        let bad_search = collection
            .search_items(HashMap::from([("test_bad", "test")]))
            .unwrap();
        assert_eq!(bad_search.len(), 0);

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

        collection.unlock().unwrap();
        collection.set_label("Test").unwrap();
        let label = collection.get_label().unwrap();
        assert_eq!(label, "Test");

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
        {
            let ss = SecretService::connect(EncryptionType::Plain).unwrap();
            let collection = ss.get_default_collection().unwrap();
            label = collection.get_label().unwrap();
            path = collection.collection_path.clone();
        }
        {
            let ss = SecretService::connect(EncryptionType::Plain).unwrap();
            let collection_prime = ss.get_collection_by_path(path).unwrap();
            let label_prime = collection_prime.get_label().unwrap();
            assert_eq!(label, label_prime);
        }
    }
}
