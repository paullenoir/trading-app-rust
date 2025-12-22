use sea_orm::*;
use rust_decimal::Decimal;
use chrono::NaiveDate;
use crate::models::{trade, trades_fermes, stock};
use crate::models::dto::CreateTradeRequest;
use crate::services::wallet_service::WalletService;

pub struct TradeService;

impl TradeService {
    /// Crée un nouveau trade (achat ou vente)
    /// Pour les achats, vérifie d'abord que l'utilisateur a assez de fonds
    /// Pour les ventes, déclenche automatiquement la logique FIFO
    pub async fn create_trade(
        db: &DatabaseConnection,
        user_id: i32,
        request: CreateTradeRequest,
    ) -> Result<trade::Model, DbErr> {
        let prix_total = request.quantite * request.prix_unitaire;

        // CORRECTION CRITIQUE #3: Vérifier la balance avant un achat
        if request.trade_type == "achat" {
            // 1. Récupérer la devise du stock
            let stock_option = stock::Entity::find()
                .filter(stock::Column::SymbolAlphavantage.eq(&request.symbol))
                .one(db)
                .await?;

            let stock = stock_option.ok_or_else(|| {
                DbErr::Custom(format!("Stock not found: {}", request.symbol))
            })?;

            let currency = stock.currency.unwrap_or_else(|| "CAD".to_string());

            // 2. Vérifier si l'utilisateur a assez de trésorerie
            let has_funds = WalletService::has_sufficient_funds(
                db,
                user_id,
                &currency,
                prix_total,
            ).await?;

            if !has_funds {
                let error_msg = WalletService::get_insufficient_funds_message(
                    db,
                    user_id,
                    &currency,
                    prix_total,
                ).await?;

                return Err(DbErr::Custom(error_msg));
            }
        }

        // Initialiser quantite_restante selon le type de trade
        let quantite_restante = if request.trade_type == "achat" {
            request.quantite
        } else {
            Decimal::ZERO
        };

        let new_trade = trade::ActiveModel {
            user_id: Set(user_id),
            symbol: Set(Some(request.symbol.clone())),
            trade_type: Set(Some(request.trade_type.clone())),
            quantite: Set(Some(request.quantite)),
            prix_unitaire: Set(Some(request.prix_unitaire)),
            prix_total: Set(Some(prix_total)),
            date: Set(Some(request.date.clone())),
            quantite_restante: Set(quantite_restante),
            ..Default::default()
        };

        let trade_result = new_trade.insert(db).await?;

        // Si c'est une vente, traiter le FIFO
        if request.trade_type == "vente" {
            Self::process_sale_fifo(db, user_id, &trade_result).await?;
        }

        Ok(trade_result)
    }

    /// Traite une vente selon la méthode FIFO (First In, First Out)
    /// Ferme les trades d'achat les plus anciens en premier
    async fn process_sale_fifo(
        db: &DatabaseConnection,
        user_id: i32,
        sale_trade: &trade::Model,
    ) -> Result<(), DbErr> {
        let symbol = sale_trade.symbol.as_ref().unwrap();
        let mut remaining_quantity = sale_trade.quantite.unwrap();

        // CORRECTION CRITIQUE #2: Filtrer sur quantite_restante > 0
        let buy_trades = trade::Entity::find()
            .filter(trade::Column::UserId.eq(user_id))
            .filter(trade::Column::Symbol.eq(symbol))
            .filter(trade::Column::TradeType.eq("achat"))
            .filter(trade::Column::QuantiteRestante.gt(Decimal::ZERO))
            .order_by_asc(trade::Column::Date)
            .all(db)
            .await?;

        for buy_trade in buy_trades {
            if remaining_quantity <= Decimal::ZERO {
                break;
            }

            let available_quantity = buy_trade.quantite_restante;
            let quantity_to_close = remaining_quantity.min(available_quantity);

            Self::create_closed_trade(
                db,
                user_id,
                &buy_trade,
                sale_trade,
                quantity_to_close,
            ).await?;

            // Mettre à jour quantite_restante du trade d'achat
            let mut active_buy: trade::ActiveModel = buy_trade.into();
            active_buy.quantite_restante = Set(available_quantity - quantity_to_close);
            active_buy.update(db).await?;

            remaining_quantity -= quantity_to_close;
        }

        // Vérification: impossible de vendre plus qu'on ne possède
        if remaining_quantity > Decimal::ZERO {
            return Err(DbErr::Custom(format!(
                "Attempted to sell {} units of {} but only had enough buy positions to cover {} units. \
                 Short selling is not currently supported.",
                sale_trade.quantite.unwrap(),
                symbol,
                sale_trade.quantite.unwrap() - remaining_quantity
            )));
        }

        Ok(())
    }

    /// Crée un enregistrement de trade fermé avec calcul des gains/pertes
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

    /// Vérifie si l'utilisateur possède assez de quantité d'un symbole pour vendre
    pub async fn get_available_quantity(
        db: &DatabaseConnection,
        user_id: i32,
        symbol: &str,
    ) -> Result<Decimal, DbErr> {
        let buy_trades = trade::Entity::find()
            .filter(trade::Column::UserId.eq(user_id))
            .filter(trade::Column::Symbol.eq(symbol))
            .filter(trade::Column::TradeType.eq("achat"))
            .filter(trade::Column::QuantiteRestante.gt(Decimal::ZERO))
            .all(db)
            .await?;

        let total_available: Decimal = buy_trades
            .iter()
            .map(|t| t.quantite_restante)
            .sum();

        Ok(total_available)
    }
}