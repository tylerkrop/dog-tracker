mod feeding;
mod handlers;
mod migration;
mod treat;

use axum::{
    routing::{delete, get, post},
    Router,
};
use sea_orm::{Database, DatabaseConnection};
use std::net::SocketAddr;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub tx: broadcast::Sender<()>,
}

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./dog_tracker.db?mode=rwc".to_string());

    let db = Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");

    migration::run(&db).await.expect("Failed to run migrations");

    let (tx, _) = broadcast::channel::<()>(16);
    let state = AppState { db, tx };

    let app = Router::new()
        .route("/api/feeding", post(handlers::add_feeding))
        .route("/api/feeding/{id}", delete(handlers::delete_feeding))
        .route("/api/treat", post(handlers::add_treat))
        .route("/api/treat/{id}", delete(handlers::delete_treat))
        .route("/api/calendar/{year}/{month}", get(handlers::get_calendar))
        .route("/api/day/{date}", get(handlers::get_day))
        .route("/api/events", get(handlers::events))
        .fallback(handlers::serve_frontend)
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let host: [u8; 4] = if std::env::var("BIND_ALL").is_ok() {
        [0, 0, 0, 0]
    } else {
        [127, 0, 0, 1]
    };
    let addr = SocketAddr::from((host, port));
    println!("Listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}
