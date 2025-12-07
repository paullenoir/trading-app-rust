pub mod health;
pub mod stocks;
pub mod admin;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(health::health_check)
            .configure(stocks::stocks_routes)
            .configure(admin::admin_routes)
    );
}