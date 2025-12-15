use serde::Serialize;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
//#[sea_orm(table_name = "strategy_results")]
#[sea_orm(table_name = "strategy_results_test")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub strategy_id: i32,
    pub symbol: Option<String>,
    pub date: Option<String>,
    pub recommendation: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::strategy::Entity",
        from = "Column::StrategyId",
        to = "super::strategy::Column::Id"
    )]
    Strategy,

    #[sea_orm(
        belongs_to = "super::stock::Entity",
        from = "Column::Symbol",
        to = "super::stock::Column::SymbolAlphavantage"
    )]
    Stock,
}

impl Related<super::strategy::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Strategy.def()
    }
}

impl Related<super::stock::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Stock.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}