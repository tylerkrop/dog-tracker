use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use dog_tracker::{build_app, migration, AppState};
use http_body_util::BodyExt;
use sea_orm::{ConnectOptions, Database};
use serde_json::{json, Value};
use tokio::sync::broadcast;
use tower::ServiceExt;

async fn setup() -> Router {
    let mut opts = ConnectOptions::new("sqlite::memory:");
    // In-memory sqlite is per-connection; pin to a single connection so all
    // queries hit the same database.
    opts.max_connections(1).min_connections(1);
    let db = Database::connect(opts).await.expect("connect");
    migration::run(&db).await.expect("migrate");
    let (tx, _) = broadcast::channel::<()>(16);
    build_app(AppState { db, tx })
}

async fn send(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    let body = match body {
        Some(v) => Body::from(serde_json::to_vec(&v).unwrap()),
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .expect("response");
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, value)
}

// ── POST /api/feeding ───────────────────────────────────────────

#[tokio::test]
async fn add_feeding_one_half_scoop() {
    let app = setup().await;
    let (status, body) = send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({ "amount_half_scoops": 1, "fed_at": "2024-05-15 08:30:00" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["amount_half_scoops"], 1);
    assert_eq!(body["scoops"], 0.5);
    assert_eq!(body["fed_at"], "2024-05-15 08:30:00");
    assert_eq!(body["edited"], false);
}

#[tokio::test]
async fn add_feeding_full_scoop_with_edited_flag() {
    let app = setup().await;
    let (status, body) = send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({
            "amount_half_scoops": 2,
            "fed_at": "2020-01-15 12:00:00",
            "edited": true,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["scoops"], 1.0);
    assert_eq!(body["edited"], true);
}

#[tokio::test]
async fn add_feeding_defaults_edited_to_false() {
    let app = setup().await;
    let (_, body) = send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({ "amount_half_scoops": 1, "fed_at": "2024-05-15 08:30:00" })),
    )
    .await;
    assert_eq!(body["edited"], false);
}

#[tokio::test]
async fn add_feeding_rejects_invalid_amount() {
    let app = setup().await;
    for bad in [0, 3, -1, 10] {
        let (status, _) = send(
            &app,
            "POST",
            "/api/feeding",
            Some(json!({ "amount_half_scoops": bad, "fed_at": "2024-05-15 08:30:00" })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "amount {bad} should fail");
    }
}

#[tokio::test]
async fn add_feeding_rejects_invalid_timestamp_format() {
    let app = setup().await;
    for bad in ["2024-05-15", "not-a-date", "2024-05-15T08:30:00", ""] {
        let (status, _) = send(
            &app,
            "POST",
            "/api/feeding",
            Some(json!({ "amount_half_scoops": 1, "fed_at": bad })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "fed_at {bad:?} should fail");
    }
}

#[tokio::test]
async fn add_feeding_rejects_missing_fed_at() {
    let app = setup().await;
    let (status, _) = send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({ "amount_half_scoops": 1 })),
    )
    .await;
    // Missing required field — axum's Json extractor returns 422 for this.
    assert!(
        status.is_client_error(),
        "expected 4xx, got {status}",
    );
}

// ── DELETE /api/feeding/:id ─────────────────────────────────────

