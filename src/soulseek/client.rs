// TODO: Remove this once we have a proper API
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use color_eyre::Result;
use governor::{
    Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, state::direct::NotKeyed,
};
use soulseek_rs::client::Client as SoulseekClient;
use std::num::NonZeroU32;

use crate::soulseek::types::SearchConfig;

// ============================================================================
// Trait for SoulSeek Client (for testability)
// ============================================================================

#[async_trait::async_trait]
pub trait SoulSeekClientTrait: Send + Sync {
    async fn login(&mut self, username: &str, password: &str) -> Result<()>;
    async fn search(&self, query: &str, timeout: Duration) -> Result<Vec<FileSearchResponse>>;
}

// Wrapper for real soulseek-rs-lib client
pub struct SoulSeekClientWrapper {
    client: Option<Arc<Mutex<SoulseekClient>>>,
}

impl SoulSeekClientWrapper {
    pub fn new() -> Self {
        Self { client: None }
    }

    /// Get access to the inner client for direct access (e.g., downloads)
    /// Returns a guard that can be used to call methods on the client
    pub fn get_client(&self) -> Option<std::sync::MutexGuard<'_, SoulseekClient>> {
        self.client.as_ref().and_then(|m| m.lock().ok())
    }
}

#[async_trait::async_trait]
impl SoulSeekClientTrait for SoulSeekClientWrapper {
    async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        log::debug!("Logging in to SoulSeek as user: {}", username);

        // Run the synchronous operations in a blocking task
        let username = username.to_string();
        let password = password.to_string();

        soulseek_rs::utils::logger::enable_buffering();

        let client = tokio::task::spawn_blocking(move || -> Result<SoulseekClient> {
            let mut client = SoulseekClient::new(&username, &password);
            client.connect();
            client.login()?;
            Ok(client)
        })
        .await??;

        self.client = Some(Arc::new(Mutex::new(client)));
        log::info!("Successfully logged in to SoulSeek");
        Ok(())
    }

    async fn search(&self, query: &str, timeout: Duration) -> Result<Vec<FileSearchResponse>> {
        let client_mutex = self
            .client
            .as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("Client not initialized"))?
            .clone();
        let query = query.to_string();

        // Run the synchronous search in a blocking task
        let search_results = tokio::task::spawn_blocking(move || {
            let client = client_mutex.lock().unwrap();
            client.search(&query, timeout)
        })
        .await??;

        // Convert SearchResult to FileSearchResponse
        let mut responses = Vec::new();
        for result in search_results {
            let files: Vec<FileInfo> = result
                .files
                .iter()
                .map(|f| {
                    // Convert HashMap<u32, u32> to HashMap<u8, u32>
                    let mut attrs = HashMap::new();
                    for (k, v) in &f.attribs {
                        if *k <= u8::MAX as u32 {
                            attrs.insert(*k as u8, *v);
                        }
                    }
                    FileInfo {
                        filename: f.name.clone(),
                        size: f.size,
                        attrs,
                    }
                })
                .collect();

            responses.push(FileSearchResponse {
                username: result.username.clone(),
                token: result.token.to_string(),
                files,
                slots_free: result.slots > 0,
                avg_speed: result.speed as f64,
                queue_length: 0, // Not available in SearchResult
            });
        }

        Ok(responses)
    }
}

// Placeholder types - these should match soulseek-rs-lib types
#[derive(Debug, Clone)]
pub struct FileSearchResponse {
    pub username: String,
    pub token: String,
    pub files: Vec<FileInfo>,
    pub slots_free: bool,
    pub avg_speed: f64,
    pub queue_length: u32,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub filename: String,
    pub size: u64,
    pub attrs: HashMap<u8, u32>, // Map from attribute key to value
}

// ============================================================================
// Client Context
// ============================================================================

type DirectRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

pub struct SoulSeekClientContext {
    pub client: Arc<dyn SoulSeekClientTrait>,
    pub rate_limiter: Arc<DirectRateLimiter>,
    pub config: SearchConfig,
    // Store the wrapper to access the inner client for downloads
    wrapper: Arc<SoulSeekClientWrapper>,
}

impl SoulSeekClientContext {
    pub async fn new(config: SearchConfig) -> Result<Self> {
        log::debug!("Creating SoulSeek client context");

        let mut wrapper = SoulSeekClientWrapper::new();
        wrapper.login(&config.username, &config.password).await?;

        // Wrap the wrapper in Arc for sharing
        let wrapper = Arc::new(wrapper);
        let client: Arc<dyn SoulSeekClientTrait> = wrapper.clone();

        let searches_per_time = config.searches_per_time.unwrap_or(34);
        let renew_time_secs = config.renew_time_secs.unwrap_or(220);

        log::debug!(
            "Rate limiter configured: {} searches per {}s",
            searches_per_time,
            renew_time_secs
        );

        // Create governor rate limiter
        let quota = Quota::with_period(std::time::Duration::from_secs(renew_time_secs as u64))
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid rate limit period"))?
            .allow_burst(NonZeroU32::new(searches_per_time).unwrap());

        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        log::info!("SoulSeek client context created successfully");
        Ok(Self {
            client,
            rate_limiter,
            config,
            wrapper,
        })
    }

    /// Get access to the inner soulseek client for downloads
    pub fn get_soulseek_client(&self) -> Option<std::sync::MutexGuard<'_, SoulseekClient>> {
        self.wrapper.get_client()
    }
}
