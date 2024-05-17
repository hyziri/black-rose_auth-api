use sea_orm::DatabaseConnection;

use sea_orm::ColumnTrait;

use crate::error::DbOrReqwestError;
use crate::eve::data::character::CharacterRepository;

#[cfg(not(test))]
use eve_esi::character::get_character_affiliations;

#[cfg(test)]
use crate::mock::eve_esi_mock::get_character_affiliations;

pub async fn update_affiliation(
    db: &DatabaseConnection,
    character_ids: Vec<i32>,
) -> Result<(), DbOrReqwestError> {
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
