use serde::Serialize;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "indicators_test")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub date: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub symbol: String,
    pub ema20: Option<String>,
    pub ema50: Option<String>,
    pub ema200: Option<String>,
    pub rsi25: Option<String>,
    pub stochastic14_7_7: Option<String>,
    pub point_pivot: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}