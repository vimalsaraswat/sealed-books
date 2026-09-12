//! Hedera Consensus Service client and publisher for Sealed Books Server.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishReceipt {
    pub topic_id: String,
    pub sequence_number: u64,
    pub consensus_timestamp: String,
    pub transaction_id: String,
}

#[derive(Clone)]
pub struct HederaLivePublisher {
    client: Arc<hedera::Client>,
}

impl HederaLivePublisher {
    pub fn new_from_env() -> Result<Self, String> {
        let operator_id_str = std::env::var("HEDERA_OPERATOR_ACCOUNT_ID")
            .map_err(|_| "HEDERA_OPERATOR_ACCOUNT_ID not found in environment".to_string())?;
        let operator_key_str = std::env::var("HEDERA_OPERATOR_KEY")
            .map_err(|_| "HEDERA_OPERATOR_KEY not found in environment".to_string())?;

        let operator_id = hedera::AccountId::from_str(&operator_id_str)
            .map_err(|e| format!("Invalid HEDERA_OPERATOR_ACCOUNT_ID: {e}"))?;
        let operator_key = hedera::PrivateKey::from_str(&operator_key_str)
            .map_err(|e| format!("Invalid HEDERA_OPERATOR_KEY: {e}"))?;

        let client = hedera::Client::for_testnet();
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
        let topic_id = hedera::TopicId::from_str(topic_id_str)
            .map_err(|e| format!("Invalid topic ID {topic_id_str}: {e}"))?;

        let mut tx = hedera::TopicMessageSubmitTransaction::new();
        tx.topic_id(topic_id);
        tx.message(message.to_vec());

        let response = tx
            .execute(&*self.client)
            .await
            .map_err(|e| format!("Hedera transaction execution failed: {e}"))?;

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
}

impl MockPublisher {
    pub fn new() -> Self {
        Self {
            next_seq: Arc::new(AtomicU64::new(1)),
        }
    }

    pub async fn publish(&self, topic_id: &str, _message: &[u8]) -> Result<PublishReceipt, String> {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        let now = Utc::now();
        let ts_sec = now.timestamp();
        let ts_subsec = now.timestamp_subsec_nanos();
        let consensus_ts = format!("{ts_sec}.{ts_subsec:09}");

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

    pub fn live_from_env() -> Result<Self, String> {
        Ok(PublisherClient::Live(HederaLivePublisher::new_from_env()?))
    }

    pub async fn publish(&self, topic_id: &str, message: &[u8]) -> Result<PublishReceipt, String> {
        match self {
            PublisherClient::Live(p) => p.publish(topic_id, message).await,
            PublisherClient::Mock(p) => p.publish(topic_id, message).await,
        }
    }
}
