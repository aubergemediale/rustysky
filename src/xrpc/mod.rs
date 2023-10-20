mod http_client;
mod xrpc_post;
mod xrpc_session;
mod xrpc_types;

pub use http_client::{clear_client, set_http_debug_logging};
pub use xrpc_post::{CreatePostRequest, Post};
pub use xrpc_session::{CreateSessionRequest, CreateSessionResponse};
pub use xrpc_types::ProfileViewDetailedResponse;

use http_client::{get, post, post_auth, post_refresh};
use xrpc_session::RefreshSessionResponse;

use crate::types::BlueskyConfiguration;
use anyhow::Result;

const XRPC_ENDPOINT: &str = "/xrpc/";

fn create_url(host: &str, endpoint: &str) -> String {
    format!("{}{}{}", host, XRPC_ENDPOINT, endpoint)
}

pub async fn create_session(
    request: &CreateSessionRequest,
    config: &BlueskyConfiguration,
) -> Result<CreateSessionResponse, (Option<u16>, String)> {
    let url = create_url(&config.xrpc_host, "com.atproto.server.createSession");
    post(url, request, config.xrpc_connection_pooling).await
}

pub async fn refresh_session(
    refresh_jwt: &str,
    config: &BlueskyConfiguration,
) -> Result<RefreshSessionResponse, (Option<u16>, String)> {
    let url = create_url(&config.xrpc_host, "com.atproto.server.refreshSession");
    post_refresh(url, refresh_jwt, config.xrpc_connection_pooling).await
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
        "{}{}app.bsky.actor.getProfile?actor={}",
        config.xrpc_host, XRPC_ENDPOINT, session.did
    );

    get(&url, &session.access_jwt, config.xrpc_connection_pooling).await
}

pub async fn create_post(
    post_request: &CreatePostRequest,
    session: &mut CreateSessionResponse,
    config: &BlueskyConfiguration,
) -> Result<String, (Option<u16>, String)> {
    let url = create_url(&config.xrpc_host, "com.atproto.repo.createRecord");
    post_auth(
        url,
        &session.access_jwt,
        post_request,
        config.xrpc_connection_pooling,
    )
    .await
}
