use std::sync::Arc;
use tokio_postgres::Client;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tokio::sync::Mutex;

const STATION_ID: i32 = 1;

const ALLOWED_RECORDING_COLUMNS: &[&str] = &[
    "co2",
    "pressure",
    "humidity",
    "air_temperature",
    "water_temperature",
    "ground_temperature",
    "wind_speed",
    "net_irradiance",
    "rate_of_evaporation",
];

#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<Client>,
    pub most_recent_image: Arc<Mutex<Vec<f32>>>,
}

impl ApiState {
    pub fn new(db: Arc<Client>, most_recent_image: Arc<Mutex<Vec<f32>>>) -> Self {
        Self {
            db,
            most_recent_image,
        }
    }
}

pub fn routes() -> Router<ApiState> {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/station", get(get_station).post(update_station))
        .route(
            "/station/bitmap",
            get(get_station_bitmap).post(update_station_bitmap),
        )
        .route("/recordings", get(get_recordings))
        .route("/most_recent_image", get(get_most_recent_image))
}

async fn get_most_recent_image(State(state): State<ApiState>) -> Json<Vec<f32>> {
    let image = state.most_recent_image.lock().await;
    Json(image.clone())
}

#[derive(Serialize)]
struct StationResponse {
    id: i32,
    name: String,
    water_depth: f32,
    last_updated: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
struct UpdateStationRequest {
    name: Option<String>,
    water_depth: Option<f32>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Deserialize)]
struct SurfaceQuery {
    surface: String,
}

#[derive(Serialize)]
struct BitmapResponse {
    surface: String,
    bitmap: String,
}

#[derive(Deserialize)]
struct UpdateBitmapRequest {
    bitmap: String,
}

#[derive(Deserialize, Debug)]
struct RecordingsQuery {
    start: String,
    end: String,
    preferred_rows: usize,
    columns: String,
}

#[derive(Serialize)]
struct RecordingsResponse {
    total_in_range: i64,
    returned_rows: usize,
    rows: Vec<Map<String, Value>>,
}

fn into_http_error(message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": message.into() })),
    )
}

fn bitvec_to_bitmap_string(bits: &bit_vec::BitVec) -> String {
    let mut out = String::with_capacity(bits.len());
    for bit in bits {
        out.push(if bit { '1' } else { '0' });
    }
    out
}

fn validate_bitmap(bitmap: &str) -> Result<(), String> {
    if bitmap.len() != 768 {
        return Err("bitmap must be exactly 768 bits long".to_owned());
    }

    if bitmap.chars().any(|c| c != '0' && c != '1') {
        return Err("bitmap must contain only '0' and '1'".to_owned());
    }

    Ok(())
}

fn parse_requested_columns(columns: &str) -> Result<Vec<String>, String> {
    let parsed: Vec<String> = columns
        .split(',')
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    if parsed.is_empty() {
        return Err("at least one recording column must be requested".to_owned());
    }

    for col in &parsed {
        if !ALLOWED_RECORDING_COLUMNS.contains(&col.as_str()) {
            return Err(format!("unsupported recording column: {col}"));
        }
    }

    Ok(parsed)
}

async fn get_station(
    State(state): State<ApiState>,
) -> Result<Json<StationResponse>, (StatusCode, Json<Value>)> {
    let row = state
        .db
        .query_one(
            "
            SELECT
                id,
                name,
                water_depth,
                last_updated::text AS last_updated,
                ST_Y(location::geometry) AS latitude,
                ST_X(location::geometry) AS longitude
            FROM stations
            WHERE id = $1
            ",
            &[&STATION_ID],
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(StationResponse {
        id: row.get(0),
        name: row.get(1),
        water_depth: row.get(2),
        last_updated: row.get(3),
        latitude: row.get(4),
        longitude: row.get(5),
    }))
}

async fn update_station(
    State(state): State<ApiState>,
    Json(payload): Json<UpdateStationRequest>,
) -> Result<Json<StationResponse>, (StatusCode, Json<Value>)> {
    if payload.latitude.is_some() ^ payload.longitude.is_some() {
        return Err(into_http_error(
            "latitude and longitude must both be supplied when updating location",
        ));
    }

    let row = state
        .db
        .query_one(
            "
            UPDATE stations
            SET
                name = COALESCE($1, name),
                water_depth = COALESCE($2, water_depth),
                location = COALESCE(
                    CASE
                        WHEN $3::double precision IS NOT NULL AND $4::double precision IS NOT NULL
                        THEN ST_SetSRID(ST_MakePoint($4::double precision, $3::double precision), 4326)::geography
                        ELSE NULL
                    END,
                    location
                ),
                last_updated = now()
            WHERE id = $5
            RETURNING
                id,
                name,
                water_depth,
                last_updated::text AS last_updated,
                ST_Y(location::geometry) AS latitude,
                ST_X(location::geometry) AS longitude
            ",
            &[
                &payload.name,
                &payload.water_depth,
                &payload.latitude,
                &payload.longitude,
                &STATION_ID,
            ],
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        })?;

    Ok(Json(StationResponse {
        id: row.get(0),
        name: row.get(1),
        water_depth: row.get(2),
        last_updated: row.get(3),
        latitude: row.get(4),
        longitude: row.get(5),
    }))
}

