use serde::{Deserialize, Serialize};
use std::{env, process::exit};

struct BlueskyServer {
    username: String,
    password: String,
    hosturl: String,
}

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    identifier: String,
    password: String,
}

#[tokio::main]
async fn main() {
    let server = create_server();

    let _ = server.login().await;
}

fn create_server() -> BlueskyServer {
    let buserkey: &str = "BLUESKY_USERNAME";
    let bpasswordkey: &str = "BLUESKY_PASSWORD";
    let username = env::var_os(buserkey)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    if username.len() == 0 {
        println!("Could not find the BLUESKY_USERNAME environment variable or it was empty.");
        exit(1);
    }

    let password = env::var_os(bpasswordkey)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    if password.len() == 0 {
        println!("Could not find the BLUESKY_PASSWORD environment variable or it was empty.");
        exit(1);
    }
    println!(
        "Using Bluesky credentials for {} with password from environment variable {}.",
        username, bpasswordkey
    );
    return BlueskyServer {
        username: username,
        password: password,
        hosturl: "https://bsky.social".to_string(),
    };
}

impl BlueskyServer {
    async fn login(&self) -> std::result::Result<(), reqwest::Error>  {
        let url = format!("{}/xrpc/com.atproto.server.createSession", self.hosturl);
        let client = reqwest::Client::new();

        // Create a LoginRequest struct with the username and password
        let login_request = LoginRequest {
            identifier: self.username.to_string(),
            password: self.password.to_string(),
        };

        // Serialize the LoginRequest struct to JSON
        let body= serde_json::to_string(&login_request).expect("serialization error");

        let res = client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await;

        let u = res.unwrap();
        println!("Status: {}", u.status());
        println!("Headers:\n{:#?}", u.headers());
        println!("Body:\n{}", u.text().await.unwrap());
        Ok(())
    }
}
