use env_logger::{Builder, Env};
use log::LevelFilter;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{env, process::exit};

struct BlueskyServer {
    username: String,
    password: String,
    hosturl: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserData {
    did: String,
    handle: String,
    email: String,
    #[serde(rename = "emailConfirmed")]
    email_confirmed: bool,
    #[serde(rename = "accessJwt")]
    access_jwt: String,
    #[serde(rename = "refreshJwt")]
    refresh_jwt: String,
}

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    identifier: String,
    password: String,
}

// todo: anyhow, log via trace, crate derivemore, derive from, into
// note: don't forget to look at problems (from clippy) and check rustfmt is working on save.

#[tokio::main]
async fn main() {
    configure_logging();

    let server = create_server().expect("Could not create server, exiting");

    let user: UserData;
    let mut errcount = 0;
    loop {
        let loginresult = server.login().await;
        match loginresult {
            Ok(u) => {
                info!("Login successful: {:#?}", u);
                user = u;
                break;
            }
            Err(e) => {
                if errcount >= 5 {
                    error!("Login failed too many times, exiting.");
                    exit(1);
                }
                errcount += 1;
                info!("Error: {e}. Login failed {} times, retrying.", errcount);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
    info!("Hello {}!", user.handle);
}

fn configure_logging() {
    let log_level = LevelFilter::Info;

    // Initialize the logger with the specified log level and console output
    Builder::from_env(Env::default().default_filter_or(format!("{:?}", log_level)))
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] {} - {}",
                record.level(),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.args()
            )
        })
        .init();
    // how can I add output to a file?
}

fn create_server() -> Option<BlueskyServer> {
    let buserkey: &str = "BLUESKY_USERNAME";
    let bpasswordkey: &str = "BLUESKY_PASSWORD";

    let username = env::var_os(buserkey)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    if username.is_empty() {
        println!("Could not find the BLUESKY_USERNAME environment variable or it was empty.");
        return None;
    }

    let password = env::var_os(bpasswordkey)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    if password.is_empty() {
        println!("Could not find the BLUESKY_PASSWORD environment variable or it was empty.");
        return None;
    }

    println!(
        "Using Bluesky credentials for {} with password from environment variable {}.",
        username, bpasswordkey
    );
    Some(BlueskyServer {
        username,
        password,
        hosturl: "https://bsky.social".to_string(),
    })
}

impl BlueskyServer {
    async fn login(&self) -> std::result::Result<UserData, Box<dyn std::error::Error>> {
        let url = format!("{}/xrpc/com.atproto.server.createSession", self.hosturl);
        let client = reqwest::Client::new();

        // Create a LoginRequest struct with the username and password
        let login_request = LoginRequest {
            identifier: self.username.to_string(),
            password: self.password.to_string(),
        };

        // Serialize the LoginRequest struct to JSON
        let body = serde_json::to_string(&login_request).expect("serialization error");

        let response = client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?; // using ? here will return early if there is a network failure

        // Check if the HTTP response status code is not 200 (successful login)
        if response.status().as_u16() != 200 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                response.status().to_string(),
            )));
        }

        println!("Headers:\n{:#?}", response.headers());

        let body = response.text().await?.to_string();
        let user_data: UserData = serde_json::from_str(&body).expect("Failed to parse JSON");
        Ok(user_data)
    }
}
