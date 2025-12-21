use serde::{Serialize, Deserialize};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: Option<String>,
    pub password_hash: Option<String>, // Format: pbkdf2:sha256:iterations$salt$hash
    #[serde(skip_serializing)] // Ne pas exposer le wallet en JSON (temporaire)
    pub wallet: Option<String>, // Temporaire - changera en JSONB plus tard
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::wallet::Entity")]
    Wallet,

    #[sea_orm(has_many = "super::trade::Entity")]
    Trade,

    #[sea_orm(has_many = "super::trades_fermes::Entity")]
    TradesFermes,
}

impl Related<super::wallet::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Wallet.def()
    }
}

impl Related<super::trade::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Trade.def()
    }
}

impl Related<super::trades_fermes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TradesFermes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}