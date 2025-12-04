use actix_web::{get, HttpResponse};
use chrono::Utc;
use crate::models::health::HealthResponse;

#[get("api/health")]
pub async fn health_check() -> HttpResponse {
    let response = HealthResponse {
        status: "ok".to_string(),
        time: Utc::now(),
    };

    HttpResponse::Ok().json(response)
}