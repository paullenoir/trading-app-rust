use serde::Serialize;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "historicdata")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub symbol: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub date: String,
    pub open: Option<String>,
    pub high: Option<String>,
    pub low: Option<String>,
    pub close: Option<String>,
    pub volume: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}