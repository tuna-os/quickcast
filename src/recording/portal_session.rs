use std::os::fd::OwnedFd;

use anyhow::{Context, Result};
use gtk::prelude::*;

use crate::screencast_portal::{
    CursorMode, PersistMode, Proxy, Session, SourceType, Stream, WindowIdentifier,
};

pub(super) async fn start(
    cursor_mode: CursorMode,
    source_type: SourceType,
    is_multiple_sources: bool,
    restore_token: Option<&str>,
    persist_mode: PersistMode,
    parent_window: Option<&impl IsA<gtk::Window>>,
) -> Result<(Session, Vec<Stream>, Option<String>, OwnedFd)> {
    let proxy = Proxy::new()
        .await
        .context("Failed to create screencast proxy")?;

    tracing::debug!(
        version = ?proxy.version(),
        available_cursor_modes = ?proxy.available_cursor_modes(),
        available_source_types = ?proxy.available_source_types(),
        "Created screencast proxy"
    );

    let session = proxy
        .create_session()
        .await
        .context("Failed to create session")?;

    tracing::debug!(
        ?cursor_mode,
        ?source_type,
        is_multiple_sources,
        has_restore_token = restore_token.is_some(),
        ?persist_mode,
        "Selecting sources"
    );

    session
        .select_sources(
            source_type,
            is_multiple_sources,
            cursor_mode,
            restore_token,
            persist_mode,
        )
        .await
        .context("Failed to select sources")?;

    let window_identifier = if let Some(window) = parent_window {
        WindowIdentifier::new(window.as_ref()).await
    } else {
        WindowIdentifier::None
    };

    tracing::debug!(%window_identifier, "Starting session");

    let (streams, restore_token) = session
        .start(window_identifier)
        .await
        .context("Failed to start session")?;

    let fd = session
        .open_pipe_wire_remote()
        .await
        .context("Failed to open pipewire remote")?;

    Ok((session, streams, restore_token, fd))
}
