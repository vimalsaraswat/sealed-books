//! OTP code generation, expiration, and database lifecycle management.

use crate::api::error::ApiError;
use chrono::{DateTime, Utc};
use k256::elliptic_curve::rand_core::{OsRng, RngCore};
use rusqlite::Connection;

/// Generates a cryptographically secure 6-digit numeric OTP code.
pub fn generate_otp_code() -> String {
    let mut rng = OsRng;
    format!("{:06}", (rng.next_u32() % 900_000) + 100_000)
}

/// Stores or updates an active OTP code for an email address with a 15-minute TTL.
pub fn store_otp(conn: &Connection, email: &str, code: &str) -> Result<(), ApiError> {
    let expires_at = (Utc::now() + chrono::Duration::minutes(15)).to_rfc3339();
    let created_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO auth_otps (email, code, expires_at, created_at) VALUES (?1, ?2, ?3, ?4);",
        rusqlite::params![email, code, expires_at, created_at],
    )
    .map_err(|e| ApiError::Internal(format!("Failed to store verification code: {e}")))?;

    Ok(())
}

/// Verifies a submitted OTP code for an email address and consumes it if valid and not expired.
pub fn verify_and_consume_otp(
    conn: &Connection,
    email: &str,
    submitted_code: &str,
) -> Result<bool, ApiError> {
    let stored: Result<(String, String), rusqlite::Error> = conn.query_row(
        "SELECT code, expires_at FROM auth_otps WHERE email = ?1;",
        rusqlite::params![email],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    let (stored_code, expires_at_str) = match stored {
        Ok(vals) => vals,
        Err(_) => return Ok(false),
    };

    // Check expiration against current time (collapsed per clippy::collapsible_if)
    if let Ok(expires_at) = DateTime::parse_from_rfc3339(&expires_at_str)
        && Utc::now() > expires_at.with_timezone(&Utc)
    {
        // Expired: prune record and reject
        let _ = conn.execute(
            "DELETE FROM auth_otps WHERE email = ?1;",
            rusqlite::params![email],
        );
        return Ok(false);
    }

    // Exact match check only — no universal dev bypasses
    let is_valid = stored_code == submitted_code;

    if is_valid {
        let _ = conn.execute(
            "DELETE FROM auth_otps WHERE email = ?1;",
            rusqlite::params![email],
        );
    }

    Ok(is_valid)
}
