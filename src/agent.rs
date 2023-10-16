use crate::types::*;
use anyhow::Result;

pub fn get_default_configuration() -> BlueskyConfiguration {
    BlueskyConfiguration {
        request_content_type: "application/json".to_string(),
        xrpc_host: "https://bsky.social".to_string(),
        xrpc_create_session: "/xrpc/com.atproto.server.createSession".to_string(),
    }
}

pub async fn create_session(
    request: &CreateSessionRequest,
    config: &BlueskyConfiguration,
) -> Result<CreateSessionResponse> {
    let url = format!("{}/xrpc/com.atproto.server.createSession", config.xrpc_host);
    let user_data: CreateSessionResponse = crate::xrpc::fetch(url, request).await?;
    Ok(user_data)
}
