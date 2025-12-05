use serde::Serialize;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "stock")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub compagny_name: String,
    pub is_alive: Option<String>,
    pub low_data: Option<String>,
    pub symbol_alphavantage: Option<String>,
    pub currency: Option<String>,
}

//C'est le placeholder pour les relations futures.
//Un stock a plusieurs position
////#[sea_orm(has_many = "super::position::Entity")]
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

//définit des hooks (actions automatiques) avant/après insert/update/delete.
impl ActiveModelBehavior for ActiveModel {}