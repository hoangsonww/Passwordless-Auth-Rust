use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info};

/// Webhook event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    UserRegistered,
    UserAuthenticated,
    SessionCreated,
    SessionRevoked,
    TotpEnrolled,
    WebauthnRegistered,
}

/// Webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: WebhookEventType,
    pub user_id: String,
    pub email: Option<String>,
    pub timestamp: String,
    pub metadata: Option<serde_json::Value>,
}

/// Webhook sender configuration
#[derive(Clone)]
pub struct WebhookSender {
    client: Client,
    webhook_url: Option<String>,
    webhook_secret: Option<String>,
}

impl WebhookSender {
    pub fn new(webhook_url: Option<String>, webhook_secret: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        Self {
            client,
            webhook_url,
            webhook_secret,
        }
    }

    /// Send a webhook event (async, fire-and-forget)
    pub async fn send(&self, payload: WebhookPayload) {
        if let Some(url) = &self.webhook_url {
            info!("Sending webhook for event: {:?}", payload.event);

            let mut request = self.client.post(url).json(&payload);

            // Add secret as header if configured
            if let Some(secret) = &self.webhook_secret {
                request = request.header("X-Webhook-Secret", secret);
            }

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        info!("Webhook sent successfully: {:?}", payload.event);
                    } else {
                        error!(
                            "Webhook failed with status {}: {:?}",
                            response.status(),
                            payload.event
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to send webhook: {}", e);
                }
            }
        }
    }

    /// Send webhook in background (spawn task)
    pub fn send_background(&self, payload: WebhookPayload) {
        if self.webhook_url.is_some() {
            let sender = self.clone();
            tokio::spawn(async move {
                sender.send(payload).await;
            });
        }
    }
}
