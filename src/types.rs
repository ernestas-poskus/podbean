//! Data types for the Podbean API client.
//!
//! This module defines the various data structures used to represent
//! Podbean API resources and responses.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Response from OAuth token endpoint.
#[derive(Debug, Serialize, Deserialize)]
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
pub struct AuthToken {
    access_token: String,
    token_type: String,
    expires_in: u64,
    scope: Option<String>,
    refresh_token: Option<String>,
    created_at: Instant,
}

impl AuthToken {
    /// Checks if the token is expired.
    ///
    /// Considers a token expired if it has less than 5 minutes of validity left.
    pub fn is_expired(&self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.created_at);

        // Consider token expired if less than 5 minutes remaining
        elapsed.as_secs() + 300 > self.expires_in
    }

    /// Gets the access token string.
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    /// Gets the token type.
    pub fn token_type(&self) -> &str {
        &self.token_type
    }

    /// Gets the refresh token, if any.
    pub fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }
}

impl From<TokenResponse> for AuthToken {
    fn from(response: TokenResponse) -> Self {
        Self {
            access_token: response.access_token,
            token_type: response.token_type,
            expires_in: response.expires_in,
            scope: response.scope,
            refresh_token: response.refresh_token,
            created_at: Instant::now(),
        }
    }
}

/// Represents a media item in Podbean.
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct MediaListResponse {
    /// Total number of media items
    pub count: u32,

    /// List of media items
    pub media: Vec<MediaItem>,
}

/// Represents a podcast episode.
#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    /// Unique identifier for the episode
    pub id: String,

    /// Episode title
    pub title: String,

    /// Episode description or show notes
    pub content: String,

    /// Publication status (e.g., "published", "draft")
    pub status: String,

    /// URL to the episode page
    pub post_url: String,

    /// URL to play the episode
    pub player_url: String,

    /// When the episode was published
    pub publish_time: String,

    /// When the episode was created
    pub created_at: String,

    /// Duration in seconds
    pub duration: u64,

    /// URL to download the episode audio
    pub download_url: String,
}

/// Response for a list of episodes.
#[derive(Debug, Serialize, Deserialize)]
pub struct EpisodeListResponse {
    /// Total number of episodes
    pub count: u32,

    /// List of episodes
    pub episodes: Vec<Episode>,
}

/// Represents a podcast.
#[derive(Debug, Serialize, Deserialize)]
pub struct Podcast {
    /// Unique identifier for the podcast
    pub podcast_id: String,

    /// Podcast title
    pub title: String,

    /// Podcast description
    pub description: String,

    /// URL to the podcast logo/artwork
    pub logo: String,

    /// URL to the podcast page
    pub url: String,

    /// Primary category
    pub category: String,

    /// Secondary category
    pub subcategory: Option<String>,
}

/// Response for a list of podcasts.
#[derive(Debug, Serialize, Deserialize)]
pub struct PodcastListResponse {
    /// Total number of podcasts
    pub count: u32,

    /// List of podcasts
    pub podcasts: Vec<Podcast>,
}
