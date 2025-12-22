use sea_orm::*;
use rust_decimal::Decimal;
use chrono::{NaiveDate, Utc};
use crate::models::{trade, trades_fermes};
use crate::models::dto::CreateTradeRequest;

pub struct TradeService;

impl TradeService {
    pub async fn create_trade(
        db: &DatabaseConnection,
        user_id: i32,
        request: CreateTradeRequest,
    ) -> Result<trade::Model, DbErr> {
        let prix_total = request.quantite * request.prix_unitaire;

        let new_trade = trade::ActiveModel {
            user_id: Set(user_id),
            symbol: Set(Some(request.symbol.clone())),
            trade_type: Set(Some(request.trade_type.clone())),
            quantite: Set(Some(request.quantite)),
            prix_unitaire: Set(Some(request.prix_unitaire)),
            prix_total: Set(Some(prix_total)),
            date: Set(Some(request.date.clone())),
            ..Default::default()
        };

        let trade_result = new_trade.insert(db).await?;

        if request.trade_type == "vente" {
            Self::process_sale_fifo(db, user_id, &trade_result).await?;
        }

        Ok(trade_result)
    }

    async fn process_sale_fifo(
        db: &DatabaseConnection,
        user_id: i32,
        sale_trade: &trade::Model,
    ) -> Result<(), DbErr> {
        let symbol = sale_trade.symbol.as_ref().unwrap();
        let mut remaining_quantity = sale_trade.quantite.unwrap();

        let buy_trades = trade::Entity::find()
            .filter(trade::Column::UserId.eq(user_id))
            .filter(trade::Column::Symbol.eq(symbol))
            .filter(trade::Column::TradeType.eq("achat"))
            .order_by_asc(trade::Column::Date)
            .all(db)
            .await?;

        for buy_trade in buy_trades {
            if remaining_quantity <= Decimal::ZERO {
                break;
            }

            let buy_quantity = buy_trade.quantite.unwrap();
            let quantity_to_close = remaining_quantity.min(buy_quantity);

            Self::create_closed_trade(
                db,
                user_id,
                &buy_trade,
                sale_trade,
                quantity_to_close,
            ).await?;

            remaining_quantity -= quantity_to_close;
        }

        Ok(())
    }

    async fn create_closed_trade(
        db: &DatabaseConnection,
        user_id: i32,
        buy_trade: &trade::Model,
        sale_trade: &trade::Model,
        quantity: Decimal,
    ) -> Result<(), DbErr> {
        let buy_price = buy_trade.prix_unitaire.unwrap();
        let sale_price = sale_trade.prix_unitaire.unwrap();
        let gain = (sale_price - buy_price) * quantity;
        let pourcentage = ((sale_price - buy_price) / buy_price * Decimal::from(100)).round();

        let date_achat = NaiveDate::parse_from_str(&buy_trade.date.as_ref().unwrap(), "%Y-%m-%d").ok();
        let date_vente = NaiveDate::parse_from_str(&sale_trade.date.as_ref().unwrap(), "%Y-%m-%d").ok();

        let temps_jours = if let (Some(achat), Some(vente)) = (date_achat, date_vente) {
            (vente - achat).num_days() as i32
        } else {
            0
        };

        let unique_id = format!("{}_{}_{}_{}",
                                user_id,
                                buy_trade.id,
                                sale_trade.id,
                                chrono::Utc::now().timestamp_millis()
        );

        let closed_trade = trades_fermes::ActiveModel {
            id: Set(unique_id),
            user_id: Set(user_id),
            symbol: Set(Some(buy_trade.symbol.clone().unwrap())),
            date_achat: Set(Some(buy_trade.date.clone().unwrap())),
            prix_achat: Set(Some(buy_price.to_string())),
            date_vente: Set(Some(sale_trade.date.clone().unwrap())),
            prix_vente: Set(Some(sale_price.to_string())),
            pourcentage_gain: Set(Some(pourcentage.to_string().parse().unwrap_or(0))),
            gain_dollars: Set(Some(gain)),
            temps_jours: Set(Some(temps_jours)),
            trade_achat_id: Set(Some(buy_trade.id)),
            trade_vente_id: Set(Some(sale_trade.id)),
        };

        closed_trade.insert(db).await?;
        Ok(())
    }
}