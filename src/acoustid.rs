// TODO: Remove this once we have a proper API
#![allow(dead_code)]

use color_eyre::Result;
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
    get_rate_limiter().until_ready().await;

    let url = format!(
        "https://api.acoustid.org/v2/lookup?client={}&meta=recordings&duration={}&fingerprint={}",
        api_key, duration, fingerprint
    );

    let resp: AcoustIdResponse = client.get(&url).send().await?.json().await?;

    Ok(resp)
}
