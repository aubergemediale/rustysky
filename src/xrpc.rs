use anyhow::anyhow;
use anyhow::Result;
use log::debug;

pub async fn fetch(url: String, body: String) -> Result<String> {
    let client = reqwest::Client::new();

    let response = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .await?;

    // Check if the HTTP response status code is not 200 (successful login)
    if response.status().as_u16() != 200 {
        return Err(anyhow!(
            "Login failed with status code: {}",
            response.status()
        ));
    }

    debug!("Response Headers:\n{:#?}", response.headers());

    let body = response.text().await?.to_string();

    Ok(body)
}
