//! Hedera Consensus Service (HCS) Probe for Phase 0.
//!
//! Validates:
//! 1. Client setup with testnet operator credentials.
//! 2. Topic creation (`TopicCreateTransaction`).
//! 3. Message submission (`TopicMessageSubmitTransaction`).
//! 4. Mirror node polling and base64 message decoding.

use base64::Engine;
use serde::Deserialize;
use std::env;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct MirrorMessagesResponse {
    messages: Vec<MirrorMessage>,
}

#[derive(Debug, Deserialize)]
struct MirrorMessage {
    consensus_timestamp: String,
    message: String,
    running_hash: String,
    sequence_number: u64,
}

pub async fn run_hedera_probe() -> Result<(), Box<dyn std::error::Error>> {
    let operator_id_str = env::var("HEDERA_OPERATOR_ACCOUNT_ID")
        .map_err(|_| "HEDERA_OPERATOR_ACCOUNT_ID not found in environment or .env")?;
    let operator_key_str = env::var("HEDERA_OPERATOR_KEY")
        .map_err(|_| "HEDERA_OPERATOR_KEY not found in environment or .env")?;

    println!("• Operator Account: {operator_id_str}");
    let operator_id = hedera::AccountId::from_str(&operator_id_str)?;
    let operator_key = hedera::PrivateKey::from_str(&operator_key_str)?;

    let client = hedera::Client::for_testnet();
    client.set_operator(operator_id, operator_key);

    println!("• Creating HCS Topic...");
    let mut tx_create = hedera::TopicCreateTransaction::new();
    tx_create.topic_memo("Sealed Books Phase 0 Spike");

    let resp = tx_create.execute(&client).await?;
    let receipt = resp.get_receipt(&client).await?;
    let topic_id = receipt.topic_id.ok_or("Receipt did not contain topic ID")?;

    println!("  -> Topic created: {topic_id}");
    println!("  -> HashScan URL:  https://hashscan.io/testnet/topic/{topic_id}");

    let test_payload = format!(
        "SealedBooks::Phase0::Probe::{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis()
    );
    println!("• Submitting message: \"{test_payload}\"...");

    let mut tx_msg = hedera::TopicMessageSubmitTransaction::new();
    tx_msg.topic_id(topic_id);
    tx_msg.message(test_payload.as_bytes().to_vec());

    let msg_resp = tx_msg.execute(&client).await?;
    let msg_receipt = msg_resp.get_receipt(&client).await?;

    println!(
        "  -> Message submitted! Sequence number: {}, Status: {:?}",
        msg_receipt.topic_sequence_number, msg_receipt.status
    );

    println!("• Polling Hedera Testnet Mirror Node REST API for topic messages...");
    let mirror_url =
        format!("https://testnet.mirrornode.hedera.com/api/v1/topics/{topic_id}/messages");
    println!("  -> Endpoint: {mirror_url}");

    let http_client = reqwest::Client::new();
    let mut found_message: Option<MirrorMessage> = None;

    for attempt in 1..=15 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        print!("  Attempt {attempt}/15... ");
        let resp = http_client.get(&mirror_url).send().await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let body: MirrorMessagesResponse = r.json().await?;
                if let Some(msg) = body.messages.into_iter().next() {
                    println!("found!");
                    found_message = Some(msg);
                    break;
                } else {
                    println!("no messages yet.");
                }
            }
            Ok(r) => {
                println!("HTTP {}", r.status());
            }
            Err(e) => {
                println!("Error: {e}");
            }
        }
    }

    let msg = found_message.ok_or("Timeout waiting for message to appear in mirror node")?;
    println!("• Retrieved message from Mirror Node:");
    println!("  - Consensus Timestamp: {}", msg.consensus_timestamp);
    println!("  - Sequence Number:     {}", msg.sequence_number);
    println!("  - Running Hash:        {}", msg.running_hash);
    println!("  - Raw Base64 Payload:  {}", msg.message);

    let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(&msg.message)?;
    let decoded_str = String::from_utf8(decoded_bytes)?;
    println!("  - Decoded String:      \"{decoded_str}\"");

    if decoded_str == test_payload {
        println!("  -> Verification SUCCESS: Decoded payload matches original message exactly.");
    } else {
        return Err(
            format!("Payload mismatch! Expected '{test_payload}', got '{decoded_str}'").into(),
        );
    }

    Ok(())
}
