use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};

use entity::eve_character::Model as Character;
use entity::prelude::EveCharacter;

pub struct CharacterRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> CharacterRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        character_id: i32,
        character_name: String,
        corporation_id: i32,
    ) -> Result<Character, sea_orm::DbErr> {
        let character = entity::eve_character::ActiveModel {
            character_id: Set(character_id),
            character_name: Set(character_name),
            corporation_id: Set(corporation_id),
            last_updated: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        character.insert(self.db).await
    }

    pub async fn update(
        &self,
        character_id: i32,
        new_corporation_id: i32,
    ) -> Result<Character, sea_orm::DbErr> {
        let character = self.get_one(character_id).await?;

        match character {
            Some(character) => {
                let mut character: entity::eve_character::ActiveModel = character.into();

                character.corporation_id = Set(new_corporation_id);
                character.last_updated = Set(chrono::Utc::now().naive_utc());

                character.update(self.db).await
            }
            None => Err(sea_orm::DbErr::RecordNotFound(format!(
                "Character with id {} not found",
                character_id
            ))),
        }
    }

    pub async fn get_one(&self, id: i32) -> Result<Option<Character>, sea_orm::DbErr> {
        EveCharacter::find_by_id(id).one(self.db).await
    }

    pub async fn get_many(
        &self,
        ids: &[i32],
        page: u64,
        page_size: u64,
    ) -> Result<Vec<Character>, sea_orm::DbErr> {
        let ids: Vec<sea_orm::Value> = ids.iter().map(|&id| id.into()).collect();

        EveCharacter::find()
            .filter(entity::eve_character::Column::Id.is_in(ids))
            .paginate(self.db, page_size)
            .fetch_page(page)
            .await
    }

    pub async fn get_by_filtered(
        &self,
        filters: Vec<migration::SimpleExpr>,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<Character>, sea_orm::DbErr> {
        let mut query = EveCharacter::find();

        for filter in filters {
            query = query.filter(filter);
        }

        query.paginate(self.db, page_size).fetch_page(page).await
    }
}
#[cfg(test)]
mod tests {
    use crate::eve::data::corporation::CorporationRepository;

    use super::*;
    use rand::{distributions::Alphanumeric, Rng};
    use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};

    async fn initialize_test(db: &DatabaseConnection) -> Result<i32, sea_orm::DbErr> {
        let schema = Schema::new(DbBackend::Sqlite);

        let stmts = vec![
            schema.create_table_from_entity(entity::prelude::EveAlliance),
            schema.create_table_from_entity(entity::prelude::EveCorporation),
            schema.create_table_from_entity(entity::prelude::EveCharacter),
        ];

        for stmt in stmts {
            let _ = db.execute(db.get_database_backend().build(&stmt)).await?;
        }

        let mut rng = rand::thread_rng();

        // create corporation first due to foreign key constraint
        let corporation_id = rng.gen::<i32>();
        let alliance_id = None;
        let ceo = rng.gen::<i32>();
        let corporation_name = (&mut rng)
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect::<String>();

        let corporation_repo = CorporationRepository::new(db);

        let _ = corporation_repo
            .create(corporation_id, corporation_name.clone(), alliance_id, ceo)
            .await?;

        Ok(corporation_id)
    }

    #[tokio::test]
    async fn create_character() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        let corporation_id = initialize_test(&db).await?;
        let character_repo = CharacterRepository::new(&db);

        let mut rng = rand::thread_rng();

        let character_id = rng.gen::<i32>();
        let character_name: String = rng
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        let created_character = character_repo
            .create(character_id, character_name.clone(), corporation_id)
            .await?;

        assert_eq!(created_character.character_id, character_id);
        assert_eq!(created_character.character_name, character_name);
        assert_eq!(created_character.corporation_id, corporation_id);

        Ok(())
    }

    #[tokio::test]
    async fn get_one_character() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        let corporation_id = initialize_test(&db).await?;
        let character_repo = CharacterRepository::new(&db);

        let mut rng = rand::thread_rng();

        let character_id = rng.gen::<i32>();
        let character_name: String = rng
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        let created_character = character_repo
            .create(character_id, character_name.clone(), corporation_id)
            .await?;

        let retrieved_character = character_repo.get_one(created_character.id).await?;

        assert_eq!(retrieved_character.unwrap(), created_character);

        Ok(())
    }

    #[tokio::test]
    async fn get_many_characters() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        let corporation_id = initialize_test(&db).await?;
        let character_repo = CharacterRepository::new(&db);

        let mut rng = rand::thread_rng();
        let mut created_characters = Vec::new();

        let mut generated_ids = std::collections::HashSet::new();
        for _ in 0..5 {
            let mut character_id: i32 = rng.gen::<i32>();
            while generated_ids.contains(&character_id) {
                character_id = rng.gen::<i32>();
            }
            generated_ids.insert(character_id);

            let character_name: String = (&mut rng)
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();

            let created_character = character_repo
                .create(character_id, character_name.clone(), corporation_id)
                .await?;

            created_characters.push(created_character);
        }

        let created_character_ids = created_characters
            .iter()
            .map(|a| a.id)
            .collect::<Vec<i32>>();

        let mut retrieved_characters = character_repo
            .get_many(&created_character_ids, 0, 5)
            .await?;

        created_characters.sort_by_key(|a| a.id);
        retrieved_characters.sort_by_key(|a| a.id);

        assert_eq!(retrieved_characters, created_characters);

        Ok(())
    }

    #[tokio::test]
    async fn get_filtered_characters() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        let corporation_id = initialize_test(&db).await?;
        let character_repo = CharacterRepository::new(&db);

        let mut created_characters = Vec::new();

        let mut rng = rand::thread_rng();

        let mut generated_ids = std::collections::HashSet::new();
        for _ in 0..5 {
            let mut character_id: i32 = rng.gen::<i32>();
            while generated_ids.contains(&character_id) {
                character_id = rng.gen::<i32>();
            }
            generated_ids.insert(character_id);

            let character_name: String = (&mut rng)
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();

            let created_character = character_repo
                .create(character_id, character_name.clone(), corporation_id)
                .await?;

            created_characters.push(created_character);
        }

        let filters =
            vec![entity::eve_character::Column::CharacterId.eq(created_characters[0].character_id)];

        let retrieved_characters = character_repo.get_by_filtered(filters, 0, 5).await?;

        assert_eq!(retrieved_characters.len(), 1);

        Ok(())
    }
}
