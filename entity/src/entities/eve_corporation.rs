//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "eve_corporation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub corporation_id: i32,
    pub corporation_name: String,
    pub alliance_id: Option<i32>,
    pub ceo: i32,
    pub last_updated: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::eve_alliance::Entity",
        from = "Column::AllianceId",
        to = "super::eve_alliance::Column::AllianceId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    EveAlliance,
    #[sea_orm(has_many = "super::eve_character::Entity")]
    EveCharacter,
}

impl Related<super::eve_alliance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EveAlliance.def()
    }
}

impl Related<super::eve_character::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EveCharacter.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
