mod http_client;
pub mod xrpc_session;
pub mod xrpc_types;
pub use http_client::clear_client;

use http_client::*;
use xrpc_session::*;
use xrpc_types::*;

use anyhow::Result;

use crate::types::BlueskyConfiguration;

pub async fn create_session(
    request: &CreateSessionRequest,
    config: &BlueskyConfiguration,
) -> Result<CreateSessionResponse, (Option<u16>, String)> {
    let url = format!("{}/xrpc/com.atproto.server.createSession", config.xrpc_host);
    crate::xrpc::post(url, request, config.xrpc_connection_pooling).await
}

pub async fn refresh_session(
    refresh_jwt: &str,
    config: &BlueskyConfiguration,
) -> Result<RefreshSessionResponse, (Option<u16>, String)> {
    let url = format!(
        "{}/xrpc/com.atproto.server.refreshSession",
        config.xrpc_host
    );

    crate::xrpc::post_refresh(url, refresh_jwt, config.xrpc_connection_pooling).await
}

pub async fn get_profile(
    session: &mut CreateSessionResponse,
    config: &BlueskyConfiguration,
) -> Result<ProfileViewDetailedResponse, (Option<u16>, String)> {
    if session.session_needs_refresh() {
        refresh_session(&session.refresh_jwt, config)
            .await
            .map(|refreshed_session| session.update_from_refresh(&refreshed_session))?;
    }

    let url = format!(
        "{}/xrpc/app.bsky.actor.getProfile?actor={}",
        config.xrpc_host, session.did
    );

    crate::xrpc::get_debug(&url, &session.access_jwt, config.xrpc_connection_pooling).await
}
