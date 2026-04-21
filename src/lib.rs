pub mod feeding;
pub mod handlers;
pub mod migration;
pub mod treat;

use axum::{
    routing::{delete, get, post},
    Router,
};
use sea_orm::DatabaseConnection;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub tx: broadcast::Sender<()>,
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/api/feeding", post(handlers::add_feeding))
        .route("/api/feeding/{id}", delete(handlers::delete_feeding))
        .route("/api/treat", post(handlers::add_treat))
        .route("/api/treat/{id}", delete(handlers::delete_treat))
        .route("/api/calendar/{year}/{month}", get(handlers::get_calendar))
        .route("/api/day/{date}", get(handlers::get_day))
        .route("/api/events", get(handlers::events))
        .fallback(handlers::serve_frontend)
        .with_state(state)
}
