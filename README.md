# Podbean API Client

A fully async Rust client for the [Podbean API](https://developers.podbean.com/podbean-api-docs/), built with Tokio and Reqwest.

## Features

- **Fully async**: Built on Tokio runtime and Reqwest for efficient HTTP requests
- **OAuth2 support**: Handles authentication, token refresh, and authorization flows
- **Rate limiting**: Built-in rate limiting to avoid hitting API limits
- **Comprehensive API coverage**: Supports podcasts, episodes, media files, and more
- **Proper error handling**: Custom error types with detailed information
- **Type-safe**: Strongly typed API responses with Serde

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
podbean = "0.1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quick Start

```rust
use podbean::PodbeanClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new client with your credentials
    let mut client = PodbeanClient::new("your_client_id", "your_client_secret");

    // Generate authorization URL for the user to visit
    let auth_url = client.get_authorization_url(
        "https://your-app.com/callback",
        Some("state_for_csrf_protection")
    )?;
    println!("Please visit: {}", auth_url);

    // After user authorization, exchange the code for a token
    client.authorize("authorization_code_from_callback", "https://your-app.com/callback").await?;

    // Now you can use the API
    let podcasts = client.list_podcasts(None, Some(10)).await?;
    println!("You have {} podcasts", podcasts.count);

    Ok(())
}
```

## Examples

### Uploading and Publishing a Podcast Episode

```rust
// Upload an audio file
let media_key = client.upload_media("/path/to/episode.mp3", "audio/mpeg").await?;

// Publish a new episode
let episode_id = client.publish_episode(
    "your_podcast_id",
    "Episode Title",
    "Episode description and show notes...",
    &media_key,
    "publish", // Can be "publish", "draft", or "schedule"
    None, // Publish immediately
).await?;

println!("Published new episode with ID: {}", episode_id);
```

### Managing Episodes

```rust
// List episodes
let episodes = client.list_episodes(Some("your_podcast_id"), None, Some(20)).await?;
println!("Found {} episodes", episodes.count);

// Get a specific episode
let episode = client.get_episode("episode_id").await?;
println!("Episode: {} (URL: {})", episode.title, episode.player_url);

// Update an episode
client.update_episode(
    "episode_id",
    Some("Updated Title"),
    Some("Updated description"),
    None, // Keep current status
    None, // Keep current publish time
).await?;

// Delete an episode
client.delete_episode("episode_id").await?;
```

## API Reference

### Authentication

- `PodbeanClient::new(client_id, client_secret)` - Create a new client
- `client.get_authorization_url(redirect_uri, state)` - Generate OAuth authorization URL
- `client.authorize(code, redirect_uri)` - Exchange authorization code for token
- `client.refresh_token()` - Refresh the access token when expired

### Podcasts

- `client.list_podcasts(offset, limit)` - List podcasts for the authenticated user

### Episodes

- `client.list_episodes(podcast_id, offset, limit)` - List episodes
- `client.get_episode(episode_id)` - Get a specific episode
- `client.publish_episode(podcast_id, title, content, media_key, status, publish_timestamp)` - Publish a new episode
- `client.update_episode(episode_id, title, content, status, publish_timestamp)` - Update an episode
- `client.delete_episode(episode_id)` - Delete an episode

### Media Files

- `client.upload_media(file_path, content_type)` - Upload a media file
- `client.list_media(offset, limit)` - List media files

## Error Handling

The library uses a custom `PodbeanError` type that provides detailed information about what went wrong:

```rust
match client.list_episodes(Some("podcast_id"), None, None).await {
    Ok(episodes) => {
        // Handle successful response
    },
    Err(err) => match err {
        PodbeanError::ApiError { code, message } => {
            eprintln!("API returned error {}: {}", code, message);
        },
        PodbeanError::RateLimitError { retry_after } => {
            if let Some(seconds) = retry_after {
                eprintln!("Rate limited, retry after {} seconds", seconds);
            }
        },
        PodbeanError::AuthError(msg) => {
            eprintln!("Authentication error: {}", msg);
        },
        // Handle other error types
        _ => eprintln!("Error: {}", err),
    },
}
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
