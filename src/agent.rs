use crate::types::*;
use anyhow::Result;

pub fn get_default_configuration() -> BlueskyConfiguration {
    BlueskyConfiguration {
        request_content_type: "application/json".to_string(),
        xrpc_host: "https://bsky.social".to_string(),
        xrpc_connection_pooling: true,
    }
}

pub async fn create_session(
    request: &CreateSessionRequest,
    config: &BlueskyConfiguration,
) -> Result<CreateSessionResponse, (Option<u16>, String)> {
    let url = format!("{}/xrpc/com.atproto.server.createSession", config.xrpc_host);
    let response: CreateSessionResponse =
        crate::xrpc::post(url, request, config.xrpc_connection_pooling).await?;
    Ok(response)
}

pub async fn refresh_session(
    refresh_jwt: &str,
    config: &BlueskyConfiguration,
) -> Result<RefreshSessionResponse, (Option<u16>, String)> {
    let url = format!(
        "{}/xrpc/com.atproto.server.refreshSession",
        config.xrpc_host
    );
    let response: RefreshSessionResponse =
        crate::xrpc::post_refresh(url, refresh_jwt, config.xrpc_connection_pooling).await?;
    Ok(response)
}

pub async fn get_profile(
    did: &str,
    auth: &str,
    config: &BlueskyConfiguration,
) -> Result<ProfileViewDetailedResponse, (Option<u16>, String)> {
    let url = format!(
        "{}/xrpc/app.bsky.actor.getProfile?actor={}",
        config.xrpc_host, did
    );
    let response: ProfileViewDetailedResponse =
        crate::xrpc::get_debug(&url, auth, config.xrpc_connection_pooling).await?;
    Ok(response)
}
