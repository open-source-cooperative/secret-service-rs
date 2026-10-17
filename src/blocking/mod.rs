//! Synchronous access to the Secret Service API.
//!
//! Do not call these methods from an async context: they block the current thread.

use crate::{EncryptionType, Error, SearchItemsResult};
use std::collections::HashMap;
use zbus::OwnedObjectPath;

mod collection;
pub use collection::Collection;
mod item;
pub use item::Item;

pub struct SecretService<'a> {
    inner: crate::SecretService<'a>,
}

impl<'a> SecretService<'a> {
    pub fn connect(encryption: EncryptionType) -> Result<Self, Error> {
        Ok(Self {
            inner: zruntime::block_on(crate::SecretService::connect(encryption))?,
        })
    }

    pub fn connect_with_existing(
        encryption: EncryptionType,
        session_conn: zbus::Connection,
    ) -> Result<Self, Error> {
        Ok(Self {
            inner: zruntime::block_on(crate::SecretService::connect_with_existing(
                encryption,
                session_conn,
            ))?,
        })
    }

    pub fn get_all_collections(&'a self) -> Result<Vec<Collection<'a>>, Error> {
        Ok(zruntime::block_on(self.inner.get_all_collections())?
            .into_iter()
            .map(Collection::from)
            .collect())
    }

    pub fn get_collection_by_alias(&'a self, alias: &str) -> Result<Collection<'a>, Error> {
        Ok(Collection::from(zruntime::block_on(
            self.inner.get_collection_by_alias(alias),
        )?))
    }

    pub fn get_default_collection(&'a self) -> Result<Collection<'a>, Error> {
        Ok(Collection::from(zruntime::block_on(
            self.inner.get_default_collection(),
        )?))
    }

    pub fn get_any_collection(&'a self) -> Result<Collection<'a>, Error> {
        Ok(Collection::from(zruntime::block_on(
            self.inner.get_any_collection(),
        )?))
    }

    pub fn create_collection(&'a self, label: &str, alias: &str) -> Result<Collection<'a>, Error> {
        Ok(Collection::from(zruntime::block_on(
            self.inner.create_collection(label, alias),
        )?))
    }

    pub fn search_items(
        &'a self,
        attributes: HashMap<&str, &str>,
    ) -> Result<SearchItemsResult<Item<'a>>, Error> {
        let found = zruntime::block_on(self.inner.search_items(attributes))?;
        Ok(SearchItemsResult {
            unlocked: found.unlocked.into_iter().map(Item::from).collect(),
            locked: found.locked.into_iter().map(Item::from).collect(),
        })
    }

    pub fn unlock_all(&'a self, items: &[&Item<'_>]) -> Result<(), Error> {
        let items: Vec<_> = items.iter().map(|item| &item.inner).collect();
        zruntime::block_on(self.inner.unlock_all(&items))
    }

    pub fn get_item_by_path(&'a self, item_path: OwnedObjectPath) -> Result<Item<'a>, Error> {
        Ok(Item::from(zruntime::block_on(
            self.inner.get_item_by_path(item_path),
        )?))
    }

    pub fn get_collection_by_path(
        &'a self,
        collection_path: OwnedObjectPath,
    ) -> Result<Collection<'a>, Error> {
        Ok(Collection::from(zruntime::block_on(
            self.inner.get_collection_by_path(collection_path),
        )?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::convert::TryFrom;
    use zbus::ObjectPath;

    #[test]
    fn should_create_secret_service() {
        SecretService::connect(EncryptionType::Plain).unwrap();
    }

    #[test]
    fn should_connect_with_existing_connection() {
        let connection = zruntime::block_on(zbus::Connection::session()).unwrap();
        let service =
            SecretService::connect_with_existing(EncryptionType::Plain, connection).unwrap();
        assert!(service.get_default_collection().is_ok());
    }

    #[test]
    fn should_get_all_collections() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let collections = ss.get_all_collections().unwrap();
        assert!(!collections.is_empty(), "no collections found");
    }

    #[test]
    fn should_get_collection_by_alias() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        ss.get_collection_by_alias("session").unwrap();
    }

    #[test]
    fn should_return_error_if_collection_doesnt_exist() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();

        match ss.get_collection_by_alias("definitely_definitely_does_not_exist") {
            Err(Error::NoResult) => {}
            _ => panic!(),
        };
    }

    #[test]
    fn should_get_default_collection() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        ss.get_default_collection().unwrap();
    }

    #[test]
    fn should_get_any_collection() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let _ = ss.get_any_collection().unwrap();
    }

    #[test_with::no_env(GITHUB_ACTIONS)]
    #[test]
    fn should_create_and_delete_collection() {
        let ss = SecretService::connect(EncryptionType::Plain).unwrap();
        let test_collection = ss.create_collection("Test", "").unwrap();
        assert_eq!(
            ObjectPath::from(test_collection.collection_path.clone()),
            ObjectPath::try_from("/org/freedesktop/secrets/collection/Test").unwrap()
        );
        test_collection.delete().unwrap();
    }

    #[test]
    fn should_search_items() {
        let ss = SecretService::connect(EncryptionType::Dh).unwrap();
        let collection = ss.get_default_collection().unwrap();

        let item = collection
            .create_item(
                "test",
                HashMap::from([("test_attribute_in_ss", "test_value")]),
                b"test_secret",
                false,
                "text/plain",
            )
            .unwrap();

        ss.search_items(HashMap::new()).unwrap();

        let bad_search = ss.search_items(HashMap::from([("test", "test")])).unwrap();
        assert_eq!(bad_search.unlocked.len(), 0);
        assert_eq!(bad_search.locked.len(), 0);

        let search_item = ss
            .search_items(HashMap::from([("test_attribute_in_ss", "test_value")]))
            .unwrap();

        assert_eq!(item.item_path, search_item.unlocked[0].item_path);
        assert_eq!(search_item.locked.len(), 0);
        item.delete().unwrap();
    }
}
