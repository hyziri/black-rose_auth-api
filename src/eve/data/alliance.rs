use std::collections::HashSet;

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use entity::eve_alliance::Model as Alliance;
use entity::prelude::EveAlliance;

pub async fn get_alliance(
    db: &DatabaseConnection,
    alliance_id: i32,
) -> Result<Option<Alliance>, sea_orm::DbErr> {
    EveAlliance::find()
        .filter(entity::eve_alliance::Column::AllianceId.eq(alliance_id))
        .one(db)
        .await
}

pub async fn create_alliance(
    db: &DatabaseConnection,
    alliance_id: i32,
) -> Result<Alliance, anyhow::Error> {
    match get_alliance(db, alliance_id).await? {
        Some(alliance) => Ok(alliance),
        None => {
            let alliance = eve_esi::alliance::get_alliance(alliance_id).await?;

            let alliance = entity::eve_alliance::ActiveModel {
                alliance_id: ActiveValue::Set(alliance_id),
                alliance_name: ActiveValue::Set(alliance.name),
                ..Default::default()
            };

            let alliance: Alliance = alliance.insert(db).await?;

            Ok(alliance)
        }
    }
}

pub async fn bulk_get_alliances(
    db: &DatabaseConnection,
    alliance_ids: Vec<i32>,
) -> Result<Vec<Alliance>, sea_orm::DbErr> {
    let unique_alliance_ids: Vec<i32> = alliance_ids
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    entity::prelude::EveAlliance::find()
        .filter(entity::eve_alliance::Column::AllianceId.is_in(unique_alliance_ids))
        .all(db)
        .await
}
