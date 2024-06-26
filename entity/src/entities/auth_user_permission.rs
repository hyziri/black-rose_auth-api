//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "auth_user_permission")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub permission_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::auth_permission::Entity",
        from = "Column::UserId",
        to = "super::auth_permission::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    AuthPermission,
    #[sea_orm(
        belongs_to = "super::auth_user::Entity",
        from = "Column::UserId",
        to = "super::auth_user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    AuthUser,
}

impl Related<super::auth_permission::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuthPermission.def()
    }
}

impl Related<super::auth_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuthUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
