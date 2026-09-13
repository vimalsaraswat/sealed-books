//! Organization management and auditor workspace APIs.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::api::auth::AuthContext;
use crate::api::error::ApiError;
use crate::db::repository::membership::Membership;
use crate::db::repository::organization::Organization;
use crate::db::repository::seal::SealRecord;
use crate::db::repository::user::User;

/// Router for organization resources mounted under `/api/organizations`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_organizations).post(create_organization))
        .route("/:id/members", get(list_organization_members))
        .route("/:id/invites", post(invite_member))
}

/// Router for auditor operations mounted under `/api/auditor`.
pub fn auditor_router() -> Router<AppState> {
    Router::new().route("/pending", get(list_pending_audits))
}

#[derive(Debug, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub base_currency: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InviteMemberRequest {
    pub email: String,
    pub name: Option<String>,
    pub role: String, // "admin", "controller", "auditor", "staff"
    pub pubkey: Option<String>,
    pub eth_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrgMemberResponse {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub eth_address: String,
    pub pubkey: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PendingAuditItem {
    pub period_id: String,
    pub organization_id: String,
    pub entity: String,
    pub start_date: String,
    pub end_date: String,
    pub statement_hash: String,
    pub root: String,
    pub dispatch_status: String,
    pub auditor_id: Option<String>,
    pub approver_1_pubkey: Option<String>,
    pub approver_1_sig: Option<String>,
    pub created_at: String,
}

/// GET /api/organizations: Lists all organizations the authenticated user belongs to.
async fn list_organizations(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<Vec<crate::api::auth::UserOrgSummary>>, ApiError> {
    let conn = state.db.conn();
    let orgs = Organization::list_for_user(conn, &auth.user.id).await?;
    let summaries = orgs
        .into_iter()
        .map(|(o, role)| crate::api::auth::UserOrgSummary {
            id: o.id,
            name: o.name,
            base_currency: o.base_currency,
            role,
        })
        .collect();

    Ok(Json(summaries))
}

/// POST /api/organizations: Creates a new organization with the caller as 'owner'.
async fn create_organization(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<CreateOrganizationRequest>,
) -> Result<(StatusCode, Json<Organization>), ApiError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest(
            "Organization name cannot be empty".into(),
        ));
    }

    let currency = payload.base_currency.unwrap_or_else(|| "USD".into());
    let org_id = format!("org_{}", &Uuid::new_v4().simple().to_string()[..8]);

    let org = Organization {
        id: org_id.clone(),
        name,
        base_currency: currency,
        created_at: Utc::now().to_rfc3339(),
    };

    let membership = Membership {
        id: format!("mem_{}_{}", &org_id, &auth.user.id),
        organization_id: org_id,
        user_id: auth.user.id.clone(),
        role: "owner".into(),
        status: "active".into(),
        created_at: Utc::now().to_rfc3339(),
    };

    let conn = state.db.conn();
    org.insert(conn).await?;
    membership.insert(conn).await?;

    Ok((StatusCode::CREATED, Json(org)))
}

/// GET /api/organizations/:id/members: Lists all members and roles of an organization.
async fn list_organization_members(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(org_id): Path<String>,
) -> Result<Json<Vec<OrgMemberResponse>>, ApiError> {
    let conn = state.db.conn();
    // Verify caller is a member of this org
    Membership::find(conn, &org_id, &auth.user.id).await?;

    let members = Membership::list_by_org(conn, &org_id).await?;
    let mut responses = Vec::new();

    for (m, u) in members {
        responses.push(OrgMemberResponse {
            id: m.id,
            user_id: u.id,
            name: u.name,
            email: u.email,
            role: m.role,
            status: m.status,
            eth_address: u.eth_address,
            pubkey: u.pubkey,
            created_at: m.created_at,
        });
    }

    Ok(Json(responses))
}

/// POST /api/organizations/:id/invites: Invites or assigns a new user into the organization.
async fn invite_member(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(org_id): Path<String>,
    Json(payload): Json<InviteMemberRequest>,
) -> Result<(StatusCode, Json<OrgMemberResponse>), ApiError> {
    if auth.role != "owner" && auth.role != "admin" {
        return Err(ApiError::Forbidden(
            "Only Organization Owners and Admins can invite new members".into(),
        ));
    }

    let role = payload.role.trim().to_lowercase();
    let valid_roles = ["admin", "controller", "auditor", "staff"];
    if !valid_roles.contains(&role.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Invalid role '{role}'. Allowed roles: admin, controller, auditor, staff"
        )));
    }

    let email = payload.email.trim().to_lowercase();
    if email.is_empty() {
        return Err(ApiError::BadRequest("Email cannot be empty".into()));
    }

    let conn = state.db.conn();

    // Check if user exists or register them
    let user = match User::find_by_email(conn, &email).await {
        Ok(u) => u,
        Err(_) => {
            let uid = format!("usr_{}", &Uuid::new_v4().simple().to_string()[..8]);
            let name = payload
                .name
                .unwrap_or_else(|| email.split('@').next().unwrap_or("User").to_string());
            let pubkey = payload.pubkey.unwrap_or_default();
            let eth_address = payload.eth_address.unwrap_or_default();
            let new_user = User {
                id: uid,
                email: email.clone(),
                name,
                pubkey,
                eth_address,
                wallet_id: None,
                created_at: Utc::now().to_rfc3339(),
            };
            new_user.insert(conn).await?;
            new_user
        }
    };

    // Check if membership already exists
    if let Ok(existing) = Membership::find(conn, &org_id, &user.id).await {
        return Err(ApiError::Conflict(format!(
            "User {} is already a member with role '{}'",
            email, existing.role
        )));
    }

    let membership = Membership {
        id: format!("mem_{}_{}", &org_id[..6.min(org_id.len())], &user.id),
        organization_id: org_id.clone(),
        user_id: user.id.clone(),
        role: role.clone(),
        status: "active".into(),
        created_at: Utc::now().to_rfc3339(),
    };

    membership.insert(conn).await?;

    Ok((
        StatusCode::CREATED,
        Json(OrgMemberResponse {
            id: membership.id,
            user_id: user.id,
            name: user.name,
            email: user.email,
            role,
            status: "active".into(),
            eth_address: user.eth_address,
            pubkey: user.pubkey,
            created_at: membership.created_at,
        }),
    ))
}

/// GET /api/auditor/pending: Lists periods awaiting review for the authenticated auditor.
async fn list_pending_audits(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<Vec<PendingAuditItem>>, ApiError> {
    let conn = state.db.conn();
    let rows = SealRecord::list_pending_audits(conn, Some(&auth.user.id)).await?;

    let items = rows
        .into_iter()
        .map(|(s, p)| PendingAuditItem {
            period_id: p.id,
            organization_id: p.organization_id,
            entity: p.entity,
            start_date: p.start_date,
            end_date: p.end_date,
            statement_hash: s.statement_hash,
            root: s.root,
            dispatch_status: s.dispatch_status,
            auditor_id: s.auditor_id,
            approver_1_pubkey: s.approver_1_pubkey,
            approver_1_sig: s.approver_1_sig,
            created_at: s.created_at,
        })
        .collect();

    Ok(Json(items))
}
