use actix_web::{post, web, HttpResponse};
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::services::strategy_service::StrategyService;
use crate::models::stock::Entity as Stock;

#[post("/calculate")]
pub async fn calculate_strategies(
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    // 1. Récupérer tous les symboles depuis la table stock
    let stocks = match Stock::find().all(db.get_ref()).await {
        Ok(stocks) => stocks,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch stocks: {}", e)
            }));
        }
    };

    // 2. Extraire les symboles (symbol_alphavantage)
    let symbols: Vec<String> = stocks
        .into_iter()
        .filter_map(|stock| stock.symbol_alphavantage)
        .collect();

    if symbols.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": "No symbols found in database"
        }));
    }

    // ⚠️ VERSION TEST : Un seul symbole hardcodé
    //let symbols = vec!["AAPL.TO".to_string()];

    // 3. Exécuter les stratégies
    let service = StrategyService::new();

    match service.execute_default_strategies(symbols.clone(), db.get_ref()).await {
        Ok(results) => {
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("Calculated strategies for {} symbols", symbols.len()),
                "total_results": results.len(),
                "symbols_processed": symbols
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e
            }))
        }
    }
}

pub fn admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/strategies")
            .service(calculate_strategies)
    );
}