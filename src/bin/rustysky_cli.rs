use anyhow::bail;
use anyhow::Result;
use env_logger::{Builder, Env};
use log::info;
use log::LevelFilter;
use rustysky::types::get_default_configuration;
use rustysky::types::BlueskyConfiguration;
use rustysky::xrpc::clear_client;
use rustysky::xrpc::create_post;
use rustysky::xrpc::create_session;
use rustysky::xrpc::get_profile;
use rustysky::xrpc::refresh_session;
use rustysky::xrpc::set_http_debug_logging;
use rustysky::xrpc::CreatePostRequest;
use rustysky::xrpc::CreateSessionRequest;
use rustysky::xrpc::CreateSessionResponse;
use rustysky::xrpc::Post;
use rustysky::xrpc::ProfileViewDetailedResponse;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
const MAX_RETRIES: u32 = 5;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_http_debug_logging(true);
    configure_logging(""); // pass a filename to log to a file, or "" for stdout
    let config: BlueskyConfiguration = get_default_configuration();

    let create_session_request = credentials_from_env()?;
    info!(
        "Using Bluesky credentials for {} from BLUESKY_USERNAME, BLUESKY_PASSWORD",
        create_session_request.identifier
    );

    let mut session: CreateSessionResponse;
    let mut errcount = 0;
    loop {
        match create_session(&create_session_request, &config).await {
            Ok(response_data) => {
                info!("Login successful: {:#?}", response_data);
                session = response_data;
                break;
            }
            Err((Some(code), message)) => {
                if errcount >= MAX_RETRIES {
                    bail!("Login failed too many times, exiting.");
                }
                info!("HTTP error with code {}: {}", code, message);
                errcount += 1;
                info!("Login failed {} times, retrying.", errcount);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err((None, message)) => {
                bail!("Other error: {}", message);
            }
        }
    }
    info!("Hello {}!", session.handle);

    // get the full profile
    let profile: ProfileViewDetailedResponse;
    errcount = 0;
    loop {
        match get_profile(&mut session, &config).await {
            Ok(response_data) => {
                info!("Get Profile successful: {:#?}", response_data);
                profile = response_data;
                break;
            }
            Err((Some(code), message)) => {
                if errcount >= MAX_RETRIES {
                    bail!("Get Profile failed too many times, exiting.");
                }
                info!("HTTP error with code {}: {}", code, message);
                errcount += 1;
                info!("Get Profile failed {} times, retrying.", errcount);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err((None, message)) => {
                bail!("Other error: {}", message);
            }
        }
    }
    info!(
        "Congrats {}, you already have {} followers!",
        profile.display_name.as_deref().unwrap_or(&session.handle),
        profile.followers_count.unwrap_or(0)
    );

    // just to test this, I clear teh client
    clear_client();

    match refresh_session(&session.refresh_jwt, &config).await {
        Ok(response_data) => {
            session.update_from_refresh(&response_data);
            session.print_token_info();
        }
        Err((Some(code), message)) => {
            bail!("HTTP error with code {}: {}", code, message)
        }
        Err((None, message)) => {
            bail!("Other error: {}", message)
        }
    }
    let text = format!(
        "Hi @{}, here is a link: https://www.google.com",
        session.handle,
    );
    let post = Post::new(&text, &session.did)?;
    let did = session.did.clone();
    let create_post_request = CreatePostRequest::new(&did, post);
    match create_post(&create_post_request, &mut session, &config).await {
        Ok(response_data) => {
            info!("Post created successfully: {}", response_data);
        }
        Err((Some(code), message)) => {
            bail!("HTTP error with code {}: {}", code, message)
        }
        Err((None, message)) => {
            bail!("Other error: {}", message)
        }
    }

    /* create a post https://atproto.com/blog/create-post
       let post = Post::new(
           "This is a post with a link and a mention. The link is {{LINK}} and the mention is {{MENTION}}. Here is another link and another mention: {{LINK}}, {{MENTION}}",
           vec!["https://google.com".to_string(), "https://example.com".to_string()],
           vec![session.did.clone(), profile.did.clone() ]
       );
       match create_post(&post, &mut session, &config).await {
           Ok(response_data) => {
               info!("Post created successfully: {}", response_data);
           }
           Err((Some(code), message)) => {
               bail!("HTTP error with code {}: {}", code, message)
           }
           Err((None, message)) => {
               bail!("Other error: {}", message)
           }
       }
    */
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

fn credentials_from_env() -> Result<CreateSessionRequest> {
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

    Ok(CreateSessionRequest {
        identifier,
        password,
    })
}
