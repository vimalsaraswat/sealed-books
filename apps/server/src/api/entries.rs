//! Journal entries API handlers and sub-router.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::AppState;
use crate::api::auth::AuthContext;
use crate::api::error::ApiError;
use crate::db::repository::entry::EntryExt;
use sealed_books_core::types::{Direction, Entry, Line};

/// Sub-router for entries mounted under a period (`/:id/entries`).
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_entries).post(post_entry))
}

/// Request payload for posting a new journal entry.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateEntryRequest {
    pub id: Option<String>,
    pub date: String,
    pub description: String,
    pub created_at: Option<String>,
    pub lines: Vec<CreateLineRequest>,
}

/// Request payload for a line item within a journal entry.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateLineRequest {
    pub id: Option<String>,
    pub account_id: String,
    pub direction: Direction,
    pub amount_minor: u64,
    pub description: Option<String>,
}

/// Lists all journal entries and line items for an accounting period in deterministic order.
pub async fn list_entries(
    State(state): State<AppState>,
    Path(period_id): Path<String>,
) -> Result<Json<Vec<Entry>>, ApiError> {
    let conn = state.db.conn();
    let entries = <Entry as EntryExt>::find_by_period(conn, &period_id).await?;
    Ok(Json(entries))
}

/// Posts a balanced journal entry to an open accounting period.
pub async fn post_entry(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(period_id): Path<String>,
    Json(payload): Json<CreateEntryRequest>,
) -> Result<(StatusCode, Json<Entry>), ApiError> {
    if auth.role == "auditor" {
        return Err(ApiError::Forbidden(
            "External Auditors have read-only audit access and cannot create or modify journal entries".into(),
        ));
    }

    // Generate IDs and timestamps if omitted
    let entry_id = payload
        .id
        .unwrap_or_else(|| format!("ent_{}", Uuid::new_v4().simple()));

    let created_at = payload
        .created_at
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let lines: Vec<Line> = payload
        .lines
        .into_iter()
        .map(|l| {
            let line_id =
                l.id.unwrap_or_else(|| format!("l_{}", Uuid::new_v4().simple()));
            Line::new(
                line_id,
                l.account_id,
                l.direction,
                l.amount_minor,
                l.description,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let entry = Entry {
        id: entry_id,
        date: payload.date,
        description: payload.description,
        created_at,
        lines,
    };

    let conn = state.db.conn();
    entry.post_to(conn, &period_id).await?;

    Ok((StatusCode::CREATED, Json(entry)))
}
