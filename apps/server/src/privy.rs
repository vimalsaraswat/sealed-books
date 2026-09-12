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

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| format!("Failed to build reqwest client: {e}"))?;

        Ok(Self {
            http_client,
            _app_id: app_id,
        })
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
