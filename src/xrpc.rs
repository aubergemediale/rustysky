use anyhow::anyhow;
use anyhow::Result;
use log::debug;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub async fn fetch<T: Serialize, R: DeserializeOwned>(url: String, request: T) -> Result<R> {
    let client = reqwest::Client::new();

    let body = serde_json::to_string(&request)?;

    let response = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .await?;

    if response.status().as_u16() != 200 {
        return Err(anyhow!(
            "Request failed with status code: {}",
            response.status()
        ));
    }

    debug!("Response Headers:\n{:#?}", response.headers());

    // Deserialize the response directly
    let result: R = response.json().await?;

    Ok(result)
}
