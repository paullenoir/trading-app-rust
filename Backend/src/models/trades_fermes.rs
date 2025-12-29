use serde::{Serialize, Deserialize};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "trades_fermes_rust")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: i32,
    pub symbol: Option<String>,
    pub date_achat: Option<String>,
    pub prix_achat: Option<String>,
    pub date_vente: Option<String>,
    pub prix_vente: Option<String>,
    pub pourcentage_gain: Option<i32>,
    pub gain_dollars: Option<Decimal>,
    pub temps_jours: Option<i32>,
    pub trade_achat_id: Option<i32>,
    pub trade_vente_id: Option<i32>,
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