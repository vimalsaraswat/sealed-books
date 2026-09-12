//! Request and response Data Transfer Objects for authentication.

use crate::db::repository::organization::Organization;
use crate::db::repository::user::User;
use serde::{Deserialize, Serialize};

/// Context extracted from request authorization headers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user: User,
    pub active_organization: Organization,
    pub role: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub token: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub organization_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
    pub active_organization: Organization,
    pub role: String,
    pub available_organizations: Vec<UserOrgSummary>,
}

#[derive(Debug, Serialize)]
pub struct UserOrgSummary {
    pub id: String,
    pub name: String,
    pub base_currency: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct SwitchOrgRequest {
    pub organization_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SendOtpRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct SendOtpResponse {
    pub status: String,
    pub email: String,
    pub is_new_user: bool,
}

#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub code: String,
    pub name: Option<String>,
    pub organization_name: Option<String>,
}
