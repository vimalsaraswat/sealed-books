//! Sealed Books Server library.

pub mod api;
pub mod db;
pub mod seed;

pub use api::{AppState, create_router};
pub use db::*;
pub use seed::{DEMO_ENTITY_NAME, DEMO_PERIOD_ID, seed_demo_data, seed_if_empty};
