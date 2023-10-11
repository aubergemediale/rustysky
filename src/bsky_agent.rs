use anyhow::Result;

/// Returns the module name.
pub fn get_module_name() -> Result<String> {
    Ok("bsky_agent".to_string())
}

/// Dummy call to test the module as an integration test.
pub fn call_host() -> Result<String> {
    Ok("called the api over http".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_module_name() {
        let result = get_module_name().unwrap();
        assert_eq!(result, "bsky_agent");
    }
}
