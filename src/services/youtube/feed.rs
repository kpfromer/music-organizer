use super::types::Feed;
use color_eyre::eyre::Result;
use html_parser::{Dom, Node};
use reqwest::Client;
use std::time::Duration;
use url::Url;

pub async fn get_channel_id(username: &str) -> Result<Option<String>> {
    let username = if username.starts_with("@") {
        username.to_string()
    } else {
        format!("@{}", username)
    };
    let url = format!("https://www.youtube.com/{}/videos", username);
    let response = Client::new()
        .get(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await?
        .error_for_status()?;
    let body = response.text().await?;
    let html = Dom::parse(&body)?;

    let mut stack = html.children;
    while let Some(node) = stack.pop() {
        match node {
            Node::Element(element) => {
                // Find something like:
                // <link rel="alternate" type="application/rss+xml" title="RSS" href="https://www.youtube.com/feeds/videos.xml?channel_id=UC1E-JS8L0j1Ei70D9VEFrPQ">
                if element.name == "link"
                    && element.attributes.get("type")
                        == Some(&Some("application/rss+xml".to_string()))
                    && element.attributes.get("title") == Some(&Some("RSS".to_string()))
                    && let Some(href) = element.attributes.get("href")
                    && let Some(href) = href
                {
                    let url = Url::parse(href)?;
                    let channel_id = url
                        .query_pairs()
                        .find(|(key, _)| key == "channel_id")
                        .map(|(_, value)| value.to_string());
                    if let Some(channel_id) = channel_id {
                        return Ok(Some(channel_id));
                    }
                }

                stack.extend(element.children);
            }
            Node::Text(_text) => {}
            Node::Comment(_comment) => {}
        }
    }

    Ok(None)
}

fn get_feed_url(channel_id: &str) -> String {
    format!(
        "https://www.youtube.com/feeds/videos.xml?channel_id={}",
        channel_id
    )
}

pub async fn fetch_feed(channel_id: &str) -> Result<Feed> {
    let feed_url = get_feed_url(channel_id);
    let response = Client::new()
        .get(feed_url)
        .timeout(Duration::from_secs(10))
        .send()
        .await?
        .error_for_status()?;
    let body = response.text().await?;
    let feed = serde_xml_rs::from_str::<Feed>(&body)?;
    Ok(feed)
}
