use serde::{Deserialize, Serialize};

/// Root feed element for YouTube RSS feed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feed {
    #[serde(rename = "link")]
    pub links: Vec<Link>,
    pub id: String,
    #[serde(rename = "yt:channelId")]
    pub channel_id: String,
    pub title: String,
    pub author: Author,
    pub published: String,
    #[serde(rename = "entry")]
    pub entries: Vec<Entry>,
}

/// Link element with rel and href attributes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    #[serde(rename = "@rel")]
    pub rel: String,
    #[serde(rename = "@href")]
    pub href: String,
}

/// Author element containing name and uri
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub uri: String,
}

/// Entry element representing a video in the feed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    #[serde(rename = "yt:videoId")]
    pub video_id: String,
    #[serde(rename = "yt:channelId")]
    pub channel_id: String,
    pub title: String,
    #[serde(rename = "link")]
    pub link: Link,
    pub author: Author,
    pub published: String,
    pub updated: String,
    #[serde(rename = "media:group")]
    pub media_group: MediaGroup,
}

/// Media group containing video metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaGroup {
    #[serde(rename = "media:title")]
    pub title: String,
    #[serde(rename = "media:content")]
    pub content: MediaContent,
    #[serde(rename = "media:thumbnail")]
    pub thumbnail: MediaThumbnail,
    #[serde(rename = "media:description")]
    pub description: String,
    #[serde(rename = "media:community")]
    pub community: MediaCommunity,
}

/// Media content element with attributes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaContent {
    #[serde(rename = "@url")]
    pub url: String,
    #[serde(rename = "@type")]
    pub content_type: String,
    #[serde(rename = "@width")]
    pub width: String,
    #[serde(rename = "@height")]
    pub height: String,
}

/// Media thumbnail element with attributes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaThumbnail {
    #[serde(rename = "@url")]
    pub url: String,
    #[serde(rename = "@width")]
    pub width: String,
    #[serde(rename = "@height")]
    pub height: String,
}

/// Media community element containing ratings and statistics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaCommunity {
    #[serde(rename = "media:starRating")]
    pub star_rating: MediaStarRating,
    #[serde(rename = "media:statistics")]
    pub statistics: MediaStatistics,
}

/// Media star rating with attributes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaStarRating {
    #[serde(rename = "@count")]
    pub count: String,
    #[serde(rename = "@average")]
    pub average: String,
    #[serde(rename = "@min")]
    pub min: String,
    #[serde(rename = "@max")]
    pub max: String,
}

/// Media statistics with views attribute
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaStatistics {
    #[serde(rename = "@views")]
    pub views: String,
}
