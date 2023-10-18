pub struct BlueskyConfiguration {
    pub request_content_type: String,
    pub xrpc_host: String,
    pub xrpc_connection_pooling: bool,
}
pub fn get_default_configuration() -> BlueskyConfiguration {
    BlueskyConfiguration {
        request_content_type: "application/json".to_string(),
        xrpc_host: "https://bsky.social".to_string(),
        xrpc_connection_pooling: true,
    }
}