#[tokio::test]
async fn delete_feeding_round_trip() {
    let app = setup().await;
    let (_, body) = send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({ "amount_half_scoops": 2, "fed_at": "2024-05-15 08:30:00" })),
    )
    .await;
    let id = body["id"].as_i64().unwrap();

    let (status, _) = send(&app, "DELETE", &format!("/api/feeding/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, day) = send(&app, "GET", "/api/day/2024-05-15", None).await;
    assert_eq!(day["total_scoops"], 0.0);
}

#[tokio::test]
async fn delete_feeding_missing_returns_404() {
    let app = setup().await;
    let (status, _) = send(&app, "DELETE", "/api/feeding/9999", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── POST /api/treat ─────────────────────────────────────────────

#[tokio::test]
async fn add_treat_basic() {
    let app = setup().await;
    let (status, body) = send(
        &app,
        "POST",
        "/api/treat",
        Some(json!({ "name": "Bone", "given_at": "2024-05-15 09:00:00" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Bone");
    assert_eq!(body["given_at"], "2024-05-15 09:00:00");
    assert_eq!(body["edited"], false);
}

#[tokio::test]
async fn add_treat_trims_name() {
    let app = setup().await;
    let (status, body) = send(
        &app,
        "POST",
        "/api/treat",
        Some(json!({ "name": "  Pup Cup  ", "given_at": "2024-05-15 09:00:00" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Pup Cup");
}

#[tokio::test]
async fn add_treat_empty_name_rejected() {
    let app = setup().await;
    for name in ["", "   ", "\t\n"] {
        let (status, _) = send(
            &app,
            "POST",
            "/api/treat",
            Some(json!({ "name": name, "given_at": "2024-05-15 09:00:00" })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "name {name:?} should fail");
    }
}

#[tokio::test]
async fn add_treat_with_edited_flag() {
    let app = setup().await;
    let (status, body) = send(
        &app,
        "POST",
        "/api/treat",
        Some(json!({
            "name": "Scraps",
            "given_at": "2021-06-01 12:00:00",
            "edited": true,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["edited"], true);
    assert_eq!(body["given_at"], "2021-06-01 12:00:00");
}

#[tokio::test]
async fn add_treat_rejects_invalid_timestamp() {
    let app = setup().await;
    let (status, _) = send(
        &app,
        "POST",
        "/api/treat",
        Some(json!({ "name": "Bone", "given_at": "not-a-timestamp" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ── DELETE /api/treat/:id ───────────────────────────────────────

#[tokio::test]
async fn delete_treat_round_trip() {
    let app = setup().await;
    let (_, body) = send(
        &app,
        "POST",
        "/api/treat",
        Some(json!({ "name": "Bone", "given_at": "2024-05-15 09:00:00" })),
    )
    .await;
    let id = body["id"].as_i64().unwrap();

    let (status, _) = send(&app, "DELETE", &format!("/api/treat/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_treat_missing_returns_404() {
    let app = setup().await;
    let (status, _) = send(&app, "DELETE", "/api/treat/12345", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── GET /api/day/:date ──────────────────────────────────────────

#[tokio::test]
async fn get_day_returns_only_that_days_entries() {
    let app = setup().await;
    send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({ "amount_half_scoops": 2, "fed_at": "2022-03-10 08:00:00" })),
    )
    .await;
    send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({ "amount_half_scoops": 1, "fed_at": "2022-03-10 18:00:00" })),
    )
    .await;
    send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({ "amount_half_scoops": 2, "fed_at": "2022-03-11 08:00:00" })),
    )
    .await;
    send(
        &app,
        "POST",
        "/api/treat",
        Some(json!({ "name": "Bone", "given_at": "2022-03-10 09:00:00" })),
    )
    .await;

    let (status, body) = send(&app, "GET", "/api/day/2022-03-10", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["date"], "2022-03-10");
    assert_eq!(body["total_scoops"], 1.5);
    assert_eq!(body["feedings"].as_array().unwrap().len(), 2);
    assert_eq!(body["treats"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn get_day_empty_returns_zeros() {
    let app = setup().await;
    let (status, body) = send(&app, "GET", "/api/day/2022-03-10", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_scoops"], 0.0);
    assert_eq!(body["feedings"].as_array().unwrap().len(), 0);
    assert_eq!(body["treats"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn get_day_invalid_date_format() {
    let app = setup().await;
    let (status, _) = send(&app, "GET", "/api/day/2022-99-99", None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ── GET /api/calendar/:year/:month ──────────────────────────────

#[tokio::test]
async fn calendar_aggregates_feedings_and_treats() {
    let app = setup().await;
    send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({ "amount_half_scoops": 2, "fed_at": "2022-05-01 08:00:00" })),
    )
    .await;
    send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({ "amount_half_scoops": 1, "fed_at": "2022-05-01 18:00:00" })),
    )
    .await;
    send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({ "amount_half_scoops": 2, "fed_at": "2022-05-15 08:00:00" })),
    )
    .await;
    send(
        &app,
        "POST",
        "/api/treat",
        Some(json!({ "name": "Bone", "given_at": "2022-05-01 09:00:00" })),
    )
    .await;
    send(
        &app,
        "POST",
        "/api/treat",
        Some(json!({ "name": "Bone", "given_at": "2022-05-01 14:00:00" })),
    )
    .await;
    // Different month — must not appear
    send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({ "amount_half_scoops": 2, "fed_at": "2022-06-02 08:00:00" })),
    )
    .await;

    let (status, body) = send(&app, "GET", "/api/calendar/2022/5", None).await;
    assert_eq!(status, StatusCode::OK);
    let days = body.as_array().unwrap();
    assert_eq!(days.len(), 2);
    assert_eq!(days[0]["date"], "2022-05-01");
    assert_eq!(days[0]["total_scoops"], 1.5);
    assert_eq!(days[0]["treat_count"], 2);
    assert_eq!(days[1]["date"], "2022-05-15");
    assert_eq!(days[1]["total_scoops"], 1.0);
    assert_eq!(days[1]["treat_count"], 0);
}

#[tokio::test]
async fn calendar_handles_december_year_boundary() {
    let app = setup().await;
    send(
        &app,
        "POST",
        "/api/feeding",
        Some(json!({ "amount_half_scoops": 2, "fed_at": "2022-12-31 08:00:00" })),
    )
    .await;
    let (status, body) = send(&app, "GET", "/api/calendar/2022/12", None).await;
    assert_eq!(status, StatusCode::OK);
    let days = body.as_array().unwrap();
    assert_eq!(days.len(), 1);
    assert_eq!(days[0]["date"], "2022-12-31");
}

#[tokio::test]
async fn calendar_invalid_month_rejected() {
    let app = setup().await;
    let (status, _) = send(&app, "GET", "/api/calendar/2022/13", None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn calendar_empty_month_returns_empty_list() {
    let app = setup().await;
    let (status, body) = send(&app, "GET", "/api/calendar/2000/1", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

// ── /api/events (SSE) ───────────────────────────────────────────

#[tokio::test]
async fn events_endpoint_returns_sse_stream() {
    let app = setup().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let ct = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.starts_with("text/event-stream"), "got {ct}");
}

// ── Static file serving / SPA fallback ──────────────────────────

#[tokio::test]
async fn unknown_path_falls_back_to_index() {
    let app = setup().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/some/unknown/path")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // index.html is embedded at compile time from frontend/dist; the SPA
    // fallback should serve it (200) rather than 404.
    assert_eq!(response.status(), StatusCode::OK);
}
