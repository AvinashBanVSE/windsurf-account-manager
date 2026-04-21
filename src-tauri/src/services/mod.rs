pub mod auth_service;
pub mod auth_context;
pub mod windsurf_service;
pub mod proto_parser;
pub mod analytics_service;
pub mod devin_auth_service;

pub use auth_service::*;
pub use auth_context::*;
pub use windsurf_service::*;
// pub use proto_parser::*;
pub use analytics_service::*;  // Not used directly yet, commented to avoid warnings
pub use devin_auth_service::*;

use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use std::sync::{OnceLock, RwLock, Mutex};
use std::sync::Arc;

/// Globally shared HTTP client, uses RwLock to support rebuilding on failure
static GLOBAL_HTTP_CLIENT: OnceLock<RwLock<Arc<reqwest::Client>>> = OnceLock::new();

/// HTTP client specifically for googleapis (supports proxy)
static GOOGLE_API_CLIENT: OnceLock<RwLock<Arc<reqwest::Client>>> = OnceLock::new();

/// Proxy configuration cache
static PROXY_CONFIG: OnceLock<Mutex<ProxyConfig>> = OnceLock::new();

/// Proxy configuration structure
#[derive(Clone, Default)]
struct ProxyConfig {
    enabled: bool,
    url: Option<String>,
}

/// Consecutive failure counter, used to determine if client needs rebuilding
static CONSECUTIVE_FAILURES: AtomicU32 = AtomicU32::new(0);

/// Consecutive failure threshold, rebuild HTTP client when exceeded
const FAILURE_THRESHOLD: u32 = 3;  // Lower threshold for faster rebuilding

/// Create a well-configured HTTP client
fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        // Total request timeout: 30 seconds
        .timeout(std::time::Duration::from_secs(30))
        // Connection timeout: 10 seconds (avoid connection hanging)
        .connect_timeout(std::time::Duration::from_secs(10))
        // Connection pool configuration - more aggressive settings to avoid dead connections
        .pool_max_idle_per_host(2)  // Reduce number of idle connections
        .pool_idle_timeout(std::time::Duration::from_secs(30))  // Keep idle connections for 30 seconds (shortened)
        // TCP keep-alive configuration
        .tcp_keepalive(std::time::Duration::from_secs(15))  // Shorten orten orte intervalkeep intervallive interval
        // Do not auto-redirect, avoid infinite redirects
        .redirect(reqwest::redirect::Policy::limited(5))
        // Disable HTTP/2 (some servers have problematic HTTP/2 implementations)
        .http1_only()
        // Disable connection pool reuse (use new connection for each request, more reliable but slightly slower)
        // .no_proxy()  // If you need to completely disable proxy
        .build()
        .expect("Failed to create HTTP client")
}

/// Create HTTP client for googleapis (supports proxy)
fn create_google_api_client(proxy_url: Option<&str>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(15))
        .pool_max_idle_per_host(2)
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .tcp_keepalive(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(5))
        .http1_only();
    
    // If proxy address is provided, configure proxy
    if let Some(url) = proxy_url {
        if !url.is_empty() {
            match reqwest::Proxy::all(url) {
                Ok(proxy) => {
                    println!("[Google API Client] Using proxy: {}", url);
                    builder = builder.proxy(proxy);
                }
                Err(e) => {
                    println!("[Google API Client] Failed to parse proxy URL: {}", e);
                }
            }
        }
    }
    
    builder.build().expect("Failed to create Google API client")
}

/// Get globally shared HTTP client
pub fn get_http_client() -> Arc<reqwest::Client> {
    let client_lock = GLOBAL_HTTP_CLIENT.get_or_init(|| {
        RwLock::new(Arc::new(create_http_client()))
    });
    
    // Check if client needs rebuilding
    let failures = CONSECUTIVE_FAILURES.load(Ordering::Relaxed);
    if failures >= FAILURE_THRESHOLD {
        // Try to acquire write lock to rebuild client
        if let Ok(mut guard) = client_lock.try_write() {
            // Double check to avoid duplicate rebuilding
            if CONSECUTIVE_FAILURES.load(Ordering::Relaxed) >= FAILURE_THRESHOLD {
                println!("[HTTP Client] Rebuilding client after {} consecutive failures", failures);
                *guard = Arc::new(create_http_client());
                CONSECUTIVE_FAILURES.store(0, Ordering::Relaxed);
            }
        }
    }
    
    client_lock.read().unwrap().clone()
}

/// Report request success, reset failure count
pub fn report_request_success() {
    CONSECUTIVE_FAILURES.store(0, Ordering::Relaxed);
}

/// Report request failure, increment failure count
pub fn report_request_failure() {
    let count = CONSECUTIVE_FAILURES.fetch_add(1, Ordering::Relaxed) + 1;
    println!("[HTTP Client] Request failed, consecutive failures: {}", count);
    
    // If threshold is reached, trigger rebuild immediately
    if count >= FAILURE_THRESHOLD {
        println!("[HTTP Client] Threshold reached, triggering rebuild...");
        rebuild_http_client();
    }
}

/// Report network timeout error, rebuild client immediately
pub fn report_timeout_error() {
    println!("[HTTP Client] Timeout error detected, forcing client rebuild");
    rebuild_http_client();
}

/// Force rebuild HTTP client (for manual recovery)
pub fn rebuild_http_client() {
    let client_lock = GLOBAL_HTTP_CLIENT.get_or_init(|| {
        RwLock::new(Arc::new(create_http_client()))
    });
    
    if let Ok(mut guard) = client_lock.write() {
        println!("[HTTP Client] Force rebuilding client");
        *guard = Arc::new(create_http_client());
        CONSECUTIVE_FAILURES.store(0, Ordering::Relaxed);
    }
}

/// Update proxy configuration and rebuild Google API client
pub fn update_proxy_config(enabled: bool, url: Option<String>) {
    let config_lock = PROXY_CONFIG.get_or_init(|| {
        Mutex::new(ProxyConfig::default())
    });
    
    // Update configuration
    if let Ok(mut config) = config_lock.lock() {
        config.enabled = enabled;
        config.url = url.clone();
    }
    
    // Rebuild Google API client
    let client_lock = GOOGLE_API_CLIENT.get_or_init(|| {
        RwLock::new(Arc::new(create_google_api_client(None)))
    });
    
    if let Ok(mut guard) = client_lock.write() {
        let proxy_url = if enabled { url.as_deref() } else { None };
        println!("[Google API Client] Rebuilding with proxy: {:?}", proxy_url);
        *guard = Arc::new(create_google_api_client(proxy_url));
    }
}

/// Get HTTP client for googleapis (supports proxy)
pub fn get_google_api_client() -> Arc<reqwest::Client> {
    let config_lock = PROXY_CONFIG.get_or_init(|| {
        Mutex::new(ProxyConfig::default())
    });
    
    let (enabled, url) = {
        let config = config_lock.lock().unwrap();
        (config.enabled, config.url.clone())
    };
    
    let client_lock = GOOGLE_API_CLIENT.get_or_init(|| {
        let proxy_url = if enabled { url.as_deref() } else { None };
        RwLock::new(Arc::new(create_google_api_client(proxy_url)))
    });
    
    client_lock.read().unwrap().clone()
}
