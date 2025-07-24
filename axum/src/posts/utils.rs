use reqwest::{Client, StatusCode};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

// Define a custom error type for embedding API interactions
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingApiError {
    #[error("HTTP error: {0}")]
    HttpRequest(#[from] reqwest::Error),
    #[error("API returned non-success status: {0}")]
    ApiError(StatusCode),
    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
    #[error("Embedding API is unhealthy")]
    Unhealthy,
}

/// Calls the embedding API with retry logic.
pub async fn call_embedding_api_with_retry<T>(
    embedding_api_url: &str,
    request_payload: T,
) -> Result<super::models::EmbeddingResponse, EmbeddingApiError>
where
    T: serde::Serialize,
{
    let client = Client::new();
    let max_retries = 2;
    let api_key = std::env::var("EMBED_API_KEY").unwrap_or_default();
    let mut current_retry = 0;

    while current_retry < max_retries {
        info!(
            "Attempting to call embedding API (retry {}/{})",
            current_retry + 1,
            max_retries
        );
        match client
            .post(embedding_api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request_payload)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    return Ok(response
                        .json::<super::models::EmbeddingResponse>()
                        .await
                        .map_err(EmbeddingApiError::HttpRequest)?);
                } else {
                    warn!(
                        "Embedding API returned non-success status: {}. Retrying...",
                        status
                    );
                    // Log the error body if available
                    if let Ok(body) = response.text().await {
                        warn!("Error response body: {}", body);
                    }
                    current_retry += 1;
                    sleep(Duration::from_secs(2u64.pow(current_retry))).await; // Exponential backoff
                }
            }
            Err(e) => {
                error!("Failed to call embedding API: {}. Retrying...", e);
                current_retry += 1;
                sleep(Duration::from_secs(2u64.pow(current_retry))).await; // Exponential backoff
            }
        }
    }
    error!("Max retries exceeded for embedding API call.");
    Err(EmbeddingApiError::MaxRetriesExceeded)
}

/// Checks the health of the embedding API.
pub async fn check_embedding_api_health(health_check_url: &str) -> Result<(), EmbeddingApiError> {
    let client = Client::new();
    let max_retries = 6;
    let api_key = std::env::var("EMBED_API_KEY").unwrap_or_default();
    let mut current_retry = 0;

    while current_retry < max_retries {
        info!(
            "Attempting health check (retry {}/{}) at: {}",
            current_retry + 1,
            max_retries,
            health_check_url
        );
        match client
            .get(health_check_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Embedding API is healthy.");
                    return Ok(());
                } else {
                    warn!(
                        "Embedding API health check failed with status: {}. Retrying...",
                        response.status()
                    );
                    current_retry += 1;
                    sleep(Duration::from_secs(2u64.pow(current_retry))).await; // Exponential backoff
                }
            }
            Err(e) => {
                error!(
                    "Failed to reach embedding API for health check: {}. Retrying...",
                    e
                );
                current_retry += 1;
                sleep(Duration::from_secs(2u64.pow(current_retry))).await; // Exponential backoff
            }
        }
    }
    error!("Max retries exceeded for health check.");
    Err(EmbeddingApiError::Unhealthy)
}

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::fmt::Display,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
