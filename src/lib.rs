//! # Podbean an async Podbean client for connecting to the Podbean API.
#![doc = include_str!("../README.md")]
#![deny(
    warnings,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    unreachable_pub,
    deprecated,
    unknown_lints,
    unreachable_code,
    unused_mut
)]

use reqwest::{Client, Response, StatusCode};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use types::AuthToken;
use url::Url;

mod error;
pub use error::PodbeanError;

mod types;
pub use types::{
    Episode, EpisodeListResponse, EpisodeStatus, EpisodeType, MediaFormat, MediaItem,
    MediaListResponse, PodcastListResponse, TokenResponse,
};

/// Result type for Podbean API operations.
pub type PodbeanResult<T> = Result<T, PodbeanError>;

/// A client for interacting with the Podbean API.
///
/// This client handles authentication, token management, rate limiting,
/// and provides methods to interact with Podbean's API endpoints.
#[derive(Debug, Clone)]
pub struct PodbeanClient {
    client: Client,
    client_id: String,
    client_secret: String,
    base_url: String,
    token: Option<AuthToken>,
}

impl PodbeanClient {
    /// Creates a new Podbean API client.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client ID from your Podbean API application
    /// * `client_secret` - The client secret from your Podbean API application
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use podbean::PodbeanClient;
    ///
    /// let client = PodbeanClient::new("your_client_id", "your_client_secret");
    /// ```
    pub fn new(client_id: &str, client_secret: &str) -> Result<Self, PodbeanError> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self {
            client,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            base_url: "https://api.podbean.com/v1".to_string(),
            token: None,
        })
    }

    /// Authorize the client using an authorization code.
    ///
    /// This method exchanges an authorization code for an access token
    /// after the user has authorized your application.
    ///
    /// # Arguments
    ///
    /// * `code` - The authorization code received after user authorization
    /// * `redirect_uri` - The redirect URI used in the authorization request
    ///
    /// # Returns
    ///
    /// * `Ok(())` if authorization was successful
    /// * `Err(PodbeanError)` if there was an error during authorization
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use podbean::PodbeanClient;
    /// # use tokio::runtime::Runtime;
    /// # let mut client = PodbeanClient::new("id", "secret").unwrap();
    /// # let rt = Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// let code = "authorization_code"; // From callback URL
    /// let redirect_uri = "https://your-app.com/callback";
    ///
    /// match client.authorize(code, redirect_uri).await {
    ///     Ok(_) => println!("Authorization successful!"),
    ///     Err(e) => eprintln!("Authorization failed: {}", e),
    /// }
    /// });
    /// ```
    pub async fn authorize(&mut self, code: &str, redirect_uri: &str) -> PodbeanResult<()> {
        let params = [
            ("grant_type", "client_credentials"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response = self
            .client
            .post("https://api.podbean.com/v1/oauth/token")
            .form(&params)
            .send()
            .await?;

        self.handle_token_response(response).await
    }

    /// Refresh the access token.
    ///
    /// This method uses the refresh token to obtain a new access token
    /// when the current one expires.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if token refresh was successful
    /// * `Err(PodbeanError)` if there was an error during token refresh
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use podbean::PodbeanClient;
    /// # use tokio::runtime::Runtime;
    /// # let mut client = PodbeanClient::new("id", "secret").unwrap();
    /// # let rt = Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// // Typically called automatically by the client when needed
    /// match client.refresh_token().await {
    ///     Ok(_) => println!("Token refreshed successfully"),
    ///     Err(e) => eprintln!("Token refresh failed: {}", e),
    /// }
    /// # });
    /// ```
    pub async fn refresh_token(&mut self) -> PodbeanResult<()> {
        if let Some(token) = &self.token {
            if let Some(refresh_token) = token.refresh_token() {
                let params = [
                    ("grant_type", "refresh_token"),
                    ("refresh_token", refresh_token),
                    ("client_id", &self.client_id),
                    ("client_secret", &self.client_secret),
                ];

                let response = self
                    .client
                    .post("https://api.podbean.com/v1/oauth/token")
                    .form(&params)
                    .send()
                    .await?;

                return self.handle_token_response(response).await;
            }
        }

        Err(PodbeanError::AuthError(
            "No refresh token available".to_string(),
        ))
    }

    /// Handles the token response from authorization or refresh requests.
    async fn handle_token_response(&mut self, response: Response) -> PodbeanResult<()> {
        if response.status().is_success() {
            let token_response: TokenResponse = response.json().await?;

            self.token = Some(AuthToken::from(token_response));

            Ok(())
        } else {
            Err(self.handle_error_response(response).await)
        }
    }

    /// Ensures a valid token is available, refreshing if necessary.
    async fn ensure_token(&self) -> PodbeanResult<()> {
        if let Some(token) = &self.token {
            if token.is_expired() {
                return Err(PodbeanError::AuthError(
                    "Refresh authentication token".to_string(),
                ));
            }
            Ok(())
        } else {
            Err(PodbeanError::AuthError("Not authenticated".to_string()))
        }
    }

    /// Makes a request to the Podbean API.
    ///
    /// This internal method handles token management, rate limiting,
    /// and error handling for all API requests.
    async fn make_request<T>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
    ) -> PodbeanResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.ensure_token().await?;

        let url = format!("{}{}", self.base_url, endpoint);
        let token = self.token.as_ref().unwrap();

        let mut request_builder = self.client.request(method.clone(), &url).header(
            "Authorization",
            format!("{} {}", token.token_type(), token.access_token()),
        );

        if let Some(params) = params {
            request_builder = if method == reqwest::Method::GET {
                request_builder.query(&params)
            } else {
                request_builder.form(&params)
            };
        }

        let response = request_builder.send().await?;

        if response.status().is_success() {
            let result: T = response.json().await?;
            Ok(result)
        } else {
            Err(self.handle_error_response(response).await)
        }
    }

    /// Processes error responses from the API.
    async fn handle_error_response(&self, response: Response) -> PodbeanError {
        let status = response.status();

        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            return PodbeanError::RateLimitError { retry_after };
        }

        match response.text().await {
            Ok(text) => {
                if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let (Some(error), Some(message)) = (
                        error_json.get("error").and_then(|v| v.as_str()),
                        error_json.get("error_description").and_then(|v| v.as_str()),
                    ) {
                        return PodbeanError::ApiError {
                            code: status.as_u16(),
                            message: format!("{}: {}", error, message),
                        };
                    }
                }

                PodbeanError::ApiError {
                    code: status.as_u16(),
                    message: text,
                }
            }
            Err(e) => PodbeanError::OtherError(format!("Failed to read error response: {}", e)),
        }
    }

    /// Uploads a media file to Podbean.
    ///
    /// This method uploads a media file (typically an audio file) to Podbean
    /// and returns a media key that can be used to publish episodes.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the local file to upload
    /// * `content_type` - MIME type of the file (e.g., "audio/mp3" for MP3)
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the media key if successful
    /// * `Err(PodbeanError)` if there was an error during upload
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use podbean::{PodbeanClient, MediaFormat};
    /// # use tokio::runtime::Runtime;
    /// # let mut client = PodbeanClient::new("id", "secret").unwrap();
    /// # let rt = Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// # client.authorize("code", "redirect").await.unwrap();
    /// let media_key = client.upload_media("episode.mp3".to_string(), vec![], MediaFormat::Mp3).await.unwrap();
    /// println!("Media uploaded with key: {}", media_key);
    /// # });
    /// ```
    pub async fn upload_media(
        &self,
        file_name: String,
        file_content: Vec<u8>,
        media_format: MediaFormat,
    ) -> PodbeanResult<String> {
        self.ensure_token().await?;

        // First, get the presigned URL for upload
        let mut params = HashMap::new();

        let _ = params.insert("filename".to_string(), file_name);
        let _ = params.insert("content_type".to_string(), media_format.to_string());
        let _ = params.insert("filesize".to_string(), file_content.len().to_string());

        let presigned: serde_json::Value = self
            .make_request(reqwest::Method::GET, "/files/uploadAuthorize", Some(params))
            .await?;

        let presigned_url = presigned["presigned_url"].as_str().ok_or_else(|| {
            PodbeanError::OtherError("Missing presigned_url in response".to_string())
        })?;

        let file_key = presigned["file_key"]
            .as_str()
            .ok_or_else(|| PodbeanError::OtherError("Missing file_key in response".to_string()))?;

        let upload_response = self
            .client
            .put(presigned_url)
            .header("Content-Type", media_format.to_string())
            .body(file_content)
            .send()
            .await?;

        if !upload_response.status().is_success() {
            return Err(self.handle_error_response(upload_response).await);
        }

        Ok(file_key.to_string())
    }

    /// Publishes a new episode to a podcast.
    ///
    /// # Arguments
    ///
    /// * `podcast_id` - The ID of the podcast to publish to
    /// * `title` - The title of the episode
    /// * `content` - The description or show notes for the episode
    /// * `media_key` - The media key returned from `upload_media`
    /// * `status` - Publication status: "publish", "draft", or "future"
    /// * `type` - Publication status: "public", "premium", "private"
    /// * `publish_timestamp` - The publishing timestamp of an episode. The episode will be listed based on its publishing time, from New to Old by default. If it is not set, the "current time" will be set as its publishing time.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the episode ID if successful
    /// * `Err(PodbeanError)` if there was an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use podbean::{PodbeanClient, EpisodeStatus, EpisodeType};
    /// # use tokio::runtime::Runtime;
    /// # let mut client = PodbeanClient::new("id", "secret").unwrap();
    /// # let rt = Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// # client.authorize("code", "redirect").await.unwrap();
    /// # let media_key = "media_key";
    /// let episode_id = client.publish_episode(
    ///     "podcast_id",
    ///     "My New Episode",
    ///     "Episode description and show notes...",
    ///     &media_key,
    ///     EpisodeStatus::Draft,
    ///     EpisodeType::Public,
    ///     None,
    /// ).await.unwrap();
    ///
    /// println!("Episode published with ID: {}", episode_id);
    /// # });
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub async fn publish_episode(
        &self,
        podcast_id: &str,
        title: &str,
        content: &str,
        media_key: &str,
        status: EpisodeStatus,
        episode_type: EpisodeType,
        publish_timestamp: Option<i64>,
    ) -> PodbeanResult<String> {
        let mut params = HashMap::new();
        let _ = params.insert("podcast_id".to_string(), podcast_id.to_string());
        let _ = params.insert("title".to_string(), title.to_string());
        let _ = params.insert("content".to_string(), content.to_string());
        let _ = params.insert("media_key".to_string(), media_key.to_string());
        let _ = params.insert("status".to_string(), status.to_string()); // publish, draft, future
        let _ = params.insert("type".to_string(), episode_type.to_string()); // "public", "premium", "private"

        if let Some(timestamp) = publish_timestamp {
            let _ = params.insert("publish_timestamp".to_string(), timestamp.to_string());
        }

        let response: serde_json::Value = self
            .make_request(reqwest::Method::POST, "/episodes", Some(params))
            .await?;

        response["episode"]["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| PodbeanError::OtherError("Missing episode ID in response".to_string()))
    }

    /// Gets information about a specific episode.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - The ID of the episode to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Episode)` containing the episode details if successful
    /// * `Err(PodbeanError)` if there was an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use podbean::PodbeanClient;
    /// # use tokio::runtime::Runtime;
    /// # let mut client = PodbeanClient::new("id", "secret").unwrap();
    /// # let rt = Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// # client.authorize("code", "redirect").await.unwrap();
    /// let episode = client.get_episode("episode_id").await.unwrap();
    /// println!("Episode title: {}", episode.title);
    /// println!("Listen URL: {}", episode.player_url);
    /// # });
    /// ```
    pub async fn get_episode(&self, episode_id: &str) -> PodbeanResult<Episode> {
        let mut params = HashMap::new();
        let _ = params.insert("id".to_string(), episode_id.to_string());

        self.make_request(reqwest::Method::GET, "/episodes/one", Some(params))
            .await
    }

    /// Lists episodes from a podcast.
    ///
    /// # Arguments
    ///
    /// * `podcast_id` - Optional podcast ID to filter episodes
    /// * `offset` - Optional pagination offset
    /// * `limit` - Optional number of episodes to return
    ///
    /// # Returns
    ///
    /// * `Ok(EpisodeListResponse)` containing the episodes if successful
    /// * `Err(PodbeanError)` if there was an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use podbean::PodbeanClient;
    /// # use tokio::runtime::Runtime;
    /// # let mut client = PodbeanClient::new("id", "secret").unwrap();
    /// # let rt = Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// # client.authorize("code", "redirect").await.unwrap();
    /// // Get the first 10 episodes from a specific podcast
    /// let episodes = client.list_episodes(
    ///     Some("podcast_id"),
    ///     None,  // Start from beginning
    ///     Some(10) // Get 10 episodes
    /// ).await.unwrap();
    ///
    /// println!("Found {} episodes", episodes.count);
    /// for episode in episodes.episodes {
    ///     println!("- {} ({})", episode.title, episode.publish_time);
    /// }
    /// # });
    /// ```
    pub async fn list_episodes(
        &self,
        podcast_id: Option<&str>,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> PodbeanResult<EpisodeListResponse> {
        let mut params = HashMap::new();

        if let Some(id) = podcast_id {
            let _ = params.insert("podcast_id".to_string(), id.to_string());
        }

        if let Some(offset_val) = offset {
            let _ = params.insert("offset".to_string(), offset_val.to_string());
        }

        if let Some(limit_val) = limit {
            let _ = params.insert("limit".to_string(), limit_val.to_string());
        }

        self.make_request(reqwest::Method::GET, "/episodes", Some(params))
            .await
    }

    /// Updates an existing episode.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - The ID of the episode to update
    /// * `title` - Optional new title
    /// * `content` - Optional new content/description
    /// * `status` - Optional new status
    /// * `publish_timestamp` - Optional new publication timestamp
    ///
    /// # Returns
    ///
    /// * `Ok(())` if update was successful
    /// * `Err(PodbeanError)` if there was an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use podbean::PodbeanClient;
    /// # use tokio::runtime::Runtime;
    /// # let mut client = PodbeanClient::new("id", "secret").unwrap();
    /// # let rt = Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// # client.authorize("code", "redirect").await.unwrap();
    /// // Update just the title of an episode
    /// client.update_episode(
    ///     "episode_id",
    ///     Some("Updated Title"),
    ///     None,  // Keep current content
    ///     None,  // Keep current status
    ///     None   // Keep current publish time
    /// ).await.unwrap();
    /// println!("Episode updated successfully");
    /// # });
    /// ```
    pub async fn update_episode(
        &self,
        episode_id: &str,
        title: Option<&str>,
        content: Option<&str>,
        status: Option<&str>,
        publish_timestamp: Option<i64>,
    ) -> PodbeanResult<()> {
        let mut params = HashMap::new();
        let _ = params.insert("id".to_string(), episode_id.to_string());

        if let Some(title_val) = title {
            let _ = params.insert("title".to_string(), title_val.to_string());
        }

        if let Some(content_val) = content {
            let _ = params.insert("content".to_string(), content_val.to_string());
        }

        if let Some(status_val) = status {
            let _ = params.insert("status".to_string(), status_val.to_string());
        }

        if let Some(timestamp) = publish_timestamp {
            let _ = params.insert("publish_timestamp".to_string(), timestamp.to_string());
        }

        let _: serde_json::Value = self
            .make_request(reqwest::Method::PUT, "/episodes", Some(params))
            .await?;

        Ok(())
    }

    /// Deletes an episode.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - The ID of the episode to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` if deletion was successful
    /// * `Err(PodbeanError)` if there was an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use podbean::PodbeanClient;
    /// # use tokio::runtime::Runtime;
    /// # let mut client = PodbeanClient::new("id", "secret").unwrap();
    /// # let rt = Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// # client.authorize("code", "redirect").await.unwrap();
    /// client.delete_episode("episode_id").await.unwrap();
    /// println!("Episode deleted successfully");
    /// # });
    /// ```
    pub async fn delete_episode(&self, episode_id: &str) -> PodbeanResult<()> {
        let mut params = HashMap::new();
        let _ = params.insert("id".to_string(), episode_id.to_string());

        let _: serde_json::Value = self
            .make_request(reqwest::Method::DELETE, "/episodes", Some(params))
            .await?;

        Ok(())
    }

    /// Lists podcasts for the authenticated user.
    ///
    /// # Arguments
    ///
    /// * `offset` - Optional pagination offset
    /// * `limit` - Optional number of podcasts to return
    ///
    /// # Returns
    ///
    /// * `Ok(PodcastListResponse)` containing the podcasts if successful
    /// * `Err(PodbeanError)` if there was an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use podbean::PodbeanClient;
    /// # use tokio::runtime::Runtime;
    /// # let mut client = PodbeanClient::new("id", "secret").unwrap();
    /// # let rt = Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// # client.authorize("code", "redirect").await.unwrap();
    /// let podcasts = client.list_podcasts(None, Some(10)).await.unwrap();
    /// println!("Found {} podcasts", podcasts.count);
    /// for podcast in podcasts.podcasts {
    ///     println!("- {} ({})", podcast.title, podcast.podcast_id);
    /// }
    /// # });
    /// ```
    pub async fn list_podcasts(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> PodbeanResult<PodcastListResponse> {
        let mut params = HashMap::new();

        if let Some(offset_val) = offset {
            let _ = params.insert("offset".to_string(), offset_val.to_string());
        }

        if let Some(limit_val) = limit {
            let _ = params.insert("limit".to_string(), limit_val.to_string());
        }

        self.make_request(reqwest::Method::GET, "/podcasts", Some(params))
            .await
    }

    /// Lists media files for the authenticated user.
    ///
    /// # Arguments
    ///
    /// * `offset` - Optional pagination offset
    /// * `limit` - Optional number of media files to return
    ///
    /// # Returns
    ///
    /// * `Ok(MediaListResponse)` containing the media files if successful
    /// * `Err(PodbeanError)` if there was an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use podbean::PodbeanClient;
    /// # use tokio::runtime::Runtime;
    /// # let mut client = PodbeanClient::new("id", "secret").unwrap();
    /// # let rt = Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// # client.authorize("code", "redirect").await.unwrap();
    /// let media = client.list_media(None, Some(10)).await.unwrap();
    /// println!("Found {} media files", media.count);
    /// for item in media.media {
    ///     println!("- {} ({})", item.title, item.media_key);
    /// }
    /// # });
    /// ```
    pub async fn list_media(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> PodbeanResult<MediaListResponse> {
        let mut params = HashMap::new();

        if let Some(offset_val) = offset {
            let _ = params.insert("offset".to_string(), offset_val.to_string());
        }

        if let Some(limit_val) = limit {
            let _ = params.insert("limit".to_string(), limit_val.to_string());
        }

        self.make_request(reqwest::Method::GET, "/medias", Some(params))
            .await
    }

    /// Generates an authorization URL for OAuth2 flow.
    ///
    /// Users need to visit this URL to authorize your application to
    /// access their Podbean account.
    ///
    /// # Arguments
    ///
    /// * `redirect_uri` - The URI to redirect to after authorization
    /// * `state` - Optional state parameter for CSRF protection
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the authorization URL if successful
    /// * `Err(PodbeanError)` if there was an error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use podbean::PodbeanClient;
    /// let client = PodbeanClient::new("client_id", "client_secret").unwrap();
    ///
    /// let auth_url = client.get_authorization_url(
    ///     "https://your-app.com/callback",
    ///     Some("random_state_for_csrf_protection")
    /// ).unwrap();
    ///
    /// println!("Visit this URL to authorize: {}", auth_url);
    /// ```
    pub fn get_authorization_url(
        &self,
        redirect_uri: &str,
        state: Option<&str>,
    ) -> PodbeanResult<String> {
        let mut url = Url::parse("https://api.podbean.com/v1/dialog/oauth")?;

        let _ = url
            .query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", redirect_uri);

        if let Some(state_val) = state {
            let _ = url.query_pairs_mut().append_pair("state", state_val);
        }

        Ok(url.to_string())
    }
}
