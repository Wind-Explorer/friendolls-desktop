

#[derive(Clone)]
pub struct Clients {
    pub http_client: reqwest::Client,
    pub ws_client: Option<rust_socketio::client::Client>,
    pub is_ws_initialized: bool,
}

pub struct NetworkState {
    pub clients: Option<Clients>,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self { clients: None }
    }
}

pub fn init_network_state() -> NetworkState {
    let http_client = reqwest::ClientBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .user_agent("friendolls-desktop/0.1.0")
        .build()
        .expect("Client should build");

    NetworkState {
        clients: Some(Clients {
            http_client,
            ws_client: None,
            is_ws_initialized: false,
        }),
    }
}