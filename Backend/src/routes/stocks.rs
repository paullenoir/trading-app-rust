use actix_web::{get, web, HttpResponse};
use crate::models::{
    stock::Entity as Stock,
    strategy_result::{self, Entity as StrategyResult},
    strategy::{self, Entity as Strategy},
    dto::{StockWithStrategies, StockInfo, StrategyWithResult},
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use std::collections::{HashSet, HashMap};

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

#[get("/with-strategies")]
pub async fn get_stocks_with_strategies(db: web::Data<DatabaseConnection>) -> HttpResponse {
    // 1. Trouver la date la plus récente
    let latest_date = StrategyResult::find()
        .order_by_desc(strategy_result::Column::Date)
        .one(db.get_ref())
        .await
        .ok()
        .flatten()
        .and_then(|r| r.date);

    if latest_date.is_none() {
        return HttpResponse::Ok().json(Vec::<StockWithStrategies>::new());
    }

    let latest_date = latest_date.unwrap();

    // 2. Récupérer stocks avec résultats filtrés sur cette date
    let stocks_with_results = Stock::find()
        .find_with_related(StrategyResult)
        .filter(strategy_result::Column::Date.eq(latest_date))
        .all(db.get_ref())
        .await;

    match stocks_with_results {
        Ok(stocks_with_results) => {
            // 3. Extraire tous les strategy_ids uniques
            let strategy_ids: Vec<i32> = stocks_with_results
                .iter()
                .flat_map(|(_, results)| results.iter().map(|r| r.strategy_id))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            // 4. Récupérer TOUTES les stratégies en UNE SEULE query
            let strategies_list = Strategy::find()
                .filter(strategy::Column::Id.is_in(strategy_ids))
                .all(db.get_ref())
                .await
                .unwrap_or_default();

            // 5. Créer un HashMap pour lookup O(1) au lieu de N queries
            let strategies_map: HashMap<i32, String> = strategies_list
                .into_iter()
                .filter_map(|s| s.name.map(|name| (s.id, name)))
                .collect();

            // 6. Construire la réponse finale
            let response: Vec<StockWithStrategies> = stocks_with_results
                .into_iter()
                .map(|(stock, strategy_results)| {
                    let strategies = strategy_results
                        .into_iter()
                        .map(|result| StrategyWithResult {
                            strategy_id: result.strategy_id,
                            strategy_name: strategies_map.get(&result.strategy_id).cloned(),
                            date: result.date,
                            recommendation: result.recommendation,
                        })
                        .collect();

                    StockWithStrategies {
                        stock: StockInfo {
                            company_name: stock.compagny_name,
                            symbol_alphavantage: stock.symbol_alphavantage,
                            currency: stock.currency,
                        },
                        strategies,
                    }
                })
                .collect();

            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}


pub fn stocks_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/stocks")
            .service(get_stocks)
            . service(get_stocks_with_strategies)
    );
}