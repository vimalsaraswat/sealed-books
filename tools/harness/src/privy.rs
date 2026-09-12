//! Privy Server Wallet & Signing Probe for Phase 0.
//!
//! Validates:
//! 1. Privy server authentication with App ID & Secret.
//! 2. Ethereum server wallet creation.
//! 3. Inspection of `public_key` field (Open Question 1).
//! 4. Signing 32-byte raw hash via `secp256k1_sign`.
//! 5. Splitting 65-byte signature into 64-byte `(r || s)` and recovery byte `v`.
//! 6. Local signature verification and Ethereum address derivation via `crypto.rs`.

use crate::crypto;
use base64::Engine;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use sha2::{Digest as Sha2Digest, Sha256};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct PrivyCreateWalletRequest {
    chain_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    policy_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PrivyWalletResponse {
    id: String,
    address: String,
    chain_type: String,
    #[serde(default)]
    public_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct PrivyRpcSignRequest {
    method: String,
    params: PrivyRpcSignParams,
}

#[derive(Debug, Serialize)]
struct PrivyRpcSignParams {
    hash: String,
}

#[derive(Debug, Deserialize)]
struct PrivyRpcResponse {
    data: Option<PrivyRpcData>,
    signature: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PrivyRpcData {
    signature: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PrivyPolicyRequest {
    version: String,
    name: String,
    chain_type: String,
    rules: Vec<PrivyPolicyRule>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PrivyPolicyRule {
    name: String,
    method: String,
    action: String,
    conditions: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct PrivyPolicyResponse {
    id: String,
    name: String,
}

fn build_privy_client() -> Result<(reqwest::Client, String), Box<dyn std::error::Error>> {
    let app_id =
        env::var("PRIVY_APP_ID").map_err(|_| "PRIVY_APP_ID not found in environment or .env")?;
    let app_secret = env::var("PRIVY_APP_SECRET")
        .map_err(|_| "PRIVY_APP_SECRET not found in environment or .env")?;

    let mut headers = HeaderMap::new();
    let auth_val = format!("{}:{}", app_id.trim(), app_secret.trim());
    let auth_b64 = base64::engine::general_purpose::STANDARD.encode(auth_val);
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Basic {auth_b64}"))?,
    );
    headers.insert("privy-app-id", HeaderValue::from_str(app_id.trim())?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    Ok((client, app_id))
}

pub async fn run_privy_probe() -> Result<(), Box<dyn std::error::Error>> {
    let (http_client, app_id) = build_privy_client()?;
    println!("• Privy App ID: {app_id}");

    // 1. Create a server wallet
    println!("• Creating a server wallet via POST https://api.privy.io/v1/wallets...");
    let create_wallet_req = PrivyCreateWalletRequest {
        chain_type: "ethereum".to_string(),
        policy_ids: vec![],
    };

    let resp = http_client
        .post("https://api.privy.io/v1/wallets")
        .json(&create_wallet_req)
        .send()
        .await?;

    let status = resp.status();
    let raw_text = resp.text().await?;

    if !status.is_success() {
        return Err(
            format!("Failed to create Privy wallet: HTTP {status} — Response: {raw_text}").into(),
        );
    }

    let wallet: PrivyWalletResponse = serde_json::from_str(&raw_text)?;
    println!("  -> Wallet ID:      {}", wallet.id);
    println!("  -> Wallet Address: {}", wallet.address);
    println!("  -> Chain Type:     {}", wallet.chain_type);
    println!(
        "  -> Public Key:     {}",
        wallet
            .public_key
            .as_deref()
            .unwrap_or("<null/not populated>")
    );

    println!("\n[Open Question 1 Check: Is `public_key` populated?]");
    if let Some(ref pk) = wallet.public_key {
        println!("  ANSWER: YES! Privy populated `public_key` directly: {pk}");
    } else {
        println!("  ANSWER: NO (`public_key` is null/omitted). Using signature recovery via `v`.");
    }

    // 2. Sign a 32-byte hash with secp256k1_sign
    let message_to_sign = b"Sealed Books Phase 0 Test Statement";
    let hash_bytes = Sha256::digest(message_to_sign);
    let hash_hex = format!("0x{}", hex::encode(hash_bytes));
    println!("\n• Signing 32-byte hash via secp256k1_sign...");
    println!("  Hash: {hash_hex}");

    let sign_req = PrivyRpcSignRequest {
        method: "secp256k1_sign".to_string(),
        params: PrivyRpcSignParams {
            hash: hash_hex.clone(),
        },
    };

    let rpc_url = format!("https://api.privy.io/v1/wallets/{}/rpc", wallet.id);
    let sign_resp = http_client.post(&rpc_url).json(&sign_req).send().await?;

    let sign_status = sign_resp.status();
    let sign_raw = sign_resp.text().await?;

    if !sign_status.is_success() {
        return Err(format!(
            "Privy secp256k1_sign failed: HTTP {sign_status} — Response: {sign_raw}"
        )
        .into());
    }

    let rpc_res: PrivyRpcResponse = serde_json::from_str(&sign_raw)?;
    let sig_hex_raw = rpc_res
        .data
        .and_then(|d| d.signature)
        .or(rpc_res.signature)
        .ok_or_else(|| format!("Signature field not found in response: {sign_raw}"))?;

    let sig_hex = sig_hex_raw.trim_start_matches("0x");
    let sig_bytes = hex::decode(sig_hex)?;

    println!(
        "  -> Received Signature ({} bytes): 0x{sig_hex}",
        sig_bytes.len()
    );

    let (rs_bytes, v_byte) = crypto::split_signature(&sig_bytes)?;
    println!(
        "  -> r || s (64 bytes for Hedera payload): 0x{}",
        hex::encode(rs_bytes)
    );
    println!("  -> v (recovery ID): {v_byte} (0x{v_byte:02x})");

    // 3. Local Verification with k256 & address derivation
    println!("\n• Verifying signature locally using k256...");
    let recovered = crypto::verify_and_recover(&hash_bytes, rs_bytes, v_byte)?;
    println!(
        "  -> Recovered compressed secp256k1 pubkey (33 bytes): 0x{}",
        recovered.compressed_pubkey_hex
    );
    println!(
        "  -> Derived Ethereum address from recovered pubkey: {}",
        recovered.derived_address
    );
    println!(
        "  -> Original Privy wallet address:                  {}",
        wallet.address
    );

    if recovered
        .derived_address
        .eq_ignore_ascii_case(&wallet.address)
    {
        println!("  -> MATCH! Derived address matches Privy wallet address!");
    } else {
        return Err(format!(
            "Address mismatch! Derived {} != Wallet {}",
            recovered.derived_address, wallet.address
        )
        .into());
    }

    if let Some(ref pk_hex_raw) = wallet.public_key {
        let clean_pk = pk_hex_raw.trim_start_matches("0x");
        let pk_bytes = hex::decode(clean_pk)?;
        crypto::verify_with_pubkey(&hash_bytes, rs_bytes, &pk_bytes)?;
        println!("  -> SUCCESS: Signature verified directly against Privy's `public_key` field!");
    } else {
        println!("  -> SUCCESS: Signature verified against recovered public key!");
    }

    // 4. Open Question 3 Check: Query Policies API
    println!("\n[Open Question 3 Check: Querying Privy Policies endpoint...]");
    let policies_resp = http_client
        .get("https://api.privy.io/v1/policies")
        .send()
        .await;

    match policies_resp {
        Ok(r) => {
            let p_status = r.status();
            let p_text = r.text().await.unwrap_or_default();
            println!("  Policies endpoint HTTP status: {p_status}");
            println!(
                "  Policies response preview: {}",
                &p_text.chars().take(200).collect::<String>()
            );
        }
        Err(e) => {
            println!("  Failed to query /v1/policies: {e}");
        }
    }

    Ok(())
}

/// Provisions 3 approver Privy wallets with an attached policy and extracts their public keys.
pub async fn run_provision() -> Result<(), Box<dyn std::error::Error>> {
    let (http_client, _) = build_privy_client()?;

    println!("• Creating or verifying Privy Policy for Period Approvers...");
    let policy_req = PrivyPolicyRequest {
        version: "1.0".to_string(),
        name: "Sealed Books Period Approver Policy".to_string(),
        chain_type: "ethereum".to_string(),
        rules: vec![PrivyPolicyRule {
            name: "Allow all ledger close signing operations".to_string(),
            method: "*".to_string(),
            action: "ALLOW".to_string(),
            conditions: vec![],
        }],
    };

    let policy_resp = http_client
        .post("https://api.privy.io/v1/policies")
        .json(&policy_req)
        .send()
        .await?;

    let policy_raw = policy_resp.text().await?;
    let policy: PrivyPolicyResponse = serde_json::from_str(&policy_raw)
        .map_err(|e| format!("Failed to parse policy response: {e} — Raw: {policy_raw}"))?;

    println!("  -> Policy ID:   {}", policy.id);
    println!("  -> Policy Name: {}", policy.name);

    let roles = [
        ("CFO", "Alice Chen", "APPROVER_1"),
        ("Financial Controller", "Bob Smith", "APPROVER_2"),
        ("Auditor", "Carol Vance", "APPROVER_3"),
    ];

    let test_hash = [0x42u8; 32];
    let test_hash_hex = format!("0x{}", hex::encode(test_hash));

    println!("\n• Provisioning 3 Approver Wallets with Policy Attached...");

    for (role, name, env_prefix) in roles {
        let wallet_req = PrivyCreateWalletRequest {
            chain_type: "ethereum".to_string(),
            policy_ids: vec![policy.id.clone()],
        };

        let w_resp = http_client
            .post("https://api.privy.io/v1/wallets")
            .json(&wallet_req)
            .send()
            .await?;

        let w_raw = w_resp.text().await?;
        let wallet: PrivyWalletResponse = serde_json::from_str(&w_raw).map_err(|e| {
            format!("Failed to parse wallet response for {role}: {e} — Raw: {w_raw}")
        })?;

        // Sign test hash to recover compressed public key
        let sign_req = PrivyRpcSignRequest {
            method: "secp256k1_sign".to_string(),
            params: PrivyRpcSignParams {
                hash: test_hash_hex.clone(),
            },
        };

        let rpc_url = format!("https://api.privy.io/v1/wallets/{}/rpc", wallet.id);
        let s_resp = http_client.post(&rpc_url).json(&sign_req).send().await?;
        let s_raw = s_resp.text().await?;
        let rpc_res: PrivyRpcResponse = serde_json::from_str(&s_raw)
            .map_err(|e| format!("Failed to parse sign response: {e} — Raw: {s_raw}"))?;

        let sig_hex_raw = rpc_res
            .data
            .and_then(|d| d.signature)
            .or(rpc_res.signature)
            .ok_or_else(|| format!("No signature in response: {s_raw}"))?;

        let sig_bytes = hex::decode(sig_hex_raw.trim_start_matches("0x"))?;
        let (rs_bytes, v_byte) = crypto::split_signature(&sig_bytes)?;
        let recovered = crypto::verify_and_recover(&test_hash, rs_bytes, v_byte)?;

        println!("\n  [{role} - {name}]");
        println!("    Wallet ID:   {}", wallet.id);
        println!("    Address:     {}", wallet.address);
        println!("    Pubkey (33B): 0x{}", recovered.compressed_pubkey_hex);
        println!("    Env Keys:");
        println!("      {env_prefix}_NAME=\"{name}\"");
        println!("      {env_prefix}_ROLE=\"{role}\"");
        println!("      {env_prefix}_WALLET_ID=\"{}\"", wallet.id);
        println!("      {env_prefix}_ADDRESS=\"{}\"", wallet.address);
        println!(
            "      {env_prefix}_PUBKEY=\"0x{}\"",
            recovered.compressed_pubkey_hex
        );
    }

    Ok(())
}
