use dog_tracker::{build_app, migration, AppState};
use sea_orm::Database;
use std::net::SocketAddr;
use tokio::sync::broadcast;

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

    let app = build_app(state);

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
