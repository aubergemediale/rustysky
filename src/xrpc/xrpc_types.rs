use serde::Deserialize;
use std::str;

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
