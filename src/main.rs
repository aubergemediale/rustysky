use anyhow::{anyhow, bail, Result};
use env_logger::{Builder, Env};
use log::LevelFilter;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

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
struct Credentials {
    identifier: String,
    password: String,
}

// todo: log via trace, crate derivemore, derive from, into
// note: don't forget to look at problems (from clippy) and check rustfmt is working on save.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    configure_logging("log.txt"); // pass a filename to log to a file, or "" for stdout

    let credentials = credentials_from_env()?;
    info!(
        "Using Bluesky credentials for {} from BLUESKY_USERNAME, BLUESKY_PASSWORd",
        credentials.identifier
    );

    let user: UserData;
    let mut errcount = 0;
    loop {
        let loginresult = login(&credentials, "https://bsky.social").await;
        match loginresult {
            Ok(u) => {
                info!("Login successful: {:#?}", u);
                user = u;
                break;
            }
            Err(e) => {
                if errcount >= 5 {
                    bail!("Login failed too many times, exiting.");
                }
                errcount += 1;
                info!("Error: {e}. Login failed {} times, retrying.", errcount);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
    info!("Hello {}!", user.handle);
    Ok(())
}

/// Configures the logger for the application, specifying the log output destination and format.
///
/// The `configure_logging` function allows you to set up the logger for your application,
/// determining whether log messages are printed to the standard output or saved to a log file.
/// You can specify a log filename to save log messages to a file, or provide an empty string to
/// log messages to the standard output.
///
/// # Arguments
///
/// * `log_filename` - A string representing the desired log output destination. If empty, log
///   messages will be printed to the standard output. If a filename is provided, log messages
///   will be saved to that file. You can specify a relative or absolute path as well, and the
///   function will create any necessary directories.
///
/// # Examples
///
/// To log messages to the standard output:
///
/// ```rust
/// configure_logging(""); // Log to standard output
/// ```
///
/// To log messages to a file named "mylog.txt" in the "logs" directory:
///
/// ```rust
/// configure_logging("mylog.txt"); // Log to "logs/mylog.txt" inside the "logs" directory
/// ```
///
/// To log messages to a file with an absolute path:
///
/// ```rust
/// configure_logging("/var/log/myapp.log"); // Log to "/var/log/myapp.log"
/// ```
///
/// # Note
///
/// The function creates the "logs" directory if it doesn't exist when logging to a file.
/// Be sure to "logs" to your .gitignore file if you don't want to commit the log files.
///
/// # Panics
///
/// The function will panic if it encounters errors during logger initialization.
///
fn configure_logging(log_filename: &str) {
    let log_level = LevelFilter::Debug;

    // Initialize the logger with the specified log level and console output
    let mut builder =
        Builder::from_env(Env::default().default_filter_or(format!("{:?}", log_level)));

    builder.format(|buf, record| {
        writeln!(
            buf,
            "[{}] {} - {}",
            record.level(),
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.args()
        )
    });

    if !log_filename.is_empty() {
        // Check if the provided log filename contains a path
        let full_path: String = if Path::new(log_filename).is_relative() {
            // If it's a relative path, prepend "logs/"
            format!("logs/{}", log_filename)
        } else {
            log_filename.to_string() // Use the provided path as is
        };

        if full_path.starts_with("logs/") {
            std::fs::create_dir_all("logs").expect("Can't create logs directory");
        }

        // If a log filename is provided, configure logging to log to the file
        let target = Box::new(File::create(full_path).expect("Can't create file"));
        builder
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{}:{} {} [{}] - {}",
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                    record.level(),
                    record.args()
                )
            })
            .target(env_logger::Target::Pipe(target))
            .filter(None, LevelFilter::Info);
    }

    // Initialize the logger
    builder.init()
}

fn credentials_from_env() -> Result<Credentials> {
    let bluesky_username_var: &str = "BLUESKY_USERNAME";
    let bluesky_password_var: &str = "BLUESKY_PASSWORD";

    let identifier = env::var_os(bluesky_username_var)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    if identifier.is_empty() {
        bail!("Could not find the BLUESKY_USERNAME environment variable or it was empty.");
    }

    let password = env::var_os(bluesky_password_var)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    if password.is_empty() {
        bail!("Could not find the BLUESKY_PASSWORD environment variable or it was empty.");
    }

    Ok(Credentials {
        identifier,
        password,
    })
}

async fn login(login: &Credentials, host: &str) -> Result<UserData> {
    let url = format!("{}/xrpc/com.atproto.server.createSession", host);
    let client = reqwest::Client::new();

    // Serialize the LoginRequest struct to JSON
    let body = serde_json::to_string(login)?;

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

    debug!("Headers:\n{:#?}", response.headers());

    let body = response.text().await?.to_string();
    let user_data: UserData = serde_json::from_str(&body)?;

    Ok(user_data)
}
