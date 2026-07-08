//! Data types for the Podbean API client.
//!
//! This module defines the various data structures used to represent
//! Podbean API resources and responses.

use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Response from OAuth token endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// OAuth access token
    pub access_token: String,

    /// Token type (usually "Bearer")
    pub token_type: String,

    /// Token validity in seconds
    pub expires_in: u64,

    /// OAuth scope
    pub scope: Option<String>,

    /// Refresh token for obtaining a new access token
    pub refresh_token: Option<String>,
}

/// Authentication token with metadata.
#[derive(Debug, Clone)]
pub(crate) struct AuthToken {
    access_token: String,
    token_type: String,
    expires_in: u64,
    // scope: Option<String>,
    refresh_token: Option<String>,
    created_at: Instant,
}

impl AuthToken {
    /// Checks if the token is expired.
    ///
    /// Considers a token expired if it has less than 5 minutes of validity left.
    pub(crate) fn is_expired(&self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.created_at);

        // Consider token expired if less than 5 minutes remaining
        elapsed.as_secs() + 300 > self.expires_in
    }

    /// Gets the access token string.
    pub(crate) fn access_token(&self) -> &str {
        &self.access_token
    }

    /// Gets the token type.
    pub(crate) fn token_type(&self) -> &str {
        &self.token_type
    }

    /// Gets the refresh token, if any.
    pub(crate) fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }
}

impl From<TokenResponse> for AuthToken {
    fn from(response: TokenResponse) -> Self {
        Self {
            access_token: response.access_token,
            token_type: response.token_type,
            expires_in: response.expires_in,
            // scope: response.scope,
            refresh_token: response.refresh_token,
            created_at: Instant::now(),
        }
    }
}

/// Token for a specific podcast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastToken {
    /// Unique identifier for the podcast
    pub podcast_id: String,

    /// Access token for the podcast
    pub access_token: String,

    /// Token validity in seconds
    pub expires_in: u64,
}

/// Response for a multiple podcasts token request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplePodcastsToken {
    /// Map of podcast tokens
    pub podcast_tokens: Vec<PodcastToken>,
}

/// Represents a media item in Podbean.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    /// Unique identifier for the media
    pub media_key: String,

    /// Title of the media
    pub title: String,

    /// Description or content
    pub content: String,

    /// Status (e.g., "finished", "transcoding")
    pub status: String,

    /// URL to the media file
    pub media_url: String,

    /// URL to the logo/artwork
    pub logo_url: Option<String>,

    /// URL to play the media
    pub player_url: Option<String>,

    /// When the media was published
    pub publish_time: Option<String>,

    /// When the media was created
    pub created_at: String,

    /// Duration in seconds
    pub duration: Option<u64>,
}

/// Response for a list of media items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaListResponse {
    /// Total number of media items
    pub count: u32,

    /// List of media items
    pub media: Vec<MediaItem>,
}

/// Represents a podcast episode.
/// https://developers.podbean.com/podbean-api-docs/#EpisodeObject
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Episode {
    /// Unique identifier for the episode
    pub id: String,

    /// Unique identifier for the podcast
    pub podcast_id: String,

    /// Episode title
    pub title: String,

    /// Episode description or show notes
    pub content: String,

    /// URL to the logo/artwork
    pub logo: Option<String>,

    /// Media URL for the episode
    pub media_url: String,

    /// URL to play the episode
    pub player_url: String,

    /// Permalink URL for the episode
    pub permalink_url: String,

    /// When the episode was published (Unix timestamp)
    pub publish_time: u64,

    /// Duration in seconds
    pub duration: Option<u64>,

    /// Publication status (e.g., "published", "draft")
    pub status: String,

    /// Type of episode (e.g., "public", "premium", "private")
    #[serde(rename = "type")]
    pub episode_type: String,

    /// Season number of the episode
    pub season_number: Option<u32>,

    /// Episode number in the season
    pub episode_number: Option<u32>,

    /// Apple episode type (e.g., "full", "trailer", "bonus")
    pub apple_episode_type: Option<String>,

    /// URL to the transcript
    pub transcripts_url: Option<String>,
}

/// Response for a list of episodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeListResponse {
    /// Total number of episodes
    pub count: u32,

    /// Pagination offset
    pub offset: Option<u32>,

    /// Pagination limit
    pub limit: Option<u32>,

    /// Whether there are more episodes
    pub has_more: Option<bool>,

    /// List of episodes
    pub episodes: Vec<Episode>,
}

