use sealed_books_server::{AppState, Database, create_router, seed_if_empty};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize tracing subscriber with fallback to INFO
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Sealed Books Server initializing...");

    // Initialize Database
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "sealed_books.db".to_string());
    let db = Database::open(&db_path)?;
    tracing::info!("Database initialized at {}", db_path);

    // Auto-seed demo dataset if empty
    {
        let mut conn = db.lock();
        if seed_if_empty(&mut conn)? {
            tracing::info!("Demo dataset (Acme Trading Pvt Ltd, August 2026) seeded.");
        } else {
            tracing::info!("Existing ledger data detected; skipped demo seeding.");
        }
    }

    // Assemble Axum application router
    let state = AppState::new(db);
    let app = create_router(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("HTTP server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
