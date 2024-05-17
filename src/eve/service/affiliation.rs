use std::collections::HashSet;

use sea_orm::DatabaseConnection;

use sea_orm::ColumnTrait;

use crate::error::DbOrReqwestError;
use crate::eve::data::alliance::AllianceRepository;
use crate::eve::data::character::CharacterRepository;
use crate::eve::data::corporation::CorporationRepository;
use crate::eve::model::character::CharacterAffiliationDto;

pub async fn update_affiliation(
    db: &DatabaseConnection,
    character_ids: Vec<i32>,
) -> Result<(), DbOrReqwestError> {
    #[cfg(test)]
    use crate::mock::eve_esi_mock::get_character_affiliations;
    #[cfg(not(test))]
    use eve_esi::character::get_character_affiliations;

    let repo = CharacterRepository::new(db);

    let character_ids_len = character_ids.len() as u64;

    let filters = vec![entity::eve_character::Column::CharacterId.is_in(character_ids.clone())];

    let characters = repo.get_by_filtered(filters, 0, character_ids_len).await?;
    let affiliations = get_character_affiliations(character_ids).await?;

    for character in characters {
        let affiliation = affiliations
            .iter()
            .find(|affiliation| affiliation.character_id == character.character_id);

        if let Some(affiliation) = affiliation {
            repo.update(character.id, affiliation.corporation_id)
                .await?;
        }
    }

    Ok(())
}

pub async fn get_character_affiliations(
    db: &DatabaseConnection,
    character_ids: Vec<i32>,
) -> Result<Vec<CharacterAffiliationDto>, DbOrReqwestError> {
    async fn get_characters(
        db: &DatabaseConnection,
        character_ids: Vec<i32>,
    ) -> Result<Vec<entity::eve_character::Model>, sea_orm::DbErr> {
        let character_repo = CharacterRepository::new(db);

        let character_ids_len = character_ids.len() as u64;

        let filters = vec![entity::eve_character::Column::CharacterId.is_in(character_ids)];

        let characters = character_repo
            .get_by_filtered(filters, 0, character_ids_len)
            .await?;

        Ok(characters)
    }

    async fn get_corporations(
        db: &DatabaseConnection,
        corporation_ids: HashSet<i32>,
    ) -> Result<Vec<entity::eve_corporation::Model>, sea_orm::DbErr> {
        let corporation_repo = CorporationRepository::new(db);

        let unique_corporation_ids: Vec<i32> = corporation_ids.into_iter().collect();

        let corporation_ids_len = unique_corporation_ids.len() as u64;

        let filters =
            vec![entity::eve_corporation::Column::CorporationId.is_in(unique_corporation_ids)];

        let corporations = corporation_repo
            .get_by_filtered(filters, 0, corporation_ids_len)
            .await?;

        Ok(corporations)
    }

    async fn get_alliances(
        db: &DatabaseConnection,
        alliance_ids: HashSet<i32>,
    ) -> Result<Vec<entity::eve_alliance::Model>, sea_orm::DbErr> {
        let alliance_repo = AllianceRepository::new(db);

        let unique_alliance_ids: Vec<i32> = alliance_ids.into_iter().collect();

        let alliance_ids_len = unique_alliance_ids.len() as u64;

        let filters =
            vec![entity::eve_alliance::Column::AllianceId.is_in(unique_alliance_ids.to_owned())];

        let alliances = alliance_repo
            .get_by_filtered(filters, 0, alliance_ids_len)
            .await?;

        Ok(alliances)
    }

    let characters = get_characters(db, character_ids).await?;

    let corporations =
        get_corporations(db, characters.iter().map(|c| c.corporation_id).collect()).await?;

    let alliances = get_alliances(
        db,
        corporations.iter().filter_map(|c| c.alliance_id).collect(),
    )
    .await?;

    let mut character_affiliations: Vec<CharacterAffiliationDto> = Vec::new();

    for character in characters {
        let corporation = corporations
            .iter()
            .find(|corporation| corporation.corporation_id == character.corporation_id)
            .unwrap();

        let mut alliance_id = None::<i32>;
        let mut alliance_name = None::<String>;

        if let Some(character_alliance_id) = corporation.alliance_id {
            let alliance = alliances
                .iter()
                .find(|alliance| alliance.alliance_id == character_alliance_id)
                .unwrap();

            alliance_id = Some(alliance.alliance_id);
            alliance_name = Some(alliance.alliance_name.clone());
        }

        let new_character = CharacterAffiliationDto {
            character_id: character.character_id,
            character_name: character.character_name,
            corporation_id: corporation.corporation_id,
            corporation_name: corporation.corporation_name.clone(),
            alliance_id,
            alliance_name,
        };

        character_affiliations.push(new_character);
    }

    Ok(character_affiliations)
}

#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};

    use crate::{
        error::DbOrReqwestError,
        eve::data::{character::CharacterRepository, corporation::CorporationRepository},
    };

    #[tokio::test]
    async fn update_affiliation() -> Result<(), DbOrReqwestError> {
        use super::update_affiliation;

        let db = Database::connect("sqlite::memory:").await?;
        let schema = Schema::new(DbBackend::Sqlite);

        let stmts = vec![
            schema.create_table_from_entity(entity::prelude::EveCharacter),
            schema.create_table_from_entity(entity::prelude::EveCorporation),
            schema.create_table_from_entity(entity::prelude::EveAlliance),
        ];

        for stmt in stmts {
            let _ = db.execute(db.get_database_backend().build(&stmt)).await?;
        }

        let corporation_repo = CorporationRepository::new(&db);
        let character_repo = CharacterRepository::new(&db);

        // Create corporations first to avoid sqlite foreignn key contraint errors
        // old corp

        let _ = corporation_repo
            .create(98755360, "Black Rose Inc.".to_string(), None, 2114794365)
            .await?;

        // new corp

        let _ = corporation_repo
            .create(109299958, "C C P".to_string(), None, 180548812)
            .await?;

        let character = character_repo
            .create(2114794365, "Hyziri".to_string(), 98755360)
            .await?;

        println!("{}", character.id);

        update_affiliation(&db, vec![character.character_id]).await?;

        let updated_character = character_repo.get_one(character.id).await?;

        match updated_character {
            Some(updated_character) => {
                assert_ne!(character.corporation_id, updated_character.corporation_id);

                Ok(())
            }
            None => Err(sea_orm::DbErr::RecordNotFound(format!(
                "Character with id {} not found",
                character.id
            ))
            .into()),
        }
    }
}