/// Represents a podcast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Podcast {
    /// Unique identifier for the podcast
    #[serde(alias = "podcast_id")]
    pub id: String,

    /// Podcast title
    pub title: String,

    /// Podcast description
    #[serde(alias = "description")]
    pub desc: Option<String>,

    /// URL to the podcast logo/artwork
    pub logo: String,

    /// URL to the podcast page
    #[serde(alias = "url")]
    pub website: String,

    /// Primary category name
    #[serde(alias = "category")]
    pub category_name: Option<String>,

    /// Allowed episode types (e.g., "public", "premium", "private")
    pub allow_episode_type: Option<Vec<String>>,

    /// Copyright information
    pub copyright: Option<String>,
}

/// Response for a list of podcasts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastListResponse {
    /// Total number of podcasts
    pub count: u32,

    /// Pagination offset
    pub offset: Option<u32>,

    /// Pagination limit
    pub limit: Option<u32>,

    /// Whether there are more podcasts
    pub has_more: Option<bool>,

    /// List of podcasts
    pub podcasts: Vec<Podcast>,
}

/// Represents a private podcast member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateMember {
    /// Email address of the private member
    pub email: String,

    /// Name of the private member
    pub name: String,

    /// Status of the private member (e.g., "active", "inactive")
    pub status: String,

    /// Creation timestamp (Unix timestamp)
    pub created_at: u64,
}

/// Response for a list of private members.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateMemberListResponse {
    /// List of private members
    pub private_members: Vec<PrivateMember>,
}

/// Download URL(s) for a report, which can be a single URL or a list of URLs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ReportUrl {
    /// A single report URL
    Single(String),
    /// Multiple report URLs
    Multiple(Vec<String>),
}

/// Map of months to download URLs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportUrlMap {
    /// Download URLs grouped by month (or other keys)
    pub download_urls: HashMap<String, ReportUrl>,
}

/// Represents a episode type.
#[derive(Debug)]
pub enum EpisodeType {
    /// Public episode
    Public,
    /// Premium episode
    Premium,
    /// Private episode
    Private,
}

impl fmt::Display for EpisodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EpisodeType::Public => write!(f, "public"),
            EpisodeType::Premium => write!(f, "premium"),
            EpisodeType::Private => write!(f, "private"),
        }
    }
}

/// Episode status
#[derive(Debug)]
pub enum EpisodeStatus {
    /// Publish episode
    Publish,
    /// Draft episode
    Draft,
    /// Future episode
    Future,
}

impl fmt::Display for EpisodeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EpisodeStatus::Publish => write!(f, "publish"),
            EpisodeStatus::Draft => write!(f, "draft"),
            EpisodeStatus::Future => write!(f, "future"),
        }
    }
}

/// Audio format for media files.
/// - https://help.podbean.com/support/solutions/articles/25000005097-podbean-supported-file-formats-and-single-file-size-limit
#[derive(Debug)]
pub enum MediaFormat {
    /// MP3 audio format
    Mp3,
    /// M4A audio format
    M4a,
    /// OGG audio format
    Ogg,
}

