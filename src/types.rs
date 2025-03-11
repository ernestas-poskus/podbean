use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AuthToken {
    access_token: String,
    token_type: String,
    expires_in: u64,
    scope: Option<String>,
    refresh_token: Option<String>,
    created_at: u64,
}

impl AuthToken {
    pub(crate) fn is_expired(&self) -> bool {
        let now = Instant::now();

        let created_at: Instant = now
            .checked_sub(Duration::from_secs(self.created_at))
            .unwrap_or(now);

        let elapsed = now.duration_since(created_at);

        // Consider token expired if less than 5 minutes remaining
        elapsed.as_secs() + 300 > self.expires_in
    }

    pub(crate) fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }

    pub(crate) fn token_type(&self) -> &str {
        &self.token_type
    }

    pub(crate) fn access_token(&self) -> &str {
        &self.access_token
    }
}

impl From<TokenResponse> for AuthToken {
    fn from(token: TokenResponse) -> Self {
        AuthToken {
            access_token: token.access_token,
            token_type: token.token_type,
            expires_in: token.expires_in,
            scope: token.scope,
            refresh_token: token.refresh_token,
            created_at: Instant::now().elapsed().as_secs(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    scope: Option<String>,
    refresh_token: Option<String>,
}

// Media-related structs
#[derive(Debug, Serialize, Deserialize)]
pub struct MediaItem {
    pub media_key: String,
    pub title: String,
    pub content: String,
    pub status: String,
    pub media_url: String,
    pub logo_url: Option<String>,
    pub player_url: Option<String>,
    pub publish_time: Option<String>,
    pub created_at: String,
    pub duration: Option<u64>,
    // Add more fields as needed based on the API response
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaListResponse {
    pub count: u32,
    pub media: Vec<MediaItem>,
}

// Episode-related structs
#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub title: String,
    pub content: String,
    pub status: String,
    pub post_url: String,
    pub player_url: String,
    pub publish_time: String,
    pub created_at: String,
    pub duration: u64,
    pub download_url: String,
    // Add more fields as needed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpisodeListResponse {
    pub count: u32,
    pub episodes: Vec<Episode>,
}

// Podcast-related structs
#[derive(Debug, Serialize, Deserialize)]
pub struct Podcast {
    pub podcast_id: String,
    pub title: String,
    pub description: String,
    pub logo: String,
    pub url: String,
    pub category: String,
    pub subcategory: Option<String>,
    // Add more fields as needed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PodcastListResponse {
    pub count: u32,
    pub podcasts: Vec<Podcast>,
}
