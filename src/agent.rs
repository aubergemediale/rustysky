use crate::types::*;
use anyhow::Result;

pub fn get_default_configuration() -> BlueskyConfiguration {
    BlueskyConfiguration {
        request_content_type: "application/json".to_string(),
        xrpc_host: "https://bsky.social".to_string(),
        xrpc_create_session: "/xrpc/com.atproto.server.createSession".to_string(),
        xrpc_connection_pooling: true,
    }
}

pub async fn create_session(
    request: &CreateSessionRequest,
    config: &BlueskyConfiguration,
) -> Result<CreateSessionResponse, (Option<u16>, String)> {
    let url = format!("{}{}", config.xrpc_host, config.xrpc_create_session);
    let user_data: CreateSessionResponse =
        crate::xrpc::fetch(url, request, config.xrpc_connection_pooling).await?;
    Ok(user_data)
}
