use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use std::collections::HashSet;

use entity::eve_character::Model as Character;

use crate::eve::{
    data::corporation::create_corporation, model::character::CharacterAffiliationDto,
};

use super::{alliance::bulk_get_alliances, corporation::bulk_get_corporations};

pub async fn create_character(
    db: &DatabaseConnection,
    character_id: i32,
    character_name: Option<String>,
) -> Result<Character, anyhow::Error> {
    match get_character(db, character_id).await? {
        Some(character) => Ok(character),
        None => {
            let character_name = match character_name {
                Some(name) => name,
                None => {
                    let character = eve_esi::character::get_character(character_id).await?;

                    character.name
                }
            };

            let affiliation =
                eve_esi::character::get_character_affiliations(vec![character_id]).await?;

            let character = entity::eve_character::ActiveModel {
                character_id: Set(character_id),
                character_name: Set(character_name),
                corporation_id: Set(affiliation[0].corporation_id),
                last_updated: Set(chrono::Utc::now().naive_utc()),
                ..Default::default()
            };

            let _ = create_corporation(db, affiliation[0].corporation_id).await;

            let character: Character = character.insert(db).await?;

            Ok(character)
        }
    }
}

pub async fn get_character(
    db: &DatabaseConnection,
    character_id: i32,
) -> Result<Option<Character>, sea_orm::DbErr> {
    entity::prelude::EveCharacter::find()
        .filter(entity::eve_character::Column::CharacterId.eq(character_id))
        .one(db)
        .await
}

pub async fn bulk_get_characters(
    db: &DatabaseConnection,
    character_ids: Vec<i32>,
) -> Result<Vec<Character>, sea_orm::DbErr> {
    let unique_character_ids: Vec<i32> = character_ids
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    entity::prelude::EveCharacter::find()
        .filter(entity::eve_character::Column::CharacterId.is_in(unique_character_ids))
        .all(db)
        .await
}

pub async fn bulk_get_character_affiliations(
    db: &DatabaseConnection,
    character_ids: Vec<i32>,
) -> Result<Vec<CharacterAffiliationDto>, sea_orm::DbErr> {
    let characters = bulk_get_characters(db, character_ids).await?;

    let corporation_ids: HashSet<i32> = characters
        .clone()
        .into_iter()
        .map(|character| character.corporation_id)
        .collect();
    let unique_corporation_ids: Vec<i32> = corporation_ids.into_iter().collect();
    let corporations = bulk_get_corporations(db, unique_corporation_ids).await?;

    let alliance_ids: HashSet<i32> = corporations
        .clone()
        .into_iter()
        .filter_map(|corporation| corporation.alliance_id)
        .collect();
    let unique_alliance_ids: Vec<i32> = alliance_ids.into_iter().collect();
    let alliances = bulk_get_alliances(db, unique_alliance_ids).await?;

    let mut character_affiliations: Vec<CharacterAffiliationDto> = Vec::new();

    for character in characters {
        let corporation = corporations
            .iter()
            .find(|corporation| corporation.corporation_id == character.corporation_id)
            .cloned()
            .unwrap();

        let mut alliance_id = None::<i32>;
        let mut alliance_name = None::<String>;

        if let Some(character_alliance_id) = corporation.alliance_id {
            let alliance = alliances
                .iter()
                .find(|alliance| alliance.alliance_id == character_alliance_id)
                .cloned()
                .unwrap();

            alliance_id = Some(alliance.alliance_id);
            alliance_name = Some(alliance.alliance_name);
        }

        let new_character = CharacterAffiliationDto {
            character_id: character.character_id,
            character_name: character.character_name,
            corporation_id: corporation.corporation_id,
            corporation_name: corporation.corporation_name,
            alliance_id,
            alliance_name,
        };

        character_affiliations.push(new_character);
    }

    Ok(character_affiliations)
}

pub async fn update_affiliation(
    db: &DatabaseConnection,
    character_ids: Vec<i32>,
) -> Result<(), anyhow::Error> {
    let characters = bulk_get_characters(db, character_ids.clone()).await?;
    let affiliations = eve_esi::character::get_character_affiliations(character_ids).await?;

    for character in characters {
        let affiliation = affiliations
            .iter()
            .find(|affiliation| affiliation.character_id == character.character_id);

        if let Some(affiliation) = affiliation {
            let mut character: entity::eve_character::ActiveModel = character.into();

            character.corporation_id = ActiveValue::Set(affiliation.corporation_id);

            character.update(db).await?;
        }
    }

    Ok(())
}
