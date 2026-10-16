//! Definitions for secret service interactions

// DBus Name
pub(crate) const SS_DBUS_NAME: &str = "org.freedesktop.secrets";

// Item Properties
pub(crate) const SS_ITEM_LABEL: &str = "org.freedesktop.Secret.Item.Label";
pub(crate) const SS_ITEM_ATTRIBUTES: &str = "org.freedesktop.Secret.Item.Attributes";

// Algorithm Names
pub(crate) const ALGORITHM_PLAIN: &str = "plain";
pub(crate) const ALGORITHM_DH: &str = "dh-ietf1024-sha256-aes128-cbc-pkcs7";

// Collection properties
pub(crate) const SS_COLLECTION_LABEL: &str = "org.freedesktop.Secret.Collection.Label";
