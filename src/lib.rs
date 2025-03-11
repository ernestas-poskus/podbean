use reqwest::{Client, Response, StatusCode};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use types::AuthToken;
use url::Url;

mod error;
pub use error::PodbeanError;

mod types;
pub use types::{
    Episode, EpisodeListResponse, MediaItem, MediaListResponse, PodcastListResponse, TokenResponse,
};

pub type PodbeanResult<T> = Result<T, PodbeanError>;

#[derive(Debug, Clone)]
pub struct PodbeanClient {
    client: Client,
    client_id: String,
    client_secret: String,
    base_url: String,
    token: Option<AuthToken>,
    rate_limit: RateLimiter,
}

#[derive(Debug, Clone)]
struct RateLimiter {
    requests_per_minute: u32,
    request_times: Vec<Instant>,
}

impl RateLimiter {
    fn new(requests_per_minute: u32) -> Self {
        RateLimiter {
            requests_per_minute,
            request_times: Vec::with_capacity(requests_per_minute as usize),
        }
    }

    async fn wait_if_needed(&mut self) {
        let now = Instant::now();

        // Remove request times older than 1 minute
        self.request_times
            .retain(|&time| now.duration_since(time) < Duration::from_secs(60));

        // If we've reached the limit, wait until we can make another request
        if self.request_times.len() >= self.requests_per_minute as usize {
            let oldest = self.request_times[0];
            let wait_time = Duration::from_secs(60) - now.duration_since(oldest);

            if wait_time.as_secs() > 0 {
                sleep(wait_time).await;
            }

            // Remove the oldest request time
            self.request_times.remove(0);
        }

        // Add the current time to the list
        self.request_times.push(Instant::now());
    }
}

