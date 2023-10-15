use serde::{Deserialize, Serialize};

pub struct BlueskyConfiguration {
    pub request_content_type: String,
    pub xrpc_host: String,
    pub xrpc_create_session: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSessionResponse {
    pub did: String,
    pub handle: String,
    pub email: String,
    #[serde(rename = "emailConfirmed")]
    pub email_confirmed: bool,
    #[serde(rename = "accessJwt")]
    pub access_jwt: String,
    #[serde(rename = "refreshJwt")]
    pub refresh_jwt: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub identifier: String,
    pub password: String,
}
