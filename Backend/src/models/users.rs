// ============================================================================
// MODÈLE : USERS
// ============================================================================
//
// Description:
//   Modèle de la table users_rust correspondant EXACTEMENT à la structure
//   SQL créée par la migration. Gère l'authentification classique et OAuth Google.
//
// Colonnes de la table users_rust:
//   - id (INTEGER, PRIMARY KEY, SERIAL)
//   - username (VARCHAR, UNIQUE, NOT NULL)
//   - password_hash (VARCHAR, NULL) - NULL pour OAuth Google
//   - email (VARCHAR, UNIQUE, NOT NULL)
//   - google_id (VARCHAR, UNIQUE, NULL)
//   - email_verified (BOOLEAN, DEFAULT FALSE, NOT NULL)
//   - abonnement_id (INTEGER, NULL, FK vers abonnements_rust)
//   - created_at (TIMESTAMP, DEFAULT CURRENT_TIMESTAMP)
//   - updated_at (TIMESTAMP, DEFAULT CURRENT_TIMESTAMP)
//
// Dépendances:
//   - sea_orm : ORM pour PostgreSQL
//   - serde : Sérialisation/désérialisation JSON
//
// Points d'attention:
//   - password_hash est Option<String> car NULL pour OAuth Google
//   - email est obligatoire et unique
//   - google_id est Option<String> car NULL si pas OAuth
//   - Relations avec abonnements_rust définie
//
// ============================================================================

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users_rust")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique)]
    pub username: String,

    // Option<String> car NULL pour les users Google OAuth
    pub password_hash: Option<String>,

    #[sea_orm(unique)]
    pub email: String,

    // Option<String> car NULL si pas Google OAuth
    #[sea_orm(unique)]
    pub google_id: Option<String>,

    pub email_verified: bool,

    pub abonnement_id: Option<i32>,

    pub created_at: Option<DateTime>,

    pub updated_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::abonnement::Entity",
        from = "Column::AbonnementId",
        to = "super::abonnement::Column::Id"
    )]
    Abonnement,
}

impl Related<super::abonnement::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Abonnement.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}