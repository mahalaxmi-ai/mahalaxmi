//! Anthropic Messages API client for adversarial deliberation turns.

use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::MahalaxmiResult;

/// A single message in a multi-turn chat conversation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    /// The role of the sender: `"user"` or `"assistant"`.
    pub role: String,
    /// The text content of the message.
    pub content: String,
}

/// HTTP client for the Anthropic Messages API.
///
/// Wraps `reqwest` to send system-prompted conversations and return
/// the first assistant text block from the response.
///
/// `Clone` is cheap — `reqwest::Client` is backed by an `Arc` so cloning
/// creates a second handle sharing the same connection pool.
#[derive(Clone)]
pub struct AnthropicApiClient {
    /// The model identifier to use for each request (e.g. `"claude-haiku-4-5-20251001"`).
    pub model: String,
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicApiClient {
    /// Create a new client with the given model and API key.
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            client: reqwest::Client::new(),
            api_key: api_key.into(),
        }
    }

    /// Send a chat completion request and return the assistant's reply text.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the HTTP request fails, the server returns a non-2xx
    /// status, or the response body cannot be decoded.
    pub async fn chat(&self, system: &str, messages: Vec<ChatMessage>) -> MahalaxmiResult<String> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": system,
            "messages": messages,
        });

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| MahalaxmiError::Provider {
                message: format!("Anthropic API request failed: {e}"),
                i18n_key: "error.deliberation.api_request_failed".to_owned(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(MahalaxmiError::Provider {
                message: format!("Anthropic API returned {status}: {text}"),
                i18n_key: "error.deliberation.api_error_response".to_owned(),
            });
        }

        let json: serde_json::Value =
            response
                .json()
                .await
                .map_err(|e| MahalaxmiError::Provider {
                    message: format!("Failed to parse Anthropic API response: {e}"),
                    i18n_key: "error.deliberation.api_parse_failed".to_owned(),
                })?;

        let content =
            json["content"][0]["text"]
                .as_str()
                .ok_or_else(|| MahalaxmiError::Provider {
                    message: "Anthropic API response missing content[0].text".to_owned(),
                    i18n_key: "error.deliberation.api_missing_content".to_owned(),
                })?;

        Ok(content.to_owned())
    }
}
