use reqwest::Client;

use color_eyre::eyre::{Context, Result};
use serde::Deserialize;
use url::Url;

pub const APP_NAME: &str = "Music Manager";
pub const APP_IDENTIFIER: &str = "kpfromer-music-manager";

#[derive(Debug, Deserialize)]
pub struct PlexPinResponse {
    pub id: i32,
    pub code: String,
}

/// Create a new Plex pin (used for login via device code flow)
/// Returns the response from the Plex server.
///
/// # Arguments
/// - `client`: reqwest Client instance
/// - `client_identifier`: The Plex client identifier (unique per app install)
/// - `product_name`: The display name for your app (shown in Plex login)
pub async fn create_plex_pin(client: &Client) -> Result<PlexPinResponse> {
    let url = "https://plex.tv/api/v2/pins?strong=true";
    let res = client
        .post(url)
        .header("Accept", "application/json")
        .header("X-Plex-Product", APP_NAME)
        .header("X-Plex-Client-Identifier", APP_IDENTIFIER)
        .send()
        .await?
        .error_for_status()?;
    res.json::<PlexPinResponse>()
        .await
        .wrap_err("Failed to parse plex pin response")
}

/// Constructs the Plex Auth App URL to redirect the user for authentication.
///
/// This URL can be used to direct the user to the Plex.tv authentication flow in their browser.
/// See: https://app.plex.tv/auth
///
/// # Arguments
/// - `pin_code`: The code from the generated Plex PIN
/// - `forward_url`: The URL to which the user should be returned after authenticating (must be URL-encoded)
///
/// # Returns
/// A full URL string that the user should be redirected/opened to authenticate.
///
/// # Example
/// ```
/// let url = construct_auth_app_url(
///     "pinCodeAbc",
///     "https://my-cool-plex-app.com/plex-auth/callback",
/// );
/// ```
pub fn construct_auth_app_url(pin_code: &str, forward_url: &str) -> Result<String> {
    use urlencoding::encode;
    let tuples = [
        ("context[device][product]", APP_NAME),
        ("clientID", APP_IDENTIFIER),
        ("code", pin_code),
        ("forwardUrl", forward_url),
    ];
    let url_params = tuples
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    // Not using this because we need it to be exact auth#?thing=
    // Url::parse_with_params("https://app.plex.tv/auth#", tuples).wrap_err("Failed to parse URL")
    Ok(format!("https://app.plex.tv/auth#?{}", url_params))
}

#[derive(Debug, Deserialize)]
pub struct PlexAuthResponse {
    #[serde(rename = "authToken")]
    pub auth_token: Option<String>,
}

/// If you're using the Forwarding flow, check the stored PIN id from the PIN creation step.
/// If the PIN has been claimed, the authToken field in the response will contain the user's Access Token you need to make API calls on behalf of the user.
/// If authentication failed, the authToken field will remain null.
pub async fn poll_for_plex_auth(client: &Client, pin_id: i32) -> Result<PlexAuthResponse> {
    let url = Url::parse(&format!("https://plex.tv/api/v2/pins/{}", pin_id))
        .wrap_err("Failed to parse plex pins URL")?;

    let res = client
        .get(url)
        .header("Accept", "application/json")
        .header("X-Plex-Client-Identifier", APP_IDENTIFIER)
        .send()
        .await?
        .error_for_status()?;

    res.json::<PlexAuthResponse>()
        .await
        .wrap_err("Failed to parse plex auth response")
}

#[derive(Debug, Deserialize)]
pub struct PlexResource {
    #[serde(rename = "accessToken")]
    pub access_token: Option<String>,
    // #[serde(rename = "clientIdentifier")]
    // pub client_identifier: String,
    /// The name of the server
    pub name: String,
}

/// Queries the Plex resources endpoint to get servers/devices associated with an account.
///
/// # Arguments
/// * `client` - An instance of reqwest::Client to use for the request.
/// * `client_identifier` - The Plex client identifier string.
/// * `user_token` - The user authentication token.
///
/// # Returns
/// Returns a serde_json::Value with the parsed response.
///
/// # Example
/// ```no_run
/// let res = get_plex_resources(&client, "clientId123", "token123").await?;
/// println!("{:#?}", res);
/// ```
pub async fn get_plex_resources(client: &Client, user_token: &str) -> Result<Vec<PlexResource>> {
    let url =
        "https://clients.plex.tv/api/v2/resources?includeHttps=1&includeRelay=1&includeIPv6=1";
    client
        .get(url)
        .header("Accept", "application/json")
        .header("X-Plex-Product", APP_NAME)
        .header("X-Plex-Client-Identifier", APP_IDENTIFIER)
        .header("X-Plex-Token", user_token)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<PlexResource>>()
        .await
        .wrap_err("Failed to deserialize Plex resources response")
}
