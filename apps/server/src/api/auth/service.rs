//! Core authentication services: key generation, organization provisioning, and session tokens.

use super::types::{LoginResponse, UserOrgSummary};
use crate::api::error::ApiError;
use crate::db::repository::membership::Membership;
use crate::db::repository::organization::Organization;
use crate::db::repository::session::Session;
use crate::db::repository::user::User;
use chrono::Utc;
use k256::ecdsa::SigningKey;
use k256::elliptic_curve::rand_core::OsRng;
use libsql::Connection;
use sha3::{Digest, Keccak256};
use uuid::Uuid;

/// Generates a real secp256k1 keypair and derives the Ethereum address for a new user.
pub fn generate_user_identity(email: &str, name: &str) -> User {
    let signing_key = SigningKey::random(&mut OsRng);
    let pubkey_bytes = signing_key.verifying_key().to_encoded_point(true);
    let pubkey_hex = format!("0x{}", hex::encode(pubkey_bytes.as_bytes()));

    let uncompressed = signing_key.verifying_key().to_encoded_point(false);
    let mut hasher = Keccak256::new();
    hasher.update(&uncompressed.as_bytes()[1..]);
    let hash = hasher.finalize();
    let eth_addr = format!("0x{}", hex::encode(&hash[12..]));

    let user_id = format!("usr_{}", &Uuid::new_v4().simple().to_string()[..8]);
    User {
        id: user_id,
        email: email.to_string(),
        name: name.to_string(),
        pubkey: pubkey_hex,
        eth_address: eth_addr,
        created_at: Utc::now().to_rfc3339(),
    }
}

/// Derives a readable display name from an email address (e.g. "elena.rostova@corp.com" -> "Elena Rostova").
pub fn derive_display_name(email: &str) -> String {
    let prefix = email.split('@').next().unwrap_or("User");
    let capitalized = prefix
        .split(['.', '_', '-'])
        .map(|part| {
            let mut c = part.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    if capitalized.is_empty() {
        "User".into()
    } else {
        capitalized
    }
}

/// Provisions a new user, their default enterprise organization, owner membership, and active session.
pub async fn provision_user_and_workspace(
    conn: &Connection,
    email: &str,
    name_opt: Option<&str>,
    org_name_opt: Option<&str>,
) -> Result<LoginResponse, ApiError> {
    let name = match name_opt.filter(|s| !s.trim().is_empty()) {
        Some(n) => n.trim().to_string(),
        None => derive_display_name(email),
    };

    let user = generate_user_identity(email, &name);
    user.insert(conn).await?;

    let org_name = match org_name_opt.filter(|s| !s.trim().is_empty()) {
        Some(o) => o.trim().to_string(),
        None => format!("{}'s Organization", name),
    };

    let org_id = format!("org_{}", &Uuid::new_v4().simple().to_string()[..8]);
    let org = Organization {
        id: org_id.clone(),
        name: org_name,
        base_currency: "USD".into(),
        created_at: Utc::now().to_rfc3339(),
    };
    org.insert(conn).await?;

    let membership = Membership {
        id: format!("mem_{}_{}", org_id, user.id),
        organization_id: org_id.clone(),
        user_id: user.id.clone(),
        role: "owner".into(),
        status: "active".into(),
        created_at: Utc::now().to_rfc3339(),
    };
    membership.insert(conn).await?;

    let token = format!(
        "sess_{}_{}",
        user.id,
        &Uuid::new_v4().simple().to_string()[..8]
    );
    let session = Session {
        token: token.clone(),
        user_id: user.id.clone(),
        active_organization_id: org_id.clone(),
        expires_at: "2099-01-01T00:00:00Z".into(),
    };
    session.insert(conn).await?;

    Ok(LoginResponse {
        token,
        user,
        active_organization: org.clone(),
        role: "owner".into(),
        available_organizations: vec![UserOrgSummary {
            id: org.id,
            name: org.name,
            base_currency: org.base_currency,
            role: "owner".into(),
        }],
    })
}

/// Builds a LoginResponse for an existing user in their primary organization.
pub async fn build_login_response_for_user(
    conn: &Connection,
    user: &User,
) -> Result<LoginResponse, ApiError> {
    let user_orgs = Organization::list_for_user(conn, &user.id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let (active_org, role) = if let Some(first) = user_orgs.first() {
        (first.0.clone(), first.1.clone())
    } else {
        // Automatically bootstrap organization if user has none
        let org_id = format!("org_{}", &Uuid::new_v4().simple().to_string()[..8]);
        let o = Organization {
            id: org_id.clone(),
            name: format!("{}'s Workspace", user.name),
            base_currency: "USD".into(),
            created_at: Utc::now().to_rfc3339(),
        };
        o.insert(conn).await?;

        let m = Membership {
            id: format!("mem_{}_{}", org_id, user.id),
            organization_id: org_id.clone(),
            user_id: user.id.clone(),
            role: "owner".into(),
            status: "active".into(),
            created_at: Utc::now().to_rfc3339(),
        };
        m.insert(conn).await?;
        (o, "owner".into())
    };

    let token = format!(
        "sess_{}_{}",
        user.id,
        &Uuid::new_v4().simple().to_string()[..8]
    );
    let session = Session {
        token: token.clone(),
        user_id: user.id.clone(),
        active_organization_id: active_org.id.clone(),
        expires_at: "2099-01-01T00:00:00Z".into(),
    };
    session.insert(conn).await?;

    let list = Organization::list_for_user(conn, &user.id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let summaries = list
        .into_iter()
        .map(|(o, r)| UserOrgSummary {
            id: o.id,
            name: o.name,
            base_currency: o.base_currency,
            role: r,
        })
        .collect();

    Ok(LoginResponse {
        token,
        user: user.clone(),
        active_organization: active_org,
        role,
        available_organizations: summaries,
    })
}
