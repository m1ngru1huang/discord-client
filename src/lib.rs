//! Discord API

use secrecy::{ExposeSecret, Secret};
use serde_json::json;
use thiserror::Error;
use tracing::instrument;

const DISCORD_SERVER_URL: &str = "https://discord.com/api";

#[derive(Error, Debug)]
pub enum DiscordClientError {
    #[error("Invalid Webhook Message Builder: {0}")]
    InvalidWebhookMessageBuilder(String),
    #[error("Error when sending request to Discord server")]
    FailedToSendRequest(#[from] reqwest::Error),
}

/// Webhooks related APIs.
pub struct Webhooks {
    client: reqwest::Client,
    webhook_id: String,
    webhook_token: Secret<String>,
}

/// Webhook message.
pub struct WebhookMessage {
    message: String,
}

/// Webhook message builder.
pub struct WebhookMessageBuilder {
    message: Option<String>,
}

impl WebhookMessage {
    pub fn builder() -> WebhookMessageBuilder {
        WebhookMessageBuilder { message: None }
    }
}

impl WebhookMessageBuilder {
    pub fn build(self) -> Result<WebhookMessage, DiscordClientError> {
        if self.message.is_none() {
            return Err(DiscordClientError::InvalidWebhookMessageBuilder(
                "`message` field required".into(),
            ));
        }

        Ok(WebhookMessage {
            message: self.message.unwrap_or_default(),
        })
    }
}

impl Webhooks {
    /// Create an instance of
    pub fn new(webhook_id: impl Into<String>, webhook_token: Secret<String>) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            webhook_id: webhook_id.into(),
            webhook_token,
        }
    }
}

impl Webhooks {
    /// Execute(Send) webhook message.
    #[instrument(name = "discord.webhooks.execute_webhook", skip(self, message))]
    pub async fn execute_webhook(&self, message: WebhookMessage) -> Result<(), DiscordClientError> {
        let payload = json!({
            "content": message.message,
        });
        self.client
            .post(&format!(
                "{DISCORD_SERVER_URL}/webhooks/{webhook_id}/{webhook_token}",
                webhook_id = self.webhook_id,
                webhook_token = self.webhook_token.expose_secret()
            ))
            .json(&payload)
            .send()
            .await?;
        Ok(())
    }
}
