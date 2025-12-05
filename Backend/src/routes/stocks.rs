use actix_web::{get, web, HttpResponse};
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::models::stock::Entity as Stock;

#[get("")]
pub async fn get_stocks(db_connection: web::Data<DatabaseConnection>) -> HttpResponse {
    let stocks = Stock::find()
        .all(db_connection.get_ref())
        .await;

    match stocks {
        Ok(stocks) => HttpResponse::Ok().json(stocks),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub fn stocks_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/stocks")
            .service(get_stocks)
        // Plus tard : .service(get_stock_by_id)
        // Plus tard : .service(create_stock)
    );
}