async fn get_station_bitmap(
    State(state): State<ApiState>,
    Query(query): Query<SurfaceQuery>,
) -> Result<Json<BitmapResponse>, (StatusCode, Json<Value>)> {
    let (sql, surface) = match query.surface.as_str() {
        "land" => (
            "SELECT land_bitmap FROM stations WHERE id = $1",
            "land".to_owned(),
        ),
        "water" => (
            "SELECT water_bitmap FROM stations WHERE id = $1",
            "water".to_owned(),
        ),
        _ => {
            return Err(into_http_error("surface must be either 'land' or 'water'"));
        }
    };

    let row = state.db.query_one(sql, &[&STATION_ID]).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    let bitmap: bit_vec::BitVec = row.get(0);

    Ok(Json(BitmapResponse {
        surface,
        bitmap: bitvec_to_bitmap_string(&bitmap),
    }))
}

async fn update_station_bitmap(
    State(state): State<ApiState>,
    Query(query): Query<SurfaceQuery>,
    Json(payload): Json<UpdateBitmapRequest>,
) -> Result<Json<BitmapResponse>, (StatusCode, Json<Value>)> {
    validate_bitmap(&payload.bitmap).map_err(into_http_error)?;

    let (sql, surface) = match query.surface.as_str() {
        "land" => (
            "
            UPDATE stations
            SET
                land_bitmap = $1::bit(768),
                last_updated = now()
            WHERE id = $2
            RETURNING land_bitmap
            ",
            "land".to_owned(),
        ),
        "water" => (
            "
            UPDATE stations
            SET
                water_bitmap = $1::bit(768),
                last_updated = now()
            WHERE id = $2
            RETURNING water_bitmap
            ",
            "water".to_owned(),
        ),
        _ => {
            return Err(into_http_error("surface must be either 'land' or 'water'"));
        }
    };

    let row = state
        .db
        .query_one(sql, &[&payload.bitmap, &STATION_ID])
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        })?;

    let bitmap: bit_vec::BitVec = row.get(0);

    Ok(Json(BitmapResponse {
        surface,
        bitmap: bitvec_to_bitmap_string(&bitmap),
    }))
}

async fn get_recordings(
    State(state): State<ApiState>,
    Query(query): Query<RecordingsQuery>,
) -> Result<Json<RecordingsResponse>, (StatusCode, Json<Value>)> {
    if query.preferred_rows == 0 {
        return Err(into_http_error("preferred_rows must be greater than 0"));
    }

    let requested_columns = parse_requested_columns(&query.columns).map_err(into_http_error)?;

    use chrono::{DateTime, NaiveDateTime, Utc};
    let start_dt = DateTime::parse_from_rfc3339(&query.start)
        .map_err(|e| into_http_error(format!("invalid start timestamp: {e}")))?
        .with_timezone(&Utc)
        .naive_utc();
    let end_dt = DateTime::parse_from_rfc3339(&query.end)
        .map_err(|e| into_http_error(format!("invalid end timestamp: {e}")))?
        .with_timezone(&Utc)
        .naive_utc();

    let count_row = state
        .db
        .query_one(
            "
            SELECT COUNT(*)
            FROM recordings
            WHERE station_id = $1
              AND timestamp >= $2::timestamp
              AND timestamp <= $3::timestamp
            ",
            &[&STATION_ID, &start_dt, &end_dt],
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error finding recordings count": e.to_string() })),
            )
        })?;

    let total_in_range: i64 = count_row.get(0);

    if total_in_range == 0 {
        return Ok(Json(RecordingsResponse {
            total_in_range,
            returned_rows: 0,
            rows: vec![],
        }));
    }

    let should_aggregate = (query.preferred_rows as i64) < total_in_range;

    let select_cols = requested_columns
        .iter()
        .map(|c| format!("{c}::double precision AS {c}"))
        .collect::<Vec<_>>()
        .join(", ");

    let rows = if should_aggregate {
        let avg_cols = requested_columns
            .iter()
            .map(|c| format!("AVG({c})::double precision AS {c}"))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "
            WITH grouped AS (
                SELECT
                    timestamp,
                    {select_cols},
                    NTILE($4) OVER (ORDER BY timestamp) AS bucket
                FROM recordings
                WHERE station_id = $1
                  AND timestamp >= $2::timestamp
                  AND timestamp <= $3::timestamp
            )
            SELECT
                MIN(timestamp)::text AS timestamp,
                {avg_cols}
            FROM grouped
            GROUP BY bucket
            ORDER BY bucket
            "
        );

        state
            .db
            .query(
                &sql,
                &[
                    &STATION_ID,
                    &start_dt,
                    &end_dt,
                    &(query.preferred_rows as i64),
                ],
            )
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error grouping and aggregating data": e.to_string() })),
                )
            })?
    } else {
        let sql = format!(
            "
            SELECT
                timestamp::text AS timestamp,
                {select_cols}
            FROM recordings
            WHERE station_id = $1
              AND timestamp >= $2::timestamp
              AND timestamp <= $3::timestamp
            ORDER BY timestamp
            "
        );

        state
            .db
            .query(&sql, &[&STATION_ID, &start_dt, &end_dt])
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error gathering all recordings": e.to_string() })),
                )
            })?
    };

    let mut json_rows = Vec::with_capacity(rows.len());
    for row in rows {
        let mut obj = Map::new();
        let timestamp: String = row.get(0);
        obj.insert("timestamp".to_owned(), Value::String(timestamp));

        for (i, col_name) in requested_columns.iter().enumerate() {
            let value: f64 = row.get(i + 1);
            obj.insert(col_name.clone(), Value::from(value));
        }

        json_rows.push(obj);
    }

    Ok(Json(RecordingsResponse {
        total_in_range,
        returned_rows: json_rows.len(),
        rows: json_rows,
    }))
}
