use sea_orm::*;
use rust_decimal::Decimal;
use std::collections::HashMap;
use crate::models::{wallet, trade, stock};

pub struct WalletService;

/// Représente la balance pour une devise spécifique
#[derive(Debug, Clone)]
pub struct CurrencyBalance {
    pub currency: String,
    pub total: Decimal,        // Total du wallet (ajouts + gains - pertes - retraits)
    pub invested: Decimal,     // Montant investi dans les trades en cours
    pub treasury: Decimal,     // Trésorerie disponible (total - invested)
}

impl WalletService {
    /// Calcule les balances complètes pour toutes les devises d'un utilisateur
    pub async fn calculate_balances(
        db: &DatabaseConnection,
        user_id: i32,
    ) -> Result<Vec<CurrencyBalance>, DbErr> {
        // 1. Calculer le total du wallet par devise
        let wallet_totals = Self::calculate_wallet_totals(db, user_id).await?;

        // 2. Calculer les montants investis par devise
        let invested_amounts = Self::calculate_invested_amounts(db, user_id).await?;

        // 3. Combiner pour obtenir les balances finales
        let mut all_currencies: std::collections::HashSet<String> =
            wallet_totals.keys().cloned().collect();
        all_currencies.extend(invested_amounts.keys().cloned());

        let mut balances = Vec::new();
        for currency in all_currencies {
            let total = wallet_totals.get(&currency).copied().unwrap_or(Decimal::ZERO);
            let invested = invested_amounts.get(&currency).copied().unwrap_or(Decimal::ZERO);
            let treasury = total - invested;

            balances.push(CurrencyBalance {
                currency,
                total,
                invested,
                treasury,
            });
        }

        // Trier par devise pour cohérence
        balances.sort_by(|a, b| a.currency.cmp(&b.currency));

        Ok(balances)
    }

    /// Vérifie si l'utilisateur a assez de trésorerie disponible dans une devise
    /// pour effectuer un achat d'un montant donné
    pub async fn has_sufficient_funds(
        db: &DatabaseConnection,
        user_id: i32,
        currency: &str,
        required_amount: Decimal,
    ) -> Result<bool, DbErr> {
        let treasury = Self::get_treasury_for_currency(db, user_id, currency).await?;
        Ok(treasury >= required_amount)
    }

    /// Récupère la trésorerie disponible pour une devise spécifique
    /// Si la devise n'existe pas dans le wallet, retourne 0
    pub async fn get_treasury_for_currency(
        db: &DatabaseConnection,
        user_id: i32,
        currency: &str,
    ) -> Result<Decimal, DbErr> {
        let balances = Self::calculate_balances(db, user_id).await?;

        let balance = balances
            .iter()
            .find(|b| b.currency == currency);

        match balance {
            Some(b) => Ok(b.treasury),
            None => Ok(Decimal::ZERO),
        }
    }

    /// Retourne un message d'erreur détaillé en cas de fonds insuffisants
    pub async fn get_insufficient_funds_message(
        db: &DatabaseConnection,
        user_id: i32,
        currency: &str,
        required_amount: Decimal,
    ) -> Result<String, DbErr> {
        let treasury = Self::get_treasury_for_currency(db, user_id, currency).await?;

        Ok(format!(
            "Insufficient funds: {} {} available, {} {} required (shortage: {} {})",
            treasury,
            currency,
            required_amount,
            currency,
            required_amount - treasury,
            currency
        ))
    }

    /// Calcule le total du wallet par devise (ajouts + gains - pertes - retraits)
    async fn calculate_wallet_totals(
        db: &DatabaseConnection,
        user_id: i32,
    ) -> Result<HashMap<String, Decimal>, DbErr> {
        let transactions = wallet::Entity::find()
            .filter(wallet::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        let mut totals: HashMap<String, Decimal> = HashMap::new();

        for transaction in transactions {
            let balance = totals.entry(transaction.currency.clone()).or_insert(Decimal::ZERO);

            match transaction.action.as_str() {
                "gain" | "ajout" => *balance += transaction.amount,
                "perte" | "retrait" => *balance -= transaction.amount,
                _ => {}
            }
        }

        Ok(totals)
    }

    /// Calcule les montants investis par devise (positions ouvertes)
    async fn calculate_invested_amounts(
        db: &DatabaseConnection,
        user_id: i32,
    ) -> Result<HashMap<String, Decimal>, DbErr> {
        let trades = trade::Entity::find()
            .filter(trade::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        let mut invested: HashMap<String, Decimal> = HashMap::new();

        for t in trades {
            let symbol = match &t.symbol {
                Some(s) => s,
                None => continue,
            };

            // Récupérer la devise du stock
            let stock_option = stock::Entity::find()
                .filter(stock::Column::SymbolAlphavantage.eq(symbol))
                .one(db)
                .await?;

            let currency = match stock_option {
                Some(s) => s.currency.unwrap_or_else(|| "CAD".to_string()),
                None => {
                    eprintln!("⚠️  Stock not found for symbol: {}, defaulting to CAD", symbol);
                    "CAD".to_string()
                }
            };

            let inv = invested.entry(currency).or_insert(Decimal::ZERO);

            // Calculer le montant selon le type de trade
            // IMPORTANT: Utiliser quantite_restante pour les achats (quantité encore en position)
            let trade_type = t.trade_type.as_deref().unwrap_or("");
            let prix_unitaire = t.prix_unitaire.unwrap_or(Decimal::ZERO);

            match trade_type {
                "achat" => {
                    // Utiliser quantite_restante (ce qui est encore investi)
                    let montant = t.quantite_restante * prix_unitaire;
                    *inv += montant;
                }
                "vente" => {
                    // Les ventes réduisent l'investissement, mais c'est déjà géré
                    // par quantite_restante des achats, donc on ne fait rien ici
                }
                _ => {}
            }
        }

        Ok(invested)
    }
}