use entity::auth_user_character_ownership::Model as UserCharacterOwnership;
use entity::prelude::AuthUserCharacterOwnership;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Set,
};

pub struct UserCharacterOwnershipRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserCharacterOwnershipRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        user_id: i32,
        character_id: i32,
        ownerhash: String,
        main: bool,
    ) -> Result<UserCharacterOwnership, sea_orm::DbErr> {
        let user_ownership = entity::auth_user_character_ownership::ActiveModel {
            user_id: Set(user_id),
            character_id: Set(character_id),
            ownerhash: Set(ownerhash),
            main: Set(main),
            ..Default::default()
        };

        user_ownership.insert(self.db).await
    }

    pub async fn get_one(&self, id: i32) -> Result<Option<UserCharacterOwnership>, sea_orm::DbErr> {
        AuthUserCharacterOwnership::find_by_id(id)
            .one(self.db)
            .await
    }

    pub async fn get_by_filtered(
        &self,
        filters: Vec<migration::SimpleExpr>,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<UserCharacterOwnership>, sea_orm::DbErr> {
        let mut query = AuthUserCharacterOwnership::find();

        for filter in filters {
            query = query.filter(filter);
        }

        query.paginate(self.db, page_size).fetch_page(page).await
    }

    pub async fn update(
        &self,
        id: i32,
        user_id: i32,
        ownerhash: String,
        main: bool,
    ) -> Result<UserCharacterOwnership, sea_orm::DbErr> {
        let user_ownership = self.get_one(id).await?;

        match user_ownership {
            Some(user_ownership) => {
                let mut user_ownership: entity::auth_user_character_ownership::ActiveModel =
                    user_ownership.into();

                user_ownership.user_id = Set(user_id);
                user_ownership.ownerhash = Set(ownerhash);
                user_ownership.main = Set(main);

                user_ownership.update(self.db).await
            }
            None => Err(sea_orm::DbErr::RecordNotFound(format!(
                "Character ownership with id {} not found",
                id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::data::user::UserRepository,
        eve::data::{character::CharacterRepository, corporation::CorporationRepository},
    };

    use super::*;
    use rand::{distributions::Alphanumeric, Rng};
    use sea_orm::{ColumnTrait, ConnectionTrait, Database, DbBackend, Schema};

    async fn initialize_test(
        db: &DatabaseConnection,
    ) -> Result<
        (
            entity::eve_corporation::Model,
            entity::eve_character::Model,
            entity::auth_user::Model,
        ),
        sea_orm::DbErr,
    > {
        let schema = Schema::new(DbBackend::Sqlite);
        let corporation_repo = CorporationRepository::new(db);
        let character_repo = CharacterRepository::new(db);
        let user_repo = UserRepository::new(db);

        let stmts = vec![
            schema.create_table_from_entity(entity::prelude::AuthUser),
            schema.create_table_from_entity(entity::prelude::EveAlliance),
            schema.create_table_from_entity(entity::prelude::EveCorporation),
            schema.create_table_from_entity(entity::prelude::EveCharacter),
            schema.create_table_from_entity(entity::prelude::AuthUserCharacterOwnership),
        ];

        for stmt in stmts {
            let _ = db.execute(db.get_database_backend().build(&stmt)).await?;
        }

        let mut rng = rand::thread_rng();

        // create user, character, & corporation first due to foreign key constraint
        let corporation_id = rng.gen::<i32>();
        let alliance_id = None;
        let ceo = rng.gen::<i32>();
        let corporation_name = (&mut rng)
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect::<String>();

        let corporation = corporation_repo
            .create(corporation_id, corporation_name.clone(), alliance_id, ceo)
            .await?;

        let character_id = rng.gen::<i32>();
        let character_name: String = rng
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        let character = character_repo
            .create(character_id, character_name.clone(), corporation_id)
            .await?;

        let user = user_repo.create(false).await?;

        Ok((corporation, character, user))
    }

    #[tokio::test]
    async fn create_user() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        let test_data = initialize_test(&db).await?;
        let ownership_repo = UserCharacterOwnershipRepository::new(&db);

        let rng = rand::thread_rng();

        let user_id = test_data.2.id;
        let character_id = test_data.1.character_id;
        let ownerhash: String = rng
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        let main = true;

        let created_ownership = ownership_repo
            .create(user_id, character_id, ownerhash.clone(), main)
            .await?;

        assert_eq!(user_id, created_ownership.user_id);
        assert_eq!(character_id, created_ownership.character_id);
        assert_eq!(ownerhash, created_ownership.ownerhash);
        assert_eq!(main, created_ownership.main);

        Ok(())
    }

    #[tokio::test]
    async fn get_one_user() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        let test_data = initialize_test(&db).await?;
        let ownership_repo = UserCharacterOwnershipRepository::new(&db);

        let rng = rand::thread_rng();

        let user_id = test_data.2.id;
        let character_id = test_data.1.character_id;
        let ownerhash: String = rng
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        let main = true;

        let created_ownership = ownership_repo
            .create(user_id, character_id, ownerhash.clone(), main)
            .await?;

        let retrieved_ownership = ownership_repo.get_one(created_ownership.id).await?;

        assert_eq!(retrieved_ownership.unwrap(), created_ownership);

        Ok(())
    }

    #[tokio::test]
    async fn get_filtered_users() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        let test_data = initialize_test(&db).await?;
        let ownership_repo = UserCharacterOwnershipRepository::new(&db);

        let rng = rand::thread_rng();

        let user_id = test_data.2.id;
        let character_id = test_data.1.character_id;
        let ownerhash: String = rng
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        let main = true;

        let _ = ownership_repo
            .create(user_id, character_id, ownerhash.clone(), main)
            .await?;

        let filters = vec![entity::auth_user_character_ownership::Column::Main.eq(true)];

        let retrieved_users = ownership_repo.get_by_filtered(filters, 0, 1).await?;

        assert_eq!(retrieved_users.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn update_user() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        let test_data = initialize_test(&db).await?;
        let ownership_repo = UserCharacterOwnershipRepository::new(&db);
        let user_repo = UserRepository::new(&db);

        let rng = rand::thread_rng();

        let user_id = test_data.2.id;
        let character_id = test_data.1.character_id;
        let ownerhash: String = rng
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        let main = true;

        let created_ownership = ownership_repo
            .create(user_id, character_id, ownerhash.clone(), main)
            .await?;

        let new_user = user_repo.create(false).await?;

        let updated_ownership = ownership_repo
            .update(created_ownership.id, new_user.id, ownerhash.clone(), main)
            .await?;

        assert_ne!(updated_ownership, created_ownership);

        Ok(())
    }
}
