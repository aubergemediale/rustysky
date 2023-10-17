use anyhow::Result;
use log::debug;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::{Mutex, Once};

static INIT: Once = Once::new();
static mut CLIENT: Option<Client> = None;
static CLIENT_LOCK: Mutex<()> = Mutex::new(());

/// Retrieves the globally cached `reqwest::Client` instance, creating it if it doesn't exist.
///
/// This function makes use of the `Once` construct to ensure the client is initialized only once,
/// even if called from multiple threads.
///
/// # Returns
///
/// Returns a reference to the globally cached `reqwest::Client` instance.
///
/// # Safety
///
/// This function uses unsafe code internally to deal with mutable statics.
pub fn get_client() -> &'static Client {
    unsafe {
        INIT.call_once(|| {
            CLIENT = Some(Client::new());
        });
        CLIENT.as_ref().unwrap()
    }
}

/// Clears the globally cached `reqwest::Client` instance.
///
/// This function might be used in scenarios where a fresh start is needed, or to release resources
/// held by the client. After calling this function, the next call to `get_client()` will create a
/// new client instance.
///
/// # Safety
///
/// This function uses unsafe code internally to deal with mutable statics.
pub fn clear_client() {
    let _guard = CLIENT_LOCK.lock().unwrap();
    unsafe {
        CLIENT = None;
    }
}

/// Asynchronously sends a POST request to the specified URL with the provided request body.
///
/// This function is designed to serialize the request, send it, and then deserialize the response.
/// Depending on the `use_connection_pooling` flag, it either uses a globally cached `reqwest::Client`
/// or creates a new instance for each request.
///
/// The function can return two types of errors:
/// 1. If the server responds with an HTTP error code (status > 200), the function returns a tuple
///    containing the HTTP error code (as `Option<u16>`) and a corresponding error message (as `String`).
/// 2. For other errors (like serialization errors, request errors, or deserialization errors),
///    the function returns a tuple with `None` for the HTTP error code and the error message.
///
/// # Parameters
///
/// - `url`: The URL to which the POST request should be sent.
/// - `request`: The request body data that needs to be serialized and sent.
/// - `use_connection_pooling`: A flag indicating whether to use the globally cached `reqwest::Client`
///                             instance (if set to `true`) or create a new `reqwest::Client` instance
///                             (if set to `false`).
///
/// # Returns
///
/// - `Ok(R)`: Successful response from the server, where `R` is the deserialized response type.
/// - `Err((Option<u16>, String))`: An error occurred. The `Option<u16>` indicates the HTTP error code
///                                (if applicable), and `String` provides a corresponding error message.
pub async fn fetch<T: Serialize, R: DeserializeOwned>(
    url: String,
    request: T,
    use_connection_pooling: bool,
) -> Result<R, (Option<u16>, String)> {
    let client = if use_connection_pooling {
        get_client().clone() // Clone the reference to the global client (not the instance)
    } else {
        reqwest::Client::new()
    };

    let body = match serde_json::to_string(&request) {
        Ok(b) => b,
        Err(err) => return Err((None, format!("Serialization error: {}", err))),
    };

    let response = match client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .await
    {
        Ok(res) => res,
        Err(err) => return Err((None, format!("Request error: {}", err))),
    };

    debug!("Response Headers:\n{:#?}", response.headers());

    if response.status().as_u16() > 200 {
        return Err((
            Some(response.status().as_u16()),
            format!("HTTP error with status code: {}", response.status()),
        ));
    }

    match response.json::<R>().await {
        Ok(res_data) => Ok(res_data),
        Err(err) => Err((None, format!("Deserialization error: {}", err))),
    }
}
