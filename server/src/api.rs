use std::sync::Arc;
use tokio_postgres::Client;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<Client>,
}

impl ApiState {
    pub fn new(db: Arc<Client>) -> Self {
        Self { db }
    }
}

pub fn routes() -> Router<ApiState> {
    Router::new().route("/health", get(|| async { "OK" }))
}
