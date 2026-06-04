use reqwest::Client;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use log::{info, debug};

static GLOBAL_CLIENT: LazyLock<Mutex<Option<Client>>> = LazyLock::new(|| Mutex::new(None));

pub fn init_client(max_connections: usize) {
    let mut guard = GLOBAL_CLIENT.lock().unwrap();
    if guard.is_some() {
        return;
    }
    info!("Initializing global HTTP client (max_connections={})", max_connections);
    *guard = Some(create_client_with_proxy(max_connections));
}

pub fn shared_client() -> Client {
    let guard = GLOBAL_CLIENT.lock().unwrap();
    guard.clone().expect("HTTP client not initialized, call init_client() first")
}

fn create_client_with_proxy(max_connections: usize) -> Client {
    let mut builder = Client::builder()
        .pool_max_idle_per_host(max_connections)
        .timeout(Duration::from_secs(60));

    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("ALL_PROXY"))
        .or_else(|_| std::env::var("all_proxy"))
    {
        debug!("HTTP client using proxy: {}", proxy_url);
        if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
            builder = builder.proxy(proxy);
        }
    }

    builder.build().expect("Failed to create HTTP client")
}