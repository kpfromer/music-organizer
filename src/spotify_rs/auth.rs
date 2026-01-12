use std::collections::HashMap;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::{
    alphabet,
    engine::{self, general_purpose},
};
use color_eyre::Result;
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::spotify_rs::types::{OAuthSession, SpotifyAuthResponse, SpotifyTokenResponse};

const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";

const CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

/// Generate a cryptographically secure random string for PKCE
fn generate_random_string(length: usize) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            const CHARSET: &[u8] =
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
            CHARSET[rng.random_range(0..CHARSET.len())] as char
        })
        .collect()
}

/// Generate PKCE code verifier (43-128 characters)
fn generate_code_verifier() -> String {
    generate_random_string(128)
}

/// Generate PKCE code challenge from verifier using S256 method
fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

/// Generate a random state parameter for CSRF protection
fn generate_state() -> String {
    generate_random_string(16)
}

/// Initiate Spotify OAuth flow with PKCE
/// Returns the authorization URL and creates an OAuth session
pub fn initiate_oauth(client_id: &str, redirect_uri: &str) -> (SpotifyAuthResponse, OAuthSession) {
    let code_verifier = generate_code_verifier();
    let state = generate_state();

    let scope = "playlist-read-private playlist-read-collaborative user-library-read";

    let auth_url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&state={}&scope={}",
        SPOTIFY_AUTH_URL,
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(&state),
        urlencoding::encode(scope)
    );

    let session = OAuthSession {
        code_verifier,
        state: state.clone(),
        created_at: chrono::Utc::now().timestamp(),
    };

    let response = SpotifyAuthResponse { auth_url, state };

    (response, session)
}

#[derive(Debug, thiserror::Error)]
pub enum ExchangeCodeForTokenError {
    #[error("Invalid code: {reason}")]
    InvalidCode { reason: String },
    #[error("Failed to send http request: {0}")]
    FailedToSendRequest(reqwest::Error),
    #[error("Failed to parse response")]
    FailedToParseResponse(reqwest::Error),
}

/// Exchange authorization code for access token
/// https://developer.spotify.com/documentation/web-api/tutorials/code-flow
pub async fn exchange_code_for_token(
    // The spotify client id
    client_id: &str,
    // The spotify client secret
    client_secret: &str,
    // The authorization code
    code: &str,
    // The exact redirect URI that was used to initiate the OAuth flow (it's unused though )
    redirect_uri: &str,
) -> Result<SpotifyTokenResponse, ExchangeCodeForTokenError> {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("grant_type", "authorization_code");
    params.insert("code", code);
    params.insert("redirect_uri", redirect_uri);

    let response = client
        .post(SPOTIFY_TOKEN_URL)
        // This automatically serializes to x-www-form-urlencoded and sets the header (as required by spotify)
        .form(&params)
        .header(
            "Authorization",
            format!(
                "Basic {}",
                CUSTOM_ENGINE.encode(format!("{}:{}", client_id, client_secret))
            ),
        )
        .send()
        .await
        .map_err(|error| ExchangeCodeForTokenError::FailedToSendRequest(error))?;

    if !response.status().is_success() {
        return Err(ExchangeCodeForTokenError::InvalidCode {
            reason: response
                .text()
                .await
                .unwrap_or("Failed to get error text".to_string()),
        });
    }

    let token_response: SpotifyTokenResponse = response
        .json()
        .await
        .map_err(|error| ExchangeCodeForTokenError::FailedToParseResponse(error))?;

    Ok(token_response)
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshTokenError {
    #[error("Invalid refresh token: {reason}")]
    InvalidRefreshToken { reason: String },
    #[error("Failed to send http request: {0}")]
    FailedToSendRequest(reqwest::Error),
    #[error("Failed to parse response: {0}")]
    FailedToParseResponse(reqwest::Error),
}

/// Refresh an access token using a refresh token
pub async fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<SpotifyTokenResponse, RefreshTokenError> {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("grant_type", "refresh_token");
    params.insert("refresh_token", refresh_token);
    params.insert("client_id", client_id);

    let response = client
        .post(SPOTIFY_TOKEN_URL)
        .form(&params)
        .header(
            "Authorization",
            format!(
                "Basic {}",
                CUSTOM_ENGINE.encode(format!("{}:{}", client_id, client_secret))
            ),
        )
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|error| RefreshTokenError::FailedToSendRequest(error))?;

    if !response.status().is_success() {
        return Err(RefreshTokenError::InvalidRefreshToken {
            reason: response
                .text()
                .await
                .unwrap_or("Failed to get error text".to_string()),
        });
    }

    let token_response: SpotifyTokenResponse = response
        .json()
        .await
        .map_err(|error| RefreshTokenError::FailedToParseResponse(error))?;

    Ok(token_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code_verifier() {
        let verifier = generate_code_verifier();
        assert_eq!(verifier.len(), 128);
        // Verify it only contains allowed characters
        assert!(verifier.chars().all(|c| c.is_ascii_alphanumeric()
            || c == '-'
            || c == '.'
            || c == '_'
            || c == '~'));
    }

    #[test]
    fn test_generate_code_challenge() {
        let verifier =
            "test_verifier_with_sufficient_length_for_pkce_requirements_to_be_met_and_valid";
        let challenge = generate_code_challenge(verifier);
        // Challenge should be base64url encoded SHA256 hash (43 characters without padding)
        assert_eq!(challenge.len(), 43);
        assert!(
            challenge
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
    }

    #[test]
    fn test_generate_state() {
        let state = generate_state();
        assert_eq!(state.len(), 16);
    }

    #[test]
    fn test_initiate_oauth() {
        let client_id = "test_client_id";
        let redirect_uri = "http://localhost:3000/callback";
        let (response, session) = initiate_oauth(client_id, redirect_uri);

        assert!(response.auth_url.starts_with(SPOTIFY_AUTH_URL));
        assert!(response.auth_url.contains(client_id));
        assert!(response.auth_url.contains("code_challenge_method=S256"));
        assert_eq!(response.state, session.state);
        assert_eq!(session.code_verifier.len(), 128);
    }
}
