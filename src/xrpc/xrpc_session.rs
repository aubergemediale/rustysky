use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str;

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

    pub fn print_token_info(&self) {
        // Decode the access token's payload
        let access_parts: Vec<&str> = self.access_jwt.split('.').collect();
        if access_parts.len() == 3 {
            match base64url_decode(access_parts[1]) {
                Ok(decoded) => {
                    let payload = str::from_utf8(&decoded).unwrap_or_default();
                    println!("Access Token Payload: {}", payload);

                    let v: Value = serde_json::from_str(payload).unwrap_or_default();
                    if let Some(exp) = v.get("exp") {
                        println!("Access Token Expiration: {}", exp);
                    }
                }
                Err(e) => {
                    println!("Failed to decode access token payload: {}", e);
                }
            }
        } else {
            println!("Invalid access token format");
        }

        // Decode the refresh token's payload
        let refresh_parts: Vec<&str> = self.refresh_jwt.split('.').collect();
        if refresh_parts.len() == 3 {
            match base64url_decode(refresh_parts[1]) {
                Ok(decoded) => {
                    let payload = str::from_utf8(&decoded).unwrap_or_default();
                    println!("Refresh Token Payload: {}", payload);

                    let v: Value = serde_json::from_str(payload).unwrap_or_default();
                    if let Some(exp) = v.get("exp") {
                        println!("Refresh Token Expiration: {}", exp);
                    }
                }
                Err(e) => {
                    println!("Failed to decode refresh token payload: {}", e);
                }
            }
        } else {
            println!("Invalid refresh token format");
        }
    }
    pub fn session_needs_refresh(&self) -> bool {
        // Extract expiration from access token
        let access_parts: Vec<&str> = self.access_jwt.split('.').collect();
        if access_parts.len() != 3 {
            return false; // invalid token format, consider refresh
        }

        let decoded_payload = match base64url_decode(access_parts[1]) {
            Ok(decoded) => decoded,
            Err(_) => return false, // invalid payload, consider refresh
        };

        let payload_str = match std::str::from_utf8(&decoded_payload) {
            Ok(s) => s,
            Err(_) => return false, // invalid UTF-8, consider refresh
        };

        let payload_value: serde_json::Value = match serde_json::from_str(payload_str) {
            Ok(v) => v,
            Err(_) => return false, // invalid JSON, consider refresh
        };

        if let Some(exp) = payload_value.get("exp").and_then(|v| v.as_i64()) {
            let current_timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64;

            const BUFFER_TIME: i64 = 300; // 5 minutes in seconds
            if exp - current_timestamp <= BUFFER_TIME {
                return true; // token is about to expire or has expired
            }
        }

        false
    }
}

pub fn base64url_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let mut s = input.replace('-', "+").replace('_', "/");
    match s.len() % 4 {
        2 => s.push_str("=="),
        3 => s.push('='),
        0 => {}
        _ => return Err("Invalid base64 length"),
    }
    #[allow(deprecated)]
    base64::decode(&s).map_err(|_| "Invalid base64")
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
