use serde::{Serialize, Deserialize};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "trade")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub date: Option<String>,
    pub symbol: Option<String>,
    #[serde(rename = "type")]
    #[sea_orm(column_name = "type")]
    pub trade_type: Option<String>,
    pub quantite: Option<Decimal>,
    pub prix_unitaire: Option<Decimal>,
    pub prix_total: Option<Decimal>,

    // NOUVEAU: quantite_restante pour tracking FIFO
    // Pour les achats: quantité encore disponible pour fermeture
    // Pour les ventes: toujours 0 (les ventes sont consommées immédiatement)
    //
    // Exemple:
    // - Achat 100 AAPL → quantite=100, quantite_restante=100
    // - Vente 30 AAPL  → Le trade d'achat devient: quantite=100, quantite_restante=70
    // - Vente 70 AAPL  → Le trade d'achat devient: quantite=100, quantite_restante=0
    pub quantite_restante: Decimal,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}