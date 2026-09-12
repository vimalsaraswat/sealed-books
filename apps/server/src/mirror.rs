//! Plain HTTP Hedera Mirror Node client.
//!
//! Queries the public testnet mirror node REST API for topic messages
//! without requiring the Hedera SDK, gRPC, or network keys.

use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const DEFAULT_TESTNET_MIRROR_URL: &str = "https://testnet.mirrornode.hedera.com";

/// Structure representing a message response from the Hedera mirror node REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorTopicMessage {
    pub consensus_timestamp: String,
    pub message: String, // Base64-encoded payload
    pub sequence_number: u64,
    pub topic_id: String,
    pub running_hash: Option<String>,
}

pub type MessageStoreMap = Arc<Mutex<HashMap<(String, u64), (String, Vec<u8>)>>>;

/// Shared in-memory message store for deterministic offline tests.
#[derive(Clone, Default)]
pub struct MockMessageStore {
    pub messages: MessageStoreMap,
}

impl MockMessageStore {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn insert(&self, topic_id: &str, sequence_number: u64, timestamp: &str, payload: &[u8]) {
        if let Ok(mut lock) = self.messages.lock() {
            lock.insert(
                (topic_id.to_string(), sequence_number),
                (timestamp.to_string(), payload.to_vec()),
            );
        }
    }

    pub fn get(&self, topic_id: &str, sequence_number: u64) -> Option<(String, Vec<u8>)> {
        let lock = self.messages.lock().ok()?;
        lock.get(&(topic_id.to_string(), sequence_number)).cloned()
    }
}

/// Mirror client abstraction supporting live REST queries and mock test fixtures.
#[derive(Clone)]
pub enum MirrorClient {
    Live { client: Client, base_url: String },
    Mock { store: MockMessageStore },
}

impl MirrorClient {
    pub fn live_from_env() -> Self {
        let base_url = std::env::var("HEDERA_MIRROR_URL")
            .unwrap_or_else(|_| DEFAULT_TESTNET_MIRROR_URL.to_string());
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self::Live { client, base_url }
    }

    pub fn mock(store: MockMessageStore) -> Self {
        Self::Mock { store }
    }

    /// Fetches and base64-decodes the message payload for a given topic ID and sequence number.
    /// Returns `(consensus_timestamp, payload_bytes)`.
    pub async fn fetch_message(
        &self,
        topic_id: &str,
        sequence_number: u64,
    ) -> Result<(String, Vec<u8>), String> {
        match self {
            Self::Mock { store } => {
                let (ts, bytes) = store.get(topic_id, sequence_number).ok_or_else(|| {
                    format!(
                        "Mock mirror node: message not found for topic {topic_id} seq #{sequence_number}"
                    )
                })?;
                Ok((ts, bytes))
            }
            Self::Live { client, base_url } => {
                let url = format!("{base_url}/api/v1/topics/{topic_id}/messages/{sequence_number}");
                let resp = client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| format!("Mirror node HTTP request failed: {e}"))?;

                let status = resp.status();
                if !status.is_success() {
                    let err_body = resp.text().await.unwrap_or_default();
                    return Err(format!(
                        "Mirror node returned status {status} for topic {topic_id} seq #{sequence_number}: {err_body}"
                    ));
                }

                let msg_record: MirrorTopicMessage = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse mirror node JSON: {e}"))?;

                let payload_bytes = base64::engine::general_purpose::STANDARD
                    .decode(msg_record.message.trim())
                    .map_err(|e| format!("Failed to base64-decode mirror node payload: {e}"))?;

                Ok((msg_record.consensus_timestamp, payload_bytes))
            }
        }
    }
}
