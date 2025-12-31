use actix_web::{web, HttpResponse, Responder, get};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder, QuerySelect};
use validator::Validate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use crate::middleware::AuthUser;
use crate::models::dto::{CreateTradeRequest, TradeResponse, OpenPositionResponse, ClosedTradeResponse, OpenPositionWithRecommendationsResponse, StrategyWithResult};
use crate::models::{trade, strategy, strategy_result};
use crate::services::trade_service::TradeService;
use rust_decimal::prelude::ToPrimitive;

pub async fn create_trade(
    db: web::Data<DatabaseConnection>,
    auth_user: AuthUser,
    request: web::Json<CreateTradeRequest>,
) -> impl Responder {
    if let Err(errors) = request.validate() {
        return HttpResponse::BadRequest().json(errors);
    }

    match TradeService::create_trade(&db, auth_user.user_id, request.into_inner()).await {
        Ok(trade_model) => {
            let response = TradeResponse {
                id: trade_model.id,
                user_id: trade_model.user_id,
                symbol: trade_model.symbol.unwrap_or_default(),
                trade_type: trade_model.trade_type.unwrap_or_default(),
                quantite: trade_model.quantite.unwrap_or_default(),
                prix_unitaire: trade_model.prix_unitaire.unwrap_or_default(),
                prix_total: trade_model.prix_total.unwrap_or_default(),
                date: trade_model.date.unwrap_or_default(),
            };
            HttpResponse::Created().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

#[get("")]
pub async fn get_all_trades(
    db: web::Data<DatabaseConnection>,
    auth_user: AuthUser,
) -> impl Responder {
    let trades = trade::Entity::find()
        .filter(trade::Column::UserId.eq(auth_user.user_id))
        .order_by_desc(trade::Column::Date)
        .order_by_desc(trade::Column::Id)
        .all(db.get_ref())
        .await;

    match trades {
        Ok(trades) => {
            let response: Vec<TradeResponse> = trades
                .into_iter()
                .map(|t| TradeResponse {
                    id: t.id,
                    user_id: t.user_id,
                    symbol: t.symbol.unwrap_or_default(),
                    trade_type: t.trade_type.unwrap_or_default(),
                    quantite: t.quantite.unwrap_or_default(),
                    prix_unitaire: t.prix_unitaire.unwrap_or_default(),
                    prix_total: t.prix_total.unwrap_or_default(),
                    date: t.date.unwrap_or_default(),
                })
                .collect();
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

#[get("/open")]
pub async fn get_open_positions(
    db: web::Data<DatabaseConnection>,
    auth_user: AuthUser,
) -> impl Responder {
    let trades = trade::Entity::find()
        .filter(trade::Column::UserId.eq(auth_user.user_id))
        .order_by_asc(trade::Column::Date)
        .all(db.get_ref())
        .await;

    match trades {
        Ok(trades) => {
            let mut positions: HashMap<String, (Decimal, Decimal)> = HashMap::new();

            for t in trades {
                let symbol = t.symbol.unwrap_or_default();
                let quantite = t.quantite.unwrap_or_default();
                let prix_unitaire = t.prix_unitaire.unwrap_or_default();
                let trade_type = t.trade_type.unwrap_or_default();

                let entry = positions.entry(symbol.clone()).or_insert((Decimal::ZERO, Decimal::ZERO));

                if trade_type == "achat" {
                    let total_cost = entry.0 * entry.1;
                    let new_cost = quantite * prix_unitaire;
                    entry.0 += quantite;
                    entry.1 = if entry.0 > Decimal::ZERO {
                        (total_cost + new_cost) / entry.0
                    } else {
                        Decimal::ZERO
                    };
                } else if trade_type == "vente" {
                    entry.0 -= quantite;
                }
            }

            let response: Vec<OpenPositionResponse> = positions
                .into_iter()
                .filter(|(_, (qty, _))| *qty > Decimal::ZERO)
                .map(|(symbol, (quantite_totale, prix_moyen))| OpenPositionResponse {
                    symbol,
                    quantite_totale,
                    prix_moyen,
                })
                .collect();

            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

#[get("/open-with-recommendations")]
pub async fn get_open_positions_with_recommendations(
    db: web::Data<DatabaseConnection>,
    auth_user: AuthUser,
) -> impl Responder {
    use chrono::NaiveDate;
    use crate::models::historic_data;
    use rust_decimal::prelude::ToPrimitive;

    // Récupérer tous les trades de l'utilisateur
    let trades = trade::Entity::find()
        .filter(trade::Column::UserId.eq(auth_user.user_id))
        .order_by_asc(trade::Column::Date)
        .all(db.get_ref())
        .await;

    let trades = match trades {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().json(format!("Error fetching trades: {}", e));
        }
    };

    // Calculer les positions ouvertes (FIFO) avec date d'entrée
    let mut positions: HashMap<String, (Decimal, Decimal, NaiveDate)> = HashMap::new();

    for t in &trades {
        let symbol = t.symbol.clone().unwrap_or_default();
        let quantite = t.quantite.unwrap_or_default();
        let prix_unitaire = t.prix_unitaire.unwrap_or_default();
        let trade_type = t.trade_type.clone().unwrap_or_default();

        // Parser la date String en NaiveDate (format DD/MM/YYYY)
        let date = match t.date.clone().unwrap_or_default().as_str() {
            date_str => {
                match NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
                    Ok(d) => d,
                    Err(_) => continue,
                }
            }
        };

        let entry = positions
            .entry(symbol.clone())
            .or_insert((Decimal::ZERO, Decimal::ZERO, date));

        if trade_type == "achat" {
            let total_cost = entry.0 * entry.1;
            let new_cost = quantite * prix_unitaire;
            entry.0 += quantite;
            entry.1 = if entry.0 > Decimal::ZERO {
                (total_cost + new_cost) / entry.0
            } else {
                Decimal::ZERO
            };
            if entry.2 > date {
                entry.2 = date;
            }
        } else if trade_type == "vente" {
            entry.0 -= quantite;
        }
    }

    // Pour chaque position ouverte, récupérer les recommandations + P&L
    let mut response: Vec<OpenPositionWithRecommendationsResponse> = Vec::new();

    for (symbol, (quantite_totale, prix_moyen, entry_date)) in positions {
        // Ignorer les positions fermées
        if quantite_totale <= Decimal::ZERO {
            continue;
        }

        // Récupérer le prix actuel depuis historic_data_rust (dernière clôture)
        let latest_price = historic_data::Entity::find()
            .filter(historic_data::Column::Symbol.eq(&symbol))
            .order_by_desc(historic_data::Column::Date)
            .limit(1)
            .one(db.get_ref())
            .await;

        let current_price = match latest_price {
            Ok(Some(data)) => {
                data.close
                    .and_then(|close_str| close_str.parse::<f64>().ok())
                    .and_then(|p| Decimal::from_f64_retain(p))
                    .unwrap_or(prix_moyen)
            }
            Ok(None) => prix_moyen,
            Err(_) => prix_moyen,
        };

        // Calcul du P&L
        let pnl_dollars = (current_price - prix_moyen) * quantite_totale;
        let pnl_percentage = if prix_moyen > Decimal::ZERO {
            ((current_price - prix_moyen) / prix_moyen * Decimal::from(100))
                .to_f64()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Récupérer les stratégies
        let all_strategies = strategy::Entity::find()
            .all(db.get_ref())
            .await;

        let strategies = match all_strategies {
            Ok(strats) => {
                let mut strategy_list = Vec::new();

                for strat in strats {
                    let all_results = strategy_result::Entity::find()
                        .filter(strategy_result::Column::StrategyId.eq(strat.id))
                        .filter(strategy_result::Column::Symbol.eq(&symbol))
                        .order_by_desc(strategy_result::Column::Date)
                        .all(db.get_ref())
                        .await;

                    if let Ok(results) = all_results {
                        if !results.is_empty() {
                            let sr = &results[0];

                            let recommendation_str = sr.recommendation.clone().and_then(|v| {
                                if let Some(s) = v.as_str() {
                                    return Some(s.to_string());
                                }
                                if let Some(arr) = v.as_array() {
                                    let items: Vec<String> = arr
                                        .iter()
                                        .filter_map(|item| {
                                            if let Some(s) = item.as_str() {
                                                Some(s.to_string())
                                            } else {
                                                Some(item.to_string())
                                            }
                                        })
                                        .collect();
                                    return Some(format!("[{}]", items.join(", ")));
                                }
                                Some(v.to_string())
                            });

                            strategy_list.push(StrategyWithResult {
                                strategy_id: strat.id,
                                strategy_name: strat.name.clone(),
                                date: sr.date.clone(),
                                recommendation: recommendation_str,
                            });
                        }
                    }
                }

                strategy_list
            }
            Err(_) => vec![],
        };

        // Arrondir à 2 décimales
        let prix_moyen_rounded = prix_moyen.round_dp(2);
        let current_price_rounded = current_price.round_dp(2);
        let pnl_dollars_rounded = pnl_dollars.round_dp(2);
        let pnl_percentage_rounded = (pnl_percentage * 100.0).round() / 100.0;

        response.push(OpenPositionWithRecommendationsResponse {
            symbol,
            quantite_totale,
            prix_moyen: prix_moyen_rounded,
            current_price: Some(current_price_rounded),
            pnl_dollars: Some(pnl_dollars_rounded),
            pnl_percentage: Some(pnl_percentage_rounded),
            entry_date: Some(entry_date.to_string()),
            strategies,
        });
    }

    HttpResponse::Ok().json(response)
}

#[get("/closed")]
pub async fn get_closed_trades(
    db: web::Data<DatabaseConnection>,
    auth_user: AuthUser,
) -> impl Responder {
    use crate::models::trades_fermes;

    let closed_trades = trades_fermes::Entity::find()
        .filter(trades_fermes::Column::UserId.eq(auth_user.user_id))
        .order_by_desc(trades_fermes::Column::DateVente)
        .all(db.get_ref())
        .await;

    match closed_trades {
        Ok(trades) => {
            let response: Vec<ClosedTradeResponse> = trades
                .into_iter()
                .map(|t| ClosedTradeResponse {
                    symbol: t.symbol.unwrap_or_default(),
                    date_achat: t.date_achat.unwrap_or_default(),
                    prix_achat: t.prix_achat.unwrap_or_default(),
                    date_vente: t.date_vente.unwrap_or_default(),
                    prix_vente: t.prix_vente.unwrap_or_default(),
                    pourcentage_gain: t.pourcentage_gain.unwrap_or(0),
                    gain_dollars: t.gain_dollars.unwrap_or_default(),
                    temps_jours: t.temps_jours.unwrap_or(0),
                    trade_achat_id: t.trade_achat_id.unwrap_or(0),
                    trade_vente_id: t.trade_vente_id.unwrap_or(0),
                })
                .collect();
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/trades")
            .route("", web::post().to(create_trade))
            .service(get_all_trades)
            .service(get_open_positions)
            .service(get_open_positions_with_recommendations)
            .service(get_closed_trades)
    );
}