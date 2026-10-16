pub(crate) mod collection;
pub(crate) mod item;
pub(crate) mod prompt;
pub(crate) mod service;

use serde::{Deserialize, Serialize};
use zbus::zvariant::{OwnedObjectPath, Type};

#[derive(Debug, Serialize, Deserialize, Type)]
pub(crate) struct SecretStruct {
    pub(crate) session: OwnedObjectPath,
    pub(crate) parameters: Vec<u8>,
    pub(crate) value: Vec<u8>,
    pub(crate) content_type: String,
}
