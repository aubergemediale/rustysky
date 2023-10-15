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
    login: &CreateSessionRequest,
    config: &BlueskyConfiguration,
) -> Result<CreateSessionResponse> {
    let url = format!("{}/xrpc/com.atproto.server.createSession", config.xrpc_host);

    let request_body = serde_json::to_string(login)?;

    let response_body = crate::xrpc::fetch(url, request_body).await?;

    let user_data: CreateSessionResponse = serde_json::from_str(&response_body)?;

    Ok(user_data)
}
