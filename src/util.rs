//! Contains helpers for:
//!   locking/unlocking
//!   exec_prompt
//!   formatting secrets

use crate::error::Error;
use crate::proxy::SecretStruct;
use crate::proxy::prompt::{Completed, PromptProxy};
use crate::proxy::service::ServiceProxy;
use crate::session::Session;
use crate::session::encrypt;
use crate::ss::SS_DBUS_NAME;

use futures_util::StreamExt;
use zbus::{ObjectPath, OwnedVariant, proxy::CacheProperties};

// Helper enum for locking
pub(crate) enum LockAction {
    Lock,
    Unlock,
}

pub(crate) async fn lock_or_unlock(
    conn: zbus::Connection,
    service_proxy: &ServiceProxy<'_>,
    object_path: &ObjectPath<'_>,
    lock_action: LockAction,
) -> Result<(), Error> {
    let objects = vec![object_path];

    let lock_action_res = match lock_action {
        LockAction::Lock => service_proxy.lock(objects).await?,
        LockAction::Unlock => service_proxy.unlock(objects).await?,
    };

    if lock_action_res.object_paths.is_empty() {
        exec_prompt(conn, &lock_action_res.prompt).await?;
    }
    Ok(())
}

pub(crate) fn format_secret(
    session: &Session,
    secret: &[u8],
    content_type: &str,
) -> Result<SecretStruct, Error> {
    let content_type = content_type.to_owned();

    if let Some(session_key) = session.get_aes_key() {
        let mut aes_iv = [0; 16];
        getrandom::fill(&mut aes_iv).expect("platform RNG failed");

        let encrypted_secret = encrypt(secret, session_key, &aes_iv);

        // Construct secret struct
        let parameters = aes_iv.to_vec();
        let value = encrypted_secret;

        Ok(SecretStruct {
            session: session.object_path.clone(),
            parameters,
            value,
            content_type,
        })
    } else {
        // just Plain for now
        let parameters = Vec::new();
        let value = secret.to_vec();

        Ok(SecretStruct {
            session: session.object_path.clone(),
            parameters,
            value,
            content_type,
        })
    }
}

// TODO: Users could pass their own window ID in.
const NO_WINDOW_ID: &str = "";

pub(crate) async fn exec_prompt(
    conn: zbus::Connection,
    prompt: &ObjectPath<'_>,
) -> Result<OwnedVariant, Error> {
    let prompt_proxy = PromptProxy::builder(&conn)
        .destination(SS_DBUS_NAME)
        .path(prompt)
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    let mut receive_completed_iter = prompt_proxy.receive_completed().await?;
    prompt_proxy.prompt(NO_WINDOW_ID).await?;

    handle_signal(receive_completed_iter.next().await)
}

fn handle_signal(signal: Option<Completed>) -> Result<OwnedVariant, Error> {
    // The signal stream ends without yielding a `Completed` signal when the
    // connection it was created from is closed, so the prompt can no longer
    // complete and we have no way of knowing whether it was accepted.
    let signal = signal.ok_or(Error::PromptDisconnected)?;

    let args = signal.args()?;
    if args.dismissed {
        Err(Error::Prompt)
    } else {
        OwnedVariant::try_from(args.result).map_err(From::from)
    }
}

pub(crate) fn handle_conn_error(e: zbus::Error) -> Error {
    match e {
        zbus::Error::InterfaceNotFound | zbus::Error::Address(_) => Error::Unavailable,
        zbus::Error::InputOutput(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Error::Unavailable
        }
        e => e.into(),
    }
}
