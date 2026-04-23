use axum::{
    extract::{Path, State},
    http::{header, StatusCode, Uri},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    Json,
};
use futures::stream::Stream;
use rust_embed::Embed;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::feeding;
use crate::treat;
use crate::AppState;

#[derive(Embed)]
#[folder = "frontend/dist"]
struct Assets;

// ── Response types ──────────────────────────────────────────────

#[derive(Serialize)]
pub struct FeedingResponse {
    pub id: i32,
    pub amount_half_scoops: i32,
    pub scoops: f64,
    pub fed_at: String,
    pub edited: bool,
}

#[derive(Serialize)]
pub struct TreatResponse {
    pub id: i32,
    pub name: String,
    pub given_at: String,
    pub edited: bool,
}

#[derive(Serialize)]
pub struct DayResponse {
    pub date: String,
    pub total_scoops: f64,
    pub feedings: Vec<FeedingResponse>,
    pub treats: Vec<TreatResponse>,
}

#[derive(Serialize)]
pub struct CalendarDay {
    pub date: String,
    pub total_scoops: f64,
    pub treat_count: usize,
}

#[derive(Deserialize)]
pub struct AddFeedingRequest {
    pub amount_half_scoops: i32,
    pub fed_at: String,
    #[serde(default)]
    pub edited: bool,
}

#[derive(Deserialize)]
pub struct AddTreatRequest {
    pub name: String,
    pub given_at: String,
    #[serde(default)]
    pub edited: bool,
}

// ── Helpers ─────────────────────────────────────────────────────

/// Validate a "YYYY-MM-DD HH:MM:SS" timestamp string.
fn validate_timestamp(ts: &str) -> Result<(), StatusCode> {
    chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(())
}

fn to_response(m: feeding::Model) -> FeedingResponse {
    FeedingResponse {
        id: m.id,
        amount_half_scoops: m.amount_half_scoops,
        scoops: m.amount_half_scoops as f64 / 2.0,
        fed_at: m.fed_at,
        edited: m.edited,
    }
}

fn to_treat_response(m: treat::Model) -> TreatResponse {
    TreatResponse {
        id: m.id,
        name: m.name,
        given_at: m.given_at,
        edited: m.edited,
    }
}

fn day_range(date: &str) -> (String, String) {
    let start = format!("{date} 00:00:00");
    let next_day = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .unwrap()
        .succ_opt()
        .unwrap()
        .format("%Y-%m-%d")
        .to_string();
    let end = format!("{next_day} 00:00:00");
    (start, end)
}

async fn feedings_for_date(
    db: &DatabaseConnection,
    date: &str,
) -> Result<Vec<feeding::Model>, DbErr> {
    let (start, end) = day_range(date);
    feeding::Entity::find()
        .filter(feeding::Column::FedAt.gte(&start))
        .filter(feeding::Column::FedAt.lt(&end))
        .order_by_asc(feeding::Column::FedAt)
        .all(db)
        .await
}

async fn treats_for_date(
    db: &DatabaseConnection,
    date: &str,
) -> Result<Vec<treat::Model>, DbErr> {
    let (start, end) = day_range(date);
    treat::Entity::find()
        .filter(treat::Column::GivenAt.gte(&start))
        .filter(treat::Column::GivenAt.lt(&end))
        .order_by_asc(treat::Column::GivenAt)
        .all(db)
        .await
}

fn build_day_response(
    date: &str,
    models: Vec<feeding::Model>,
    treat_models: Vec<treat::Model>,
) -> DayResponse {
    let total: i32 = models.iter().map(|f| f.amount_half_scoops).sum();
    DayResponse {
        date: date.to_string(),
        total_scoops: total as f64 / 2.0,
        feedings: models.into_iter().map(to_response).collect(),
        treats: treat_models.into_iter().map(to_treat_response).collect(),
    }
}

fn notify(state: &AppState) {
    let _ = state.tx.send(());
}

// ── Handlers ────────────────────────────────────────────────────

pub async fn get_day(
    State(state): State<AppState>,
    Path(date): Path<String>,
) -> Result<Json<DayResponse>, StatusCode> {
    chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let models = feedings_for_date(&state.db, &date)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let treat_models = treats_for_date(&state.db, &date)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(build_day_response(&date, models, treat_models)))
}

