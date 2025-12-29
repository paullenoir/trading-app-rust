use serde::Serialize;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "strategies_rust")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: Option<String>,
    pub created_by: Option<String>,
    pub shared_with: Option<Vec<String>>,
    pub is_public: Option<bool>,
    pub strategy_config: Option<serde_json::Value>,  // Pour le JSONB
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::strategy_result::Entity")]
    StrategyResults,
}

impl Related<super::strategy_result::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StrategyResults.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}