impl fmt::Display for MediaFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaFormat::Mp3 => write!(f, "audio/mp3"),
            MediaFormat::M4a => write!(f, "audio/m4a"),
            MediaFormat::Ogg => write!(f, "audio/ogg"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_podcast_deserialization() {
        let json_data = r#"{
            "id": "podcast_123",
            "title": "My Podcast",
            "desc": "A great podcast",
            "logo": "https://example.com/logo.png",
            "website": "https://example.com",
            "category_name": "Technology",
            "allow_episode_type": ["public", "premium"],
            "copyright": "Copyright 2026"
        }"#;

        let podcast: Podcast = serde_json::from_str(json_data).unwrap();
        assert_eq!(podcast.id, "podcast_123");
        assert_eq!(podcast.title, "My Podcast");
        assert_eq!(podcast.desc.as_deref(), Some("A great podcast"));
        assert_eq!(podcast.logo, "https://example.com/logo.png");
        assert_eq!(podcast.website, "https://example.com");
        assert_eq!(podcast.category_name.as_deref(), Some("Technology"));
        assert_eq!(
            podcast.allow_episode_type.as_ref().unwrap(),
            &vec!["public".to_string(), "premium".to_string()]
        );
        assert_eq!(podcast.copyright.as_deref(), Some("Copyright 2026"));
    }

    #[test]
    fn test_podcast_deserialization_with_aliases() {
        let json_data = r#"{
            "podcast_id": "podcast_123",
            "title": "My Podcast",
            "description": "A great podcast",
            "logo": "https://example.com/logo.png",
            "url": "https://example.com",
            "category": "Technology",
            "allow_episode_type": ["public", "premium"],
            "copyright": "Copyright 2026"
        }"#;

        let podcast: Podcast = serde_json::from_str(json_data).unwrap();
        assert_eq!(podcast.id, "podcast_123");
        assert_eq!(podcast.title, "My Podcast");
        assert_eq!(podcast.desc.as_deref(), Some("A great podcast"));
        assert_eq!(podcast.logo, "https://example.com/logo.png");
        assert_eq!(podcast.website, "https://example.com");
        assert_eq!(podcast.category_name.as_deref(), Some("Technology"));
        assert_eq!(
            podcast.allow_episode_type.as_ref().unwrap(),
            &vec!["public".to_string(), "premium".to_string()]
        );
        assert_eq!(podcast.copyright.as_deref(), Some("Copyright 2026"));
    }

    #[test]
    fn test_episode_deserialization() {
        let json_data = r#"{
            "id": "episode_123",
            "podcast_id": "podcast_123",
            "title": "Episode 1",
            "content": "Episode description",
            "logo": "https://example.com/logo2.png",
            "media_url": "https://example.com/episode.mp3",
            "player_url": "https://example.com/player",
            "permalink_url": "https://example.com/permalink",
            "publish_time": 1700000000,
            "duration": 3600,
            "status": "published",
            "type": "public",
            "season_number": 1,
            "episode_number": 2,
            "apple_episode_type": "full",
            "transcripts_url": "https://example.com/transcripts"
        }"#;

        let episode: Episode = serde_json::from_str(json_data).unwrap();
        assert_eq!(episode.id, "episode_123");
        assert_eq!(episode.podcast_id, "podcast_123");
        assert_eq!(episode.title, "Episode 1");
        assert_eq!(episode.content, "Episode description");
        assert_eq!(
            episode.logo.as_deref(),
            Some("https://example.com/logo2.png")
        );
        assert_eq!(episode.media_url, "https://example.com/episode.mp3");
        assert_eq!(episode.player_url, "https://example.com/player");
        assert_eq!(episode.permalink_url, "https://example.com/permalink");
        assert_eq!(episode.publish_time, 1700000000);
        assert_eq!(episode.duration, Some(3600));
        assert_eq!(episode.status, "published");
        assert_eq!(episode.episode_type, "public");
        assert_eq!(episode.season_number, Some(1));
        assert_eq!(episode.episode_number, Some(2));
        assert_eq!(episode.apple_episode_type.as_deref(), Some("full"));
        assert_eq!(
            episode.transcripts_url.as_deref(),
            Some("https://example.com/transcripts")
        );
    }

    #[test]
    fn test_private_member_deserialization() {
        let json_data = r#"{
            "email": "user@example.com",
            "name": "John Doe",
            "status": "active",
            "created_at": 1700000000
        }"#;

        let member: PrivateMember = serde_json::from_str(json_data).unwrap();
        assert_eq!(member.email, "user@example.com");
        assert_eq!(member.name, "John Doe");
        assert_eq!(member.status, "active");
        assert_eq!(member.created_at, 1700000000);
    }

    #[test]
    fn test_report_url_map_deserialization() {
        let json_data = r#"{
            "download_urls": {
                "2026-01": "https://example.com/report1.csv",
                "2026-02": ["https://example.com/report2a.csv", "https://example.com/report2b.csv"]
            }
        }"#;

        let map: ReportUrlMap = serde_json::from_str(json_data).unwrap();
        let jan = map.download_urls.get("2026-01").unwrap();
        let feb = map.download_urls.get("2026-02").unwrap();

        assert_eq!(
            jan,
            &ReportUrl::Single("https://example.com/report1.csv".to_string())
        );
        assert_eq!(
            feb,
            &ReportUrl::Multiple(vec![
                "https://example.com/report2a.csv".to_string(),
                "https://example.com/report2b.csv".to_string()
            ])
        );
    }
}
