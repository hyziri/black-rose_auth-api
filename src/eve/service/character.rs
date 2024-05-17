use sea_orm::{ColumnTrait, DatabaseConnection};

use entity::eve_character::Model as Character;

use crate::error::DbOrReqwestError;
use crate::eve::data::character::CharacterRepository;

use super::corporation::get_or_create_corporation;

#[cfg(not(test))]
use eve_esi::character::get_character;

#[cfg(test)]
use crate::mock::eve_esi_mock::get_character;

pub async fn get_or_create_character(
    db: &DatabaseConnection,
    character_id: i32,
) -> Result<Character, DbOrReqwestError> {
    let repo = CharacterRepository::new(db);

    let filters = vec![entity::eve_character::Column::CharacterId.eq(character_id)];
    let mut character = repo.get_by_filtered(filters, 0, 1).await?;

    let character = match character.pop() {
        Some(character) => return Ok(character),
        None => {
            let character = get_character(character_id).await?;

            let _ = get_or_create_corporation(db, character.corporation_id).await?;

            repo.create(character_id, character.name, character.corporation_id)
                .await?
        }
    };

    Ok(character)
}

#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};

    use crate::error::DbOrReqwestError;

    #[tokio::test]
    async fn get_or_create_character() -> Result<(), DbOrReqwestError> {
        use super::get_or_create_character;

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

        let character = get_or_create_character(&db, 180548812).await?;
        let character_2 = get_or_create_character(&db, 180548812).await?;

        assert_eq!(character, character_2);

        Ok(())
    }
}
