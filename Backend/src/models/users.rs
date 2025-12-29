use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users_rust")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub abonnement_id: Option<i32>,  // ← AJOUTER
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::abonnement::Entity",
        from = "Column::AbonnementId",
        to = "super::abonnement::Column::Id"
    )]
    Abonnement,  // ← AJOUTER
}

impl Related<super::abonnement::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Abonnement.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}