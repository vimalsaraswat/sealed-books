//! Hedera Consensus Service publisher client for Sealed Books Server.

use chrono::Utc;
use hedera::{Client, PrivateKey, TopicId, TopicMessageSubmitTransaction};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::mirror::MockMessageStore;

/// Receipt returned after a successful Hedera topic message submission.
#[derive(Debug, Clone)]
pub struct PublishReceipt {
    pub topic_id: String,
    pub sequence_number: u64,
    pub consensus_timestamp: String,
    pub transaction_id: String,
}

/// Live Hedera testnet publisher using the official Hedera SDK client.
#[derive(Clone)]
pub struct HederaLivePublisher {
    client: Arc<Client>,
}

impl HederaLivePublisher {
    pub fn new_from_env() -> Result<Self, String> {
        let account_id_str = std::env::var("HEDERA_OPERATOR_ID")
            .map_err(|_| "HEDERA_OPERATOR_ID not set in environment".to_string())?;
        let private_key_str = std::env::var("HEDERA_OPERATOR_KEY")
            .map_err(|_| "HEDERA_OPERATOR_KEY not set in environment".to_string())?;

        let client = Client::for_testnet();
        let operator_id = hedera::AccountId::from_str(&account_id_str)
            .map_err(|e| format!("Invalid HEDERA_OPERATOR_ID: {e}"))?;
        let operator_key = PrivateKey::from_str(&private_key_str)
            .map_err(|e| format!("Invalid HEDERA_OPERATOR_KEY: {e}"))?;

        client.set_operator(operator_id, operator_key);

        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub async fn publish(
        &self,
        topic_id_str: &str,
        message: &[u8],
    ) -> Result<PublishReceipt, String> {
        let topic_id = TopicId::from_str(topic_id_str)
            .map_err(|e| format!("Invalid topic ID {topic_id_str}: {e}"))?;

        let response = TopicMessageSubmitTransaction::new()
            .topic_id(topic_id)
            .message(message.to_vec())
            .execute(&*self.client)
            .await
            .map_err(|e| format!("Hedera HCS submit transaction failed: {e}"))?;

        let receipt = response
            .get_receipt(&*self.client)
            .await
            .map_err(|e| format!("Hedera transaction receipt failed: {e}"))?;

        let seq_num = receipt.topic_sequence_number;
        let tx_id = response.transaction_id.to_string();
        let timestamp = Utc::now().to_rfc3339();

        Ok(PublishReceipt {
            topic_id: topic_id_str.to_string(),
            sequence_number: seq_num,
            consensus_timestamp: timestamp,
            transaction_id: tx_id,
        })
    }
}

#[derive(Clone)]
pub struct MockPublisher {
    next_seq: Arc<AtomicU64>,
    store: Option<MockMessageStore>,
}

impl MockPublisher {
    pub fn new() -> Self {
        Self {
            next_seq: Arc::new(AtomicU64::new(1)),
            store: None,
        }
    }

    pub fn with_store(store: MockMessageStore) -> Self {
        Self {
            next_seq: Arc::new(AtomicU64::new(1)),
            store: Some(store),
        }
    }

    pub async fn publish(&self, topic_id: &str, message: &[u8]) -> Result<PublishReceipt, String> {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        let now = Utc::now();
        let ts_sec = now.timestamp();
        let ts_subsec = now.timestamp_subsec_nanos();
        let consensus_ts = format!("{ts_sec}.{ts_subsec:09}");

        if let Some(store) = &self.store {
            store.insert(topic_id, seq, &consensus_ts, message);
        }

        Ok(PublishReceipt {
            topic_id: topic_id.to_string(),
            sequence_number: seq,
            consensus_timestamp: consensus_ts,
            transaction_id: format!("0.0.10120177@{ts_sec}.{ts_subsec}"),
        })
    }
}

#[derive(Clone)]
pub enum PublisherClient {
    Live(HederaLivePublisher),
    Mock(MockPublisher),
}

impl PublisherClient {
    pub fn mock() -> Self {
        PublisherClient::Mock(MockPublisher::new())
    }

    pub fn mock_with_store(store: MockMessageStore) -> Self {
        PublisherClient::Mock(MockPublisher::with_store(store))
    }

    pub fn live_from_env() -> Result<Self, String> {
        Ok(PublisherClient::Live(HederaLivePublisher::new_from_env()?))
    }

    pub async fn publish(&self, topic_id: &str, message: &[u8]) -> Result<PublishReceipt, String> {
        match self {
            Self::Live(live) => live.publish(topic_id, message).await,
            Self::Mock(mock) => mock.publish(topic_id, message).await,
        }
    }
}
