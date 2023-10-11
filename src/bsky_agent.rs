use anyhow::Result;

/// Returns the module name.
pub fn get_module_name() -> Result<String> {
    Ok("bsky_agent".to_string())
}
