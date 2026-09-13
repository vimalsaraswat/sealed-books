//! Privy Server Wallet client and signing service.

use crate::crypto::{self, RecoveredKeyInfo};
use base64::Engine;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

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
pub struct PrivyCreateWalletRequest {
    pub chain_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivyWalletResponse {
    pub id: String,
    pub address: String,
    pub chain_type: String,
}

#[derive(Clone)]
pub struct PrivyClient {
    http_client: reqwest::Client,
    _app_id: String,
}

impl PrivyClient {
    pub fn new_from_env() -> Result<Self, String> {
        let app_id = std::env::var("PRIVY_APP_ID")
            .map_err(|_| "PRIVY_APP_ID not found in environment".to_string())?;
        let app_secret = std::env::var("PRIVY_APP_SECRET")
            .map_err(|_| "PRIVY_APP_SECRET not found in environment".to_string())?;

        let mut headers = HeaderMap::new();
        let auth_val = format!("{}:{}", app_id.trim(), app_secret.trim());
        let auth_b64 = base64::engine::general_purpose::STANDARD.encode(auth_val);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {auth_b64}"))
                .map_err(|e| format!("Invalid auth header: {e}"))?,
        );
        headers.insert(
            "privy-app-id",
            HeaderValue::from_str(app_id.trim())
                .map_err(|e| format!("Invalid privy-app-id header: {e}"))?,
        );
                headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            reqwest::header::USER_AGENT,
            HeaderValue::from_static("SealedBooks-Server/0.1"),
        );

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| format!("Failed to build reqwest client: {e}"))?;

        Ok(Self {
            http_client,
            _app_id: app_id,
        })
    }

    /// Creates a new Ethereum server wallet in Privy, automatically attaching organizational policies if configured.
    pub async fn create_ethereum_wallet(&self) -> Result<PrivyWalletResponse, String> {
        let mut policy_ids = Vec::new();
        if let Ok(pid) = std::env::var("PRIVY_POLICY_ID") {
            let pid_trimmed = pid.trim();
            if !pid_trimmed.is_empty() {
                policy_ids.push(pid_trimmed.to_string());
            }
        }

        let req = PrivyCreateWalletRequest {
            chain_type: "ethereum".to_string(),
            policy_ids,
        };

        let resp = self
            .http_client
            .post("https://api.privy.io/v1/wallets")
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Privy create wallet network error: {e}"))?;

        let status = resp.status();
        let raw_text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read Privy create wallet response: {e}"))?;

        if !status.is_success() {
            return Err(format!(
                "Privy create wallet failed with status {status}: {raw_text}"
            ));
        }

        let wallet: PrivyWalletResponse = serde_json::from_str(&raw_text).map_err(|e| {
            format!("Failed to parse Privy create wallet response: {e} — Body: {raw_text}")
        })?;

        Ok(wallet)
    }

    /// Signs a 32-byte hash using the specified Privy server wallet ID.
    pub async fn sign_hash(
        &self,
        wallet_id: &str,
        hash: &[u8; 32],
    ) -> Result<([u8; 64], RecoveredKeyInfo), String> {
        let hash_hex = format!("0x{}", hex::encode(hash));
        let rpc_req = PrivyRpcSignRequest {
            method: "secp256k1_sign".to_string(),
            params: PrivyRpcSignParams { hash: hash_hex },
        };

        let url = format!("https://api.privy.io/v1/wallets/{wallet_id}/rpc");
        let resp = self
            .http_client
            .post(&url)
            .json(&rpc_req)
            .send()
            .await
            .map_err(|e| format!("Privy RPC network error: {e}"))?;

        let status = resp.status();
        let raw_text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read Privy response: {e}"))?;

        if !status.is_success() {
            return Err(format!("Privy RPC failed with status {status}: {raw_text}"));
        }

        let rpc_res: PrivyRpcResponse = serde_json::from_str(&raw_text)
            .map_err(|e| format!("Failed to parse Privy RPC response: {e} — Body: {raw_text}"))?;

        let sig_hex_raw = rpc_res
            .data
            .and_then(|d| d.signature)
            .or(rpc_res.signature)
            .ok_or_else(|| format!("Signature field missing in Privy response: {raw_text}"))?;

        let sig_bytes = hex::decode(sig_hex_raw.trim_start_matches("0x"))
            .map_err(|e| format!("Invalid signature hex: {e}"))?;

        let (rs_bytes, v_opt) = crypto::split_signature(&sig_bytes)?;
        let v_byte = v_opt.ok_or("Privy signature missing recovery ID v")?;

        let recovered = crypto::verify_and_recover(hash, rs_bytes, v_byte)?;

        let mut sig_64 = [0u8; 64];
        sig_64.copy_from_slice(rs_bytes);

        Ok((sig_64, recovered))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_sign_with_privy_client() {
        // Only run live network test if credentials are present
        if let Ok(client) = PrivyClient::new_from_env() {
            let wallet_res = client.create_ethereum_wallet().await;
            assert!(wallet_res.is_ok(), "Failed to create Privy wallet: {:?}", wallet_res.err());
            let wallet = wallet_res.unwrap();
            assert!(wallet.address.starts_with("0x"), "Address must start with 0x: {}", wallet.address);
            assert!(!wallet.id.is_empty(), "Wallet ID must not be empty");

            // Test signing a hash with this new wallet
            let test_hash = [0x42u8; 32];
            let sign_res = client.sign_hash(&wallet.id, &test_hash).await;
            assert!(sign_res.is_ok(), "Failed to sign with newly created Privy wallet: {:?}", sign_res.err());
            let (sig, recovered) = sign_res.unwrap();
            assert_eq!(sig.len(), 64);
            assert_eq!(recovered.derived_address.to_lowercase(), wallet.address.to_lowercase());
        }
    }
}
