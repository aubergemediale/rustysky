use anyhow::Result;
use rustysky::bsky_agent::call_host;

#[test]
fn test_call_host() -> Result<()> {
    let response = call_host()?;
    assert_eq!(response, "called the api over http");
    Ok(())
}
