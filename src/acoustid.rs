// TODO: Remove this once we have a proper API
#![allow(dead_code)]

use color_eyre::Result;
use color_eyre::eyre::Context;
use governor::{
    Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, state::direct::NotKeyed,
};
use reqwest::Client;
use serde::Deserialize;
use std::num::NonZeroU32;
use std::sync::Arc;

type DirectRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

// Create a rate limiter: 1 request per second
static RATE_LIMITER: std::sync::OnceLock<Arc<DirectRateLimiter>> = std::sync::OnceLock::new();

fn get_rate_limiter() -> &'static Arc<DirectRateLimiter> {
    RATE_LIMITER.get_or_init(|| {
        let quota = Quota::per_second(NonZeroU32::new(1).unwrap());
        Arc::new(RateLimiter::direct(quota))
    })
}

#[derive(Debug, Deserialize)]
pub struct AcoustIdResponse {
    pub results: Vec<ResultItem>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ResultItem {
    pub id: String,
    #[serde(default)]
    pub recordings: Vec<AcoustIdRecording>,
    pub score: f64,
}

#[derive(Debug, Deserialize)]
pub struct AcoustIdRecording {
    pub artists: Option<Vec<AcoustIdArtist>>,
    pub duration: Option<f64>,
    pub id: String,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AcoustIdArtist {
    pub id: String,
    #[serde(default)]
    pub joinphrase: Option<String>,
    pub name: String,
}

/// Lookup a fingerprint using the AcoustID API with rate limiting
pub async fn lookup_fingerprint(
    client: &Client,
    api_key: &str,
    fingerprint: &str,
    duration: u32,
) -> Result<AcoustIdResponse> {
    // Wait for rate limit before making request
    log::debug!("Waiting for AcoustID rate limiter");
    get_rate_limiter().until_ready().await;

    let url = format!(
        "https://api.acoustid.org/v2/lookup?client={}&meta=recordings&duration={}&fingerprint={}",
        api_key, duration, fingerprint
    );

    log::debug!(
        "Making AcoustID API request (duration: {}s)\n\tURL:{}",
        duration,
        url
    );

    let resp: AcoustIdResponse = client
        .get(&url)
        .send()
        .await
        .wrap_err_with(|| format!("Failed to send AcoustID API request to {}", url))?
        .json()
        .await
        .wrap_err_with(|| format!("Failed to parse AcoustID API response from {}", url))?;

    log::debug!(
        "AcoustID response received: status={}, {} results",
        resp.status,
        resp.results.len()
    );

    log::info!(
        "AcoustID lookup complete: {} results found",
        resp.results.len()
    );

    Ok(resp)
}
