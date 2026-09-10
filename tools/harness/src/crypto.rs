//! Cryptographic signature verification and public key recovery for Phase 0.
//!
//! Handles:
//! - Parsing 65-byte secp256k1 signatures (r || s || v) from Privy.
//! - Extracting 64-byte (r || s) for Hedera consensus messages.
//! - Recovering 33-byte compressed secp256k1 public keys using recovery ID `v`.
//! - Deriving Ethereum checksum/hex addresses from recovered public keys via Keccak-256.
//! - Local signature verification without external network calls.

use k256::ecdsa::signature::hazmat::PrehashVerifier;
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use sha3::{Digest as _, Keccak256};

pub struct RecoveredKeyInfo {
    pub compressed_pubkey_hex: String,
    pub derived_address: String,
}

/// Splits a 65-byte signature into (64-byte r||s, recovery_id byte v).
pub fn split_signature(sig_bytes: &[u8]) -> Result<(&[u8], u8), String> {
    if sig_bytes.len() != 65 {
        return Err(format!(
            "Expected 65-byte signature (r || s || v), got {} bytes",
            sig_bytes.len()
        ));
    }
    Ok((&sig_bytes[0..64], sig_bytes[64]))
}

/// Derives an Ethereum address from a VerifyingKey (Keccak256 of uncompressed pubkey without 0x04 tag).
pub fn derive_eth_address(vk: &VerifyingKey) -> String {
    let uncompressed = vk.to_encoded_point(false);
    let uncompressed_slice = &uncompressed.as_bytes()[1..]; // drop 0x04 tag
    let address_hash = Keccak256::digest(uncompressed_slice);
    format!("0x{}", hex::encode(&address_hash[12..32]))
}

/// Verifies a signature against a prehash and recovers the public key and Ethereum address.
pub fn verify_and_recover(
    prehash: &[u8],
    sig_64: &[u8],
    v_byte: u8,
) -> Result<RecoveredKeyInfo, Box<dyn std::error::Error>> {
    let signature = Signature::from_slice(sig_64)?;

    // Normalize recovery ID (27/28 -> 0/1)
    let rec_id_num = if v_byte >= 27 { v_byte - 27 } else { v_byte };
    let rec_id = RecoveryId::try_from(rec_id_num)?;

    // Recover public key from prehash
    let recovered_vk = VerifyingKey::recover_from_prehash(prehash, &signature, rec_id)?;
    let compressed = recovered_vk.to_encoded_point(true);
    let compressed_hex = hex::encode(compressed.as_bytes());

    // Verify signature with the recovered key
    recovered_vk.verify_prehash(prehash, &signature)?;

    let derived_address = derive_eth_address(&recovered_vk);

    Ok(RecoveredKeyInfo {
        compressed_pubkey_hex: compressed_hex,
        derived_address,
    })
}

/// Verifies a signature against an explicitly provided compressed or uncompressed public key.
pub fn verify_with_pubkey(
    prehash: &[u8],
    sig_64: &[u8],
    pubkey_bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let signature = Signature::from_slice(sig_64)?;
    let vk = VerifyingKey::from_sec1_bytes(pubkey_bytes)?;
    vk.verify_prehash(prehash, &signature)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::SigningKey;
    use k256::elliptic_curve::rand_core::OsRng;
    use sha2::{Digest, Sha256};

    #[test]
    fn test_secp256k1_sign_verify_and_recovery() {
        // 1. Generate local key pair
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let expected_address = derive_eth_address(&verifying_key);

        // 2. Sign prehash
        let message = b"Sealed Books Modular Phase 0 Test";
        let prehash = Sha256::digest(message);
        let (signature, rec_id) = signing_key
            .sign_prehash_recoverable(&prehash)
            .expect("sign prehash");

        // Simulate 65-byte Privy response
        let mut privy_sig = Vec::new();
        privy_sig.extend_from_slice(signature.to_bytes().as_slice());
        privy_sig.push(rec_id.to_byte());

        // 3. Test splitting
        let (sig_64, v) = split_signature(&privy_sig).expect("split signature");
        assert_eq!(sig_64.len(), 64);
        assert_eq!(v, rec_id.to_byte());

        // 4. Test recovery & address derivation
        let recovered = verify_and_recover(&prehash, sig_64, v).expect("verify and recover");
        assert_eq!(
            recovered.derived_address.to_lowercase(),
            expected_address.to_lowercase()
        );

        // 5. Test direct pubkey verification
        let compressed = verifying_key.to_encoded_point(true);
        verify_with_pubkey(&prehash, sig_64, compressed.as_bytes()).expect("verify with pubkey");
    }
}
