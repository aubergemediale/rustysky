use serde::{Deserialize, Serialize};

pub struct BlueskyConfiguration {
    pub request_content_type: String,
    pub xrpc_host: String,
    pub xrpc_connection_pooling: bool,
}

#[derive(Debug, Deserialize)]
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

impl CreateSessionResponse {
    pub fn update_from_refresh(&mut self, refresh: &RefreshSessionResponse) {
        let mut updated = false;

        if self.access_jwt != refresh.access_jwt {
            log::info!("Session access_jwt updated.");
            self.access_jwt = refresh.access_jwt.clone();
            updated = true;
        }

        if self.refresh_jwt != refresh.refresh_jwt {
            log::info!("Session refresh_jwt updated.");
            self.refresh_jwt = refresh.refresh_jwt.clone();
            updated = true;
        }

        if self.handle != refresh.handle {
            log::info!("Session handle updated.");
            self.handle = refresh.handle.clone();
            updated = true;
        }

        if self.did != refresh.did {
            panic!("Did mismatch between create and refresh session responses");
        }

        if updated {
            log::info!("Session successfully refreshed.");
        } else {
            log::info!("No updates detected during session refresh.");
        }
    }
}

/*
export interface OutputSchema {
  accessJwt: string
  refreshJwt: string
  handle: string
  did: string
  [k: string]: unknown
}
*/
#[derive(Debug, Deserialize)]
pub struct RefreshSessionResponse {
    pub did: String,
    pub handle: String,
    #[serde(rename = "accessJwt")]
    pub access_jwt: String,
    #[serde(rename = "refreshJwt")]
    pub refresh_jwt: String,
}

#[derive(Serialize)]
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