pub async fn add_treat(
    State(state): State<AppState>,
    Json(req): Json<AddTreatRequest>,
) -> Result<Json<TreatResponse>, StatusCode> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    validate_timestamp(&req.given_at)?;

    let model = treat::ActiveModel {
        name: Set(name),
        given_at: Set(req.given_at),
        edited: Set(req.edited),
        ..Default::default()
    };

    let result = model
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    notify(&state);
    Ok(Json(to_treat_response(result)))
}

pub async fn delete_treat(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let result = treat::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        notify(&state);
        Ok(StatusCode::NO_CONTENT)
    }
}

pub async fn add_feeding(
    State(state): State<AppState>,
    Json(req): Json<AddFeedingRequest>,
) -> Result<Json<FeedingResponse>, StatusCode> {
    if req.amount_half_scoops != 1 && req.amount_half_scoops != 2 {
        return Err(StatusCode::BAD_REQUEST);
    }
    validate_timestamp(&req.fed_at)?;

    let model = feeding::ActiveModel {
        amount_half_scoops: Set(req.amount_half_scoops),
        fed_at: Set(req.fed_at),
        edited: Set(req.edited),
        ..Default::default()
    };

    let result = model
        .insert(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    notify(&state);
    Ok(Json(to_response(result)))
}

pub async fn delete_feeding(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let result = feeding::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        notify(&state);
        Ok(StatusCode::NO_CONTENT)
    }
}

pub async fn get_calendar(
    State(state): State<AppState>,
    Path((year, month)): Path<(i32, u32)>,
) -> Result<Json<Vec<CalendarDay>>, StatusCode> {
    let start_date =
        chrono::NaiveDate::from_ymd_opt(year, month, 1).ok_or(StatusCode::BAD_REQUEST)?;
    let end_date = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or(StatusCode::BAD_REQUEST)?;

    let start = format!("{} 00:00:00", start_date);
    let end = format!("{} 00:00:00", end_date);

    let feedings = feeding::Entity::find()
        .filter(feeding::Column::FedAt.gte(&start))
        .filter(feeding::Column::FedAt.lt(&end))
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let treats = treat::Entity::find()
        .filter(treat::Column::GivenAt.gte(&start))
        .filter(treat::Column::GivenAt.lt(&end))
        .all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut daily: std::collections::HashMap<String, (i32, usize)> =
        std::collections::HashMap::new();
    for f in feedings {
        let date = f.fed_at.split(' ').next().unwrap_or("").to_string();
        daily.entry(date).or_insert((0, 0)).0 += f.amount_half_scoops;
    }
    for t in treats {
        let date = t.given_at.split(' ').next().unwrap_or("").to_string();
        daily.entry(date).or_insert((0, 0)).1 += 1;
    }

    let mut result: Vec<CalendarDay> = daily
        .into_iter()
        .map(|(date, (scoops, treat_count))| CalendarDay {
            date,
            total_scoops: scoops as f64 / 2.0,
            treat_count,
        })
        .collect();
    result.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(Json(result))
}

// ── SSE ─────────────────────────────────────────────────────────

pub async fn events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    // On Lagged or any tick, emit a single "invalidate" event.
    let stream = BroadcastStream::new(rx).map(|_| Ok(Event::default().data("invalidate")));

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(25))
            .text("keep-alive"),
    )
}

// ── Version ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct VersionResponse {
    pub version: &'static str,
    pub build_id: &'static str,
}

pub async fn get_version() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION"),
        build_id: env!("BUILD_ID"),
    })
}

// ── Static file serving ─────────────────────────────────────────

/// Pick a Cache-Control value for a given asset path.
///
/// Vite emits hashed filenames into `assets/`, which are safe to cache
/// aggressively. Everything else (notably `index.html`) must always be
/// revalidated so home-screen PWAs pick up new builds without a manual
/// quit-and-relaunch.
fn cache_control_for(path: &str) -> &'static str {
    if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    }
}

pub async fn serve_frontend(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, mime.as_ref()),
                (header::CACHE_CONTROL, cache_control_for(path)),
            ],
            file.data.to_vec(),
        )
            .into_response();
    }

    if let Some(file) = Assets::get("index.html") {
        return (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/html"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            file.data.to_vec(),
        )
            .into_response();
    }

    StatusCode::NOT_FOUND.into_response()
}
