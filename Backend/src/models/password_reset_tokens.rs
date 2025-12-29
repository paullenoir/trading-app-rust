// ============================================================================
// MODÈLE : PASSWORD RESET TOKENS
// ============================================================================
//
// Description:
//   Modèle de la table password_reset_tokens_rust correspondant EXACTEMENT
//   à la structure SQL créée par la migration.
//
// Colonnes de la table password_reset_tokens_rust:
//   - id (INTEGER, PRIMARY KEY, SERIAL)
//   - user_id (INTEGER, NOT NULL, FK vers users_rust)
//   - token (VARCHAR, UNIQUE, NOT NULL) - UUID v4
//   - expires_at (TIMESTAMP, NOT NULL) - created_at + 1 heure
//   - used (BOOLEAN, DEFAULT FALSE, NOT NULL)
//   - created_at (TIMESTAMP, DEFAULT CURRENT_TIMESTAMP)
//
// Workflow:
//   1. User demande reset via POST /api/auth/forgot-password
//   2. Backend génère un token UUID v4 et l'insère dans cette table
//   3. Backend envoie email avec lien contenant le token
//   4. User clique sur le lien
//   5. Frontend envoie POST /api/auth/reset-password avec token + nouveau password
//   6. Backend vérifie: token existe, not expired, not used
//   7. Backend change le password et met used = true
//
// Dépendances:
//   - sea_orm : ORM pour PostgreSQL
//   - serde : Sérialisation/désérialisation JSON
//
// Points d'attention:
//   - Un token ne peut être utilisé qu'une fois (used = true)
//   - Token expire après 1 heure (3600 secondes)
//   - Token est un UUID v4 (très difficile à deviner)
//   - ON DELETE CASCADE: si user supprimé, tokens supprimés aussi
//
// ============================================================================

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "password_reset_tokens_rust")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub user_id: i32,

    #[sea_orm(unique)]
    pub token: String,

    pub expires_at: DateTime,

    pub used: bool,

    pub created_at: Option<DateTime>,
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