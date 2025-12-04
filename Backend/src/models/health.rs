use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub time: DateTime<Utc>,
}