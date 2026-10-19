//! A blocking secret service API.
//!
//! This `SecretService` blocks the current thread when making requests to the
//! secret service server instead of returning futures.
//!
//! Do not call these methods in an async context, where they can stall the runtime.
//! Use the [async `SecretService`] there instead.
//!
//! [async `SecretService`]: crate::SecretService

use crate::{EncryptionType, Error, SearchItemsResult};
use std::collections::HashMap;
use zbus::OwnedObjectPath;

mod collection;
pub use collection::Collection;
mod item;
pub use item::Item;

/// Secret Service Struct.
///
/// This the main entry point for usage of the library.
///
/// Creating a new [SecretService] will also initialize dbus
/// and negotiate a new cryptographic session
/// ([EncryptionType::Plain] or [EncryptionType::Dh])
pub struct SecretService<'a> {
    inner: crate::SecretService<'a>,
}

impl<'a> SecretService<'a> {
    /// Create a new [SecretService] instance.
    ///
    /// This will initialize its own connection to the session bus.
    pub fn connect(encryption: EncryptionType) -> Result<Self, Error> {
        Ok(Self {
            inner: zruntime::block_on(crate::SecretService::connect(encryption))?,
        })
    }

    /// Creates a new [SecretService] instance, utilizing an existing connection handle.
    ///
    /// `session_conn` should be connected to the session/user message bus.
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

    /// Get all collections
    pub fn get_all_collections(&'a self) -> Result<Vec<Collection<'a>>, Error> {
        Ok(zruntime::block_on(self.inner.get_all_collections())?
            .into_iter()
            .map(Collection::from)
            .collect())
    }

    /// Get collection by alias.
    ///
    /// Most common would be the `default` alias, but there
    /// is also a specific method for getting the collection
    /// by default alias.
    pub fn get_collection_by_alias(&'a self, alias: &str) -> Result<Collection<'a>, Error> {
        Ok(Collection::from(zruntime::block_on(
            self.inner.get_collection_by_alias(alias),
        )?))
    }

    /// Get default collection.
    /// (The collection whose alias is `default`)
    pub fn get_default_collection(&'a self) -> Result<Collection<'a>, Error> {
        Ok(Collection::from(zruntime::block_on(
            self.inner.get_default_collection(),
        )?))
    }

    /// Get any collection.
    /// First tries `default` collection, then `session`
    /// collection, then the first collection when it
    /// gets all collections.
    pub fn get_any_collection(&'a self) -> Result<Collection<'a>, Error> {
        Ok(Collection::from(zruntime::block_on(
            self.inner.get_any_collection(),
        )?))
    }

    /// Creates a new collection with a label and an alias.
    pub fn create_collection(&'a self, label: &str, alias: &str) -> Result<Collection<'a>, Error> {
        Ok(Collection::from(zruntime::block_on(
            self.inner.create_collection(label, alias),
        )?))
    }

    /// Searches all items by attributes
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

    /// Unlock all items in a batch
    pub fn unlock_all(&'a self, items: &[&Item<'_>]) -> Result<(), Error> {
        let items: Vec<_> = items.iter().map(|item| &item.inner).collect();
        zruntime::block_on(self.inner.unlock_all(&items))
    }

    /// Get an item by its dbus object path.
    pub fn get_item_by_path(&'a self, item_path: OwnedObjectPath) -> Result<Item<'a>, Error> {
        Ok(Item::from(zruntime::block_on(
            self.inner.get_item_by_path(item_path),
        )?))
    }

    /// Get a collection by its dbus object path.
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
    fn unlock_all_reports_a_closed_connection() {
        let connection = zruntime::block_on(zbus::Connection::session()).unwrap();
        let service =
            SecretService::connect_with_existing(EncryptionType::Plain, connection.clone())
                .unwrap();
        zruntime::block_on(connection.close()).unwrap();
        assert!(service.unlock_all(&[]).is_err());
    }

    #[test]
    fn should_get_all_collections() {
        // Assumes that there will always be a default
        // collection
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

        // Create an item
        let item = collection
            .create_item(
                "test",
                HashMap::from([("test_attribute_in_ss", "test_value")]),
                b"test_secret",
                false,
                "text/plain",
            )
            .unwrap();

        // handle empty vec search
        ss.search_items(HashMap::new()).unwrap();

        // handle no result
        let bad_search = ss.search_items(HashMap::from([("test", "test")])).unwrap();
        assert_eq!(bad_search.unlocked.len(), 0);
        assert_eq!(bad_search.locked.len(), 0);

        // handle correct search for item and compare
        let search_item = ss
            .search_items(HashMap::from([("test_attribute_in_ss", "test_value")]))
            .unwrap();

        assert_eq!(item.item_path, search_item.unlocked[0].item_path);
        assert_eq!(search_item.locked.len(), 0);
        item.delete().unwrap();
    }
}
