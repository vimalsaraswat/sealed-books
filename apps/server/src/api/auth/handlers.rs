//! HTTP route handlers for authentication and tenancy workflows.

use super::otp::{generate_otp_code, store_otp, verify_and_consume_otp};
use super::service::{build_login_response_for_user, provision_user_and_workspace};
use super::types::{
    AuthContext, LoginRequest, LoginResponse, RegisterRequest, SendOtpRequest, SendOtpResponse,
    SwitchOrgRequest, UserOrgSummary, VerifyOtpRequest,
};
use crate::api::AppState;
use crate::api::error::ApiError;
use crate::db::repository::membership::Membership;
use crate::db::repository::organization::Organization;
use crate::db::repository::session::Session;
use crate::db::repository::user::User;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

/// GET /api/auth/me: Returns current authenticated user profile, active organization, and permissions.
pub async fn get_me(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<LoginResponse>, ApiError> {
    let conn = state.db.conn();
    let orgs = Organization::list_for_user(conn, &auth.user.id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let summaries = orgs
        .into_iter()
        .map(|(org, role)| UserOrgSummary {
            id: org.id,
            name: org.name,
            base_currency: org.base_currency,
            role,
        })
        .collect();

    Ok(Json(LoginResponse {
        token: auth.token,
        user: auth.user,
        active_organization: auth.active_organization,
        role: auth.role,
        available_organizations: summaries,
    }))
}

/// POST /api/auth/login: Direct login by email, userId, or existing session token.
pub async fn post_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let conn = state.db.conn();

    let target_user = if let Some(ref email) = payload.email {
        User::find_by_email(conn, email.trim().to_lowercase().as_str())
            .await
            .map_err(|_| ApiError::Unauthorized("Invalid email address".into()))?
    } else if let Some(ref uid) = payload.user_id {
        User::find_by_id(conn, uid)
            .await
            .map_err(|_| ApiError::Unauthorized("Invalid user id".into()))?
    } else if let Some(ref token) = payload.token {
        let session = Session::find_by_token(conn, token)
            .await
            .map_err(|_| ApiError::Unauthorized("Invalid session token".into()))?;
        User::find_by_id(conn, &session.user_id)
            .await
            .map_err(|_| ApiError::Unauthorized("User not found for session".into()))?
    } else {
        return Err(ApiError::BadRequest(
            "One of 'email', 'user_id', or 'token' must be provided".into(),
        ));
    };

    let resp = build_login_response_for_user(conn, &target_user).await?;
    Ok(Json(resp))
}

/// POST /api/auth/register: Registers a brand new user and creates an organization.
pub async fn post_register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), ApiError> {
    let email = payload.email.trim().to_lowercase();
    if !email.contains('@') || !email.contains('.') {
        return Err(ApiError::BadRequest("Invalid email address".into()));
    }
    if payload.name.trim().is_empty() {
        return Err(ApiError::BadRequest("Name cannot be empty".into()));
    }

    let conn = state.db.conn();
    if User::find_by_email(conn, &email).await.is_ok() {
        return Err(ApiError::Conflict(
            "A user with this email address already exists. Please log in.".into(),
        ));
    }

    let resp = provision_user_and_workspace(
        conn,
        &email,
        Some(&payload.name),
        payload.organization_name.as_deref(),
    )
    .await?;

    Ok((StatusCode::CREATED, Json(resp)))
}

/// POST /api/auth/logout: Revokes and destroys the active session token.
pub async fn post_logout(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<StatusCode, ApiError> {
    let conn = state.db.conn();
    let _ = Session::delete(conn, &auth.token).await;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/auth/switch-org: Switches active organization context for the current session.
pub async fn post_switch_org(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<SwitchOrgRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let conn = state.db.conn();
    Membership::find(conn, &payload.organization_id, &auth.user.id)
        .await
        .map_err(|_| {
            ApiError::Forbidden(format!(
                "You are not a member of organization {}",
                payload.organization_id
            ))
        })?;

    Session::update_active_org(conn, &auth.token, &payload.organization_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let o = Organization::find_by_id(conn, &payload.organization_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let m = Membership::find(conn, &payload.organization_id, &auth.user.id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let list = Organization::list_for_user(conn, &auth.user.id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let summaries = list
        .into_iter()
        .map(|(org, r)| UserOrgSummary {
            id: org.id,
            name: org.name,
            base_currency: org.base_currency,
            role: r,
        })
        .collect();

    Ok(Json(LoginResponse {
        token: auth.token,
        user: auth.user,
        active_organization: o,
        role: m.role,
        available_organizations: summaries,
    }))
}

/// POST /api/auth/otp/send: Generates a 6-digit OTP code and dispatches an email.
pub async fn post_send_otp(
    State(state): State<AppState>,
    Json(payload): Json<SendOtpRequest>,
) -> Result<Json<SendOtpResponse>, ApiError> {
    let email = payload.email.trim().to_lowercase();
    if !email.contains('@') || !email.contains('.') {
        return Err(ApiError::BadRequest(
            "Please enter a valid email address".into(),
        ));
    }

    let conn = state.db.conn();
    let is_new_user = User::find_by_email(conn, &email).await.is_err();

    let code = generate_otp_code();

    store_otp(conn, &email, &code).await?;

    // Dispatch real email via EmailService (Resend in production, logs/dev in local)
    let _ = state.email.send_otp(&email, &code).await;

    Ok(Json(SendOtpResponse {
        status: "sent".into(),
        email,
        is_new_user,
    }))
}

/// POST /api/auth/otp/verify: Verifies 6-digit OTP code, signs in existing user or registers new user with default organization.
pub async fn post_verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<VerifyOtpRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), ApiError> {
    let email = payload.email.trim().to_lowercase();
    let submitted_code = payload.code.trim().to_string();

    if email.is_empty() || submitted_code.is_empty() {
        return Err(ApiError::BadRequest(
            "Email and verification code are required".into(),
        ));
    }

    let conn = state.db.conn();
    let is_valid = verify_and_consume_otp(conn, &email, &submitted_code).await?;
    if !is_valid {
        return Err(ApiError::Unauthorized(
            "Invalid or expired verification code".into(),
        ));
    }

    let existing_user = User::find_by_email(conn, &email).await.ok();

    if let Some(user) = existing_user {
        let resp = build_login_response_for_user(conn, &user).await?;
        return Ok((StatusCode::OK, Json(resp)));
    }

    // Brand new user: Automatic registration with default organization
    let resp = provision_user_and_workspace(
        conn,
        &email,
        payload.name.as_deref(),
        payload.organization_name.as_deref(),
    )
    .await?;

    Ok((StatusCode::CREATED, Json(resp)))
}
