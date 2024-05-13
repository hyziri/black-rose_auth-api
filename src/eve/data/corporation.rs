use std::collections::HashSet;

use entity::prelude::EveCorporation;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use entity::eve_corporation::Model as Corporation;

use crate::eve::data::alliance::create_alliance;

pub async fn get_corporation(
    db: &DatabaseConnection,
    corporation_id: i32,
) -> Result<Option<Corporation>, sea_orm::DbErr> {
    EveCorporation::find()
        .filter(entity::eve_corporation::Column::CorporationId.eq(corporation_id))
        .one(db)
        .await
}

pub async fn create_corporation(
    db: &DatabaseConnection,
    corporation_id: i32,
) -> Result<Corporation, anyhow::Error> {
    match get_corporation(db, corporation_id).await? {
        Some(corporation) => Ok(corporation),
        None => {
            let corporation = eve_esi::corporation::get_corporation(corporation_id).await?;

            let new_corporation = entity::eve_corporation::ActiveModel {
                corporation_id: Set(corporation_id),
                corporation_name: Set(corporation.name),
                alliance_id: Set(corporation.alliance_id),
                ceo: Set(corporation.ceo_id),
                last_updated: Set(chrono::Utc::now().naive_utc()),
                ..Default::default()
            };

            if let Some(alliance_id) = corporation.alliance_id {
                let _ = create_alliance(db, alliance_id).await;
            }

            let corporation: Corporation = new_corporation.insert(db).await?;

            Ok(corporation)
        }
    }
}

pub async fn bulk_get_corporations(
    db: &DatabaseConnection,
    corporation_ids: Vec<i32>,
) -> Result<Vec<Corporation>, sea_orm::DbErr> {
    let unique_corp_ids: Vec<i32> = corporation_ids
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    entity::prelude::EveCorporation::find()
        .filter(entity::eve_corporation::Column::CorporationId.is_in(unique_corp_ids))
        .all(db)
        .await
}
