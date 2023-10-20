use anyhow::Result;
use log::{debug, info};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

static HTTP_DEBUG_LOGGING: AtomicBool = AtomicBool::new(false);

pub fn set_http_debug_logging(value: bool) {
    HTTP_DEBUG_LOGGING.store(value, Ordering::Relaxed);
}

static mut CLIENT: Option<Client> = None;
static CLIENT_LOCK: Mutex<()> = Mutex::new(());

pub fn get_client(use_connection_pooling: bool) -> Client {
    if !use_connection_pooling {
        return Client::new();
    }
    let _guard = CLIENT_LOCK.lock().unwrap();
    unsafe {
        if CLIENT.is_none() {
            CLIENT = Some(reqwest::Client::new());
        }
        CLIENT.as_ref().unwrap().clone()
    }
}

pub fn clear_client() {
    let _guard = CLIENT_LOCK.lock().unwrap();
    unsafe {
        CLIENT = None;
    }
}

pub async fn post<T: Serialize, R: DeserializeOwned>(
    url: String,
    request: T,
    use_connection_pooling: bool,
) -> Result<R, (Option<u16>, String)> {
    let client = get_client(use_connection_pooling);
    let body = serde_json::to_string(&request)
        .map_err(|err| (None, format!("Serialization error: {}", err)))?;
    let response = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .await
        .map_err(|err| (None, format!("Request error: {}", err)))?;
    handle_response::<R>(response).await
}

pub async fn post_auth<T: Serialize, R: DeserializeOwned>(
    url: String,
    access_jwt: &str,
    request: T,
    use_connection_pooling: bool,
) -> Result<R, (Option<u16>, String)> {
    let client = get_client(use_connection_pooling);
    let body = serde_json::to_string(&request)
        .map_err(|err| (None, format!("Serialization error: {}", err)))?;
    info!("{}", body);
    let response = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header("Authorization", format!("Bearer {}", access_jwt))
        .body(body)
        .send()
        .await
        .map_err(|err| (None, format!("Request error: {}", err)))?;
    handle_response::<R>(response).await
}

pub async fn post_refresh<R: DeserializeOwned>(
    url: String,
    refresh_jwt: &str,
    use_connection_pooling: bool,
) -> Result<R, (Option<u16>, String)> {
    let client = get_client(use_connection_pooling);
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", refresh_jwt))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .await
        .map_err(|err| (None, format!("Request error: {}", err)))?;
    handle_response::<R>(response).await
}

pub async fn get<T: DeserializeOwned>(
    url: &str,
    auth: &str,
    use_connection_pooling: bool,
) -> Result<T, (Option<u16>, String)> {
    let client = get_client(use_connection_pooling);
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", auth))
        .send()
        .await
        .map_err(|err| (None, format!("Request error: {}", err)))?;
    handle_response::<T>(response).await
}

async fn handle_response<R: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<R, (Option<u16>, String)> {
    if !response.status().is_success() {
        return Err((
            Some(response.status().as_u16()),
            format!("HTTP error with status code: {}", response.status()),
        ));
    }

    if HTTP_DEBUG_LOGGING.load(Ordering::Relaxed) {
        let headers = response.headers().clone();
        let raw_json = response
            .text()
            .await
            .map_err(|err| (None, format!("Failed to get response text: {}", err)))?;
        debug!("Response Headers:\n{:#?}", headers);
        debug!("Raw JSON Response: {}", raw_json);
        serde_json::from_str::<R>(&raw_json)
            .map_err(|err| (None, format!("Deserialization error: {}", err)))
    } else {
        response
            .json::<R>()
            .await
            .map_err(|err| (None, format!("Deserialization error: {}", err)))
    }
}