impl PodbeanClient {
    pub fn new(client_id: &str, client_secret: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            base_url: "https://api.podbean.com/v1".to_string(),
            token: None,
            rate_limit: RateLimiter::new(60), // Default rate limit of 60 requests per minute
        }
    }

    pub async fn authorize(&mut self, code: &str, redirect_uri: &str) -> PodbeanResult<()> {
        let params = [
            ("grant_type", "authorization_code"),
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

    async fn handle_token_response(&mut self, response: Response) -> PodbeanResult<()> {
        if response.status().is_success() {
            let token_response: TokenResponse = response.json().await?;

            self.token = Some(AuthToken::from(token_response));

            Ok(())
        } else {
            Err(self.handle_error_response(response).await)
        }
    }

    async fn ensure_token(&mut self) -> PodbeanResult<()> {
        if let Some(token) = &self.token {
            if token.is_expired() {
                self.refresh_token().await?;
            }
            Ok(())
        } else {
            Err(PodbeanError::AuthError("Not authenticated".to_string()))
        }
    }

    async fn make_request<T>(
        &mut self,
        method: reqwest::Method,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
    ) -> PodbeanResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.ensure_token().await?;
        self.rate_limit.wait_if_needed().await;

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

    // File operations
    pub async fn upload_media(
        &mut self,
        file_path: &str,
        content_type: &str,
    ) -> PodbeanResult<String> {
        self.ensure_token().await?;

        // First, get the presigned URL for upload
        let mut params = HashMap::new();
        params.insert(
            "filename".to_string(),
            std::path::Path::new(file_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown.mp3")
                .to_string(),
        );
        params.insert("content_type".to_string(), content_type.to_string());

        let presigned: serde_json::Value = self
            .make_request(reqwest::Method::GET, "/files/uploadAuthorize", Some(params))
            .await?;

        let presigned_url = presigned["presigned_url"].as_str().ok_or_else(|| {
            PodbeanError::OtherError("Missing presigned_url in response".to_string())
        })?;

        let file_key = presigned["file_key"]
            .as_str()
            .ok_or_else(|| PodbeanError::OtherError("Missing file_key in response".to_string()))?;

        // Now upload the file to the presigned URL
        let file_content = tokio::fs::read(file_path)
            .await
            .map_err(|e| PodbeanError::OtherError(format!("Failed to read file: {}", e)))?;

        let upload_response = self
            .client
            .put(presigned_url)
            .header("Content-Type", content_type)
            .body(file_content)
            .send()
            .await?;

        if !upload_response.status().is_success() {
            return Err(self.handle_error_response(upload_response).await);
        }

        Ok(file_key.to_string())
    }

    // Episode operations
    pub async fn publish_episode(
        &mut self,
        podcast_id: &str,
        title: &str,
        content: &str,
        media_key: &str,
        status: &str,
        publish_timestamp: Option<i64>,
    ) -> PodbeanResult<String> {
        let mut params = HashMap::new();
        params.insert("podcast_id".to_string(), podcast_id.to_string());
        params.insert("title".to_string(), title.to_string());
        params.insert("content".to_string(), content.to_string());
        params.insert("media_key".to_string(), media_key.to_string());
        params.insert("status".to_string(), status.to_string()); // publish, draft, schedule

        if let Some(timestamp) = publish_timestamp {
            params.insert("publish_timestamp".to_string(), timestamp.to_string());
        }

        let response: serde_json::Value = self
            .make_request(reqwest::Method::POST, "/episodes", Some(params))
            .await?;

        response["episode"]["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| PodbeanError::OtherError("Missing episode ID in response".to_string()))
    }

    pub async fn get_episode(&mut self, episode_id: &str) -> PodbeanResult<Episode> {
        let mut params = HashMap::new();
        params.insert("id".to_string(), episode_id.to_string());

        self.make_request(reqwest::Method::GET, "/episodes/one", Some(params))
            .await
    }

    pub async fn list_episodes(
        &mut self,
        podcast_id: Option<&str>,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> PodbeanResult<EpisodeListResponse> {
        let mut params = HashMap::new();

        if let Some(id) = podcast_id {
            params.insert("podcast_id".to_string(), id.to_string());
        }

        if let Some(offset_val) = offset {
            params.insert("offset".to_string(), offset_val.to_string());
        }

        if let Some(limit_val) = limit {
            params.insert("limit".to_string(), limit_val.to_string());
        }

        self.make_request(reqwest::Method::GET, "/episodes", Some(params))
            .await
    }

    pub async fn update_episode(
        &mut self,
        episode_id: &str,
        title: Option<&str>,
        content: Option<&str>,
        status: Option<&str>,
        publish_timestamp: Option<i64>,
    ) -> PodbeanResult<()> {
        let mut params = HashMap::new();
        params.insert("id".to_string(), episode_id.to_string());

        if let Some(title_val) = title {
            params.insert("title".to_string(), title_val.to_string());
        }

        if let Some(content_val) = content {
            params.insert("content".to_string(), content_val.to_string());
        }

        if let Some(status_val) = status {
            params.insert("status".to_string(), status_val.to_string());
        }

        if let Some(timestamp) = publish_timestamp {
            params.insert("publish_timestamp".to_string(), timestamp.to_string());
        }

        let _: serde_json::Value = self
            .make_request(reqwest::Method::PUT, "/episodes", Some(params))
            .await?;

        Ok(())
    }

    pub async fn delete_episode(&mut self, episode_id: &str) -> PodbeanResult<()> {
        let mut params = HashMap::new();
        params.insert("id".to_string(), episode_id.to_string());

        let _: serde_json::Value = self
            .make_request(reqwest::Method::DELETE, "/episodes", Some(params))
            .await?;

        Ok(())
    }

    // Podcast operations
    pub async fn list_podcasts(
        &mut self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> PodbeanResult<PodcastListResponse> {
        let mut params = HashMap::new();

        if let Some(offset_val) = offset {
            params.insert("offset".to_string(), offset_val.to_string());
        }

        if let Some(limit_val) = limit {
            params.insert("limit".to_string(), limit_val.to_string());
        }

        self.make_request(reqwest::Method::GET, "/podcasts", Some(params))
            .await
    }

    // Media operations
    pub async fn list_media(
        &mut self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> PodbeanResult<MediaListResponse> {
        let mut params = HashMap::new();

        if let Some(offset_val) = offset {
            params.insert("offset".to_string(), offset_val.to_string());
        }

        if let Some(limit_val) = limit {
            params.insert("limit".to_string(), limit_val.to_string());
        }

        self.make_request(reqwest::Method::GET, "/medias", Some(params))
            .await
    }

    // Generate authorization URL
    pub fn get_authorization_url(
        &self,
        redirect_uri: &str,
        state: Option<&str>,
    ) -> PodbeanResult<String> {
        let mut url = Url::parse("https://api.podbean.com/v1/dialog/oauth")?;

        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", redirect_uri);

        if let Some(state_val) = state {
            url.query_pairs_mut().append_pair("state", state_val);
        }

        Ok(url.to_string())
    }
}
