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
