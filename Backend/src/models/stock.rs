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

//QUOI: definir les relations
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::strategy_result::Entity")]
    StrategyResults,
}

//COMMENT utiliser les relations
//Pour l'entité Strategy (Entity), voici comment aller vers StrategyResult"
impl Related<super::strategy_result::Entity> for Entity {
    //Cette fonction retourne la définition de la relation à utiliser
    fn to() -> RelationDef {
        Relation::StrategyResults.def()
        //.def() = Convertit en RelationDef (format utilisable par SeaORM)
    }
}

//définit des hooks (actions automatiques) avant/après insert/update/delete.
impl ActiveModelBehavior for ActiveModel {}