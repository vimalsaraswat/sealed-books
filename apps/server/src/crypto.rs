//! Cryptographic signature verification and public key recovery for Sealed Books Server.

use k256::ecdsa::signature::hazmat::PrehashVerifier;
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use sha3::{Digest as _, Keccak256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveredKeyInfo {
    pub compressed_pubkey: [u8; 33],
    pub compressed_pubkey_hex: String,
    pub derived_address: String,
}

/// Splits a signature (either 65-byte r||s||v or 64-byte r||s) into (64-byte r||s, optional recovery_id byte v).
pub fn split_signature(sig_bytes: &[u8]) -> Result<(&[u8], Option<u8>), String> {
    match sig_bytes.len() {
        65 => Ok((&sig_bytes[0..64], Some(sig_bytes[64]))),
        64 => Ok((sig_bytes, None)),
        other => Err(format!(
            "Expected 64-byte or 65-byte signature, got {other} bytes"
        )),
    }
}

/// Derives an Ethereum address from a VerifyingKey (Keccak-256 of uncompressed pubkey without 0x04 tag).
pub fn derive_eth_address(vk: &VerifyingKey) -> String {
    let uncompressed = vk.to_encoded_point(false);
    let uncompressed_slice = &uncompressed.as_bytes()[1..]; // drop 0x04 tag
    let address_hash = Keccak256::digest(uncompressed_slice);
    format!("0x{}", hex::encode(&address_hash[12..32]))
}

/// Verifies a 64-byte signature and recovers the signer's compressed public key and Ethereum address.
pub fn verify_and_recover(
    prehash: &[u8],
    sig_64: &[u8],
    v_byte: u8,
) -> Result<RecoveredKeyInfo, String> {
    let signature =
        Signature::from_slice(sig_64).map_err(|e| format!("Invalid 64-byte signature: {e}"))?;

    // Normalize recovery ID (27/28 -> 0/1)
    let rec_id_num = if v_byte >= 27 { v_byte - 27 } else { v_byte };
    let rec_id = RecoveryId::try_from(rec_id_num)
        .map_err(|e| format!("Invalid recovery ID {v_byte}: {e}"))?;

    let recovered_vk = VerifyingKey::recover_from_prehash(prehash, &signature, rec_id)
        .map_err(|e| format!("Public key recovery failed: {e}"))?;

    let compressed = recovered_vk.to_encoded_point(true);
    let compressed_bytes = compressed.as_bytes();
    let mut compressed_pubkey = [0u8; 33];
    compressed_pubkey.copy_from_slice(compressed_bytes);

    recovered_vk
        .verify_prehash(prehash, &signature)
        .map_err(|e| format!("Signature prehash verification failed: {e}"))?;

    let derived_address = derive_eth_address(&recovered_vk);

    Ok(RecoveredKeyInfo {
        compressed_pubkey,
        compressed_pubkey_hex: format!("0x{}", hex::encode(compressed_pubkey)),
        derived_address,
    })
}

/// Verifies a signature (64-byte or 65-byte) against an explicitly provided 33-byte compressed public key.
pub fn verify_with_pubkey(
    prehash: &[u8],
    sig_bytes: &[u8],
    pubkey_bytes: &[u8],
) -> Result<(), String> {
    let (sig_64, _) = split_signature(sig_bytes)?;
    let signature = Signature::from_slice(sig_64).map_err(|e| format!("Invalid signature: {e}"))?;
    let vk = VerifyingKey::from_sec1_bytes(pubkey_bytes)
        .map_err(|e| format!("Invalid sec1 public key bytes: {e}"))?;
    vk.verify_prehash(prehash, &signature)
        .map_err(|e| format!("Signature verification failed: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::SigningKey;
    use k256::elliptic_curve::rand_core::OsRng;
    use k256::sha2::{Digest, Sha256};

    #[test]
    fn test_signature_verify_and_recovery() {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let expected_address = derive_eth_address(&verifying_key);

        let message = b"Sealed Books Signature Test";
        let prehash = Sha256::digest(message);
        let (signature, rec_id) = signing_key
            .sign_prehash_recoverable(&prehash)
            .expect("sign prehash");

        let mut sig_65 = Vec::new();
        sig_65.extend_from_slice(signature.to_bytes().as_slice());
        sig_65.push(rec_id.to_byte());

        let (sig_64, v) = split_signature(&sig_65).expect("split signature");
        assert_eq!(v, Some(rec_id.to_byte()));

        let recovered = verify_and_recover(&prehash, sig_64, v.unwrap()).expect("recover");
        assert_eq!(
            recovered.derived_address.to_lowercase(),
            expected_address.to_lowercase()
        );

        let compressed = verifying_key.to_encoded_point(true);
        verify_with_pubkey(&prehash, sig_64, compressed.as_bytes()).expect("verify with pubkey 64");
        verify_with_pubkey(&prehash, &sig_65, compressed.as_bytes())
            .expect("verify with pubkey 65");
    }
}
