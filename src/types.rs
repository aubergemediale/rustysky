use serde::{Deserialize, Serialize};

pub struct BlueskyConfiguration {
    pub request_content_type: String,
    pub xrpc_host: String,
    pub xrpc_connection_pooling: bool,
    pub xrpc_create_session: String,
    pub xrpc_profile_view_detailed: String,
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

#[derive(Debug, Deserialize)]
pub struct ProfileViewDetailedResponse {
    pub did: String,
    pub handle: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    #[serde(rename = "followersCount")]
    pub followers_count: Option<i32>,
    #[serde(rename = "followsCount")]
    pub follows_count: Option<i32>,
    #[serde(rename = "postsCount")]
    pub posts_count: Option<i32>,
    #[serde(rename = "indexedAt")]
    pub indexed_at: Option<String>,
}
