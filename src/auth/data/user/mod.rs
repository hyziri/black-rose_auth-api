pub mod ownership;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use sea_orm::{DbErr, PaginatorTrait};
use std::collections::HashMap;

use entity::auth_user::Model as User;
use entity::auth_user_character_ownership::Model as UserCharacterOwnership;
use entity::prelude::AuthUser;

use crate::auth::model::user::{UserAffiliations, UserGroups};
use crate::eve::service::affiliation::get_character_affiliations;

pub struct UserRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self, admin: bool) -> Result<User, sea_orm::DbErr> {
        let user = entity::auth_user::ActiveModel {
            admin: Set(admin),
            created: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        user.insert(self.db).await
    }

    pub async fn get_one(&self, id: i32) -> Result<Option<User>, sea_orm::DbErr> {
        AuthUser::find_by_id(id).one(self.db).await
    }

    pub async fn get_by_filtered(
        &self,
        filters: Vec<migration::SimpleExpr>,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<User>, sea_orm::DbErr> {
        let mut query = AuthUser::find();

        for filter in filters {
            query = query.filter(filter);
        }

        query.paginate(self.db, page_size).fetch_page(page).await
    }

    pub async fn update(&self, user_id: i32, admin: bool) -> Result<User, sea_orm::DbErr> {
        let user = self.get_one(user_id).await?;

        match user {
            Some(user) => {
                let mut user: entity::auth_user::ActiveModel = user.into();

                user.admin = Set(admin);

                user.update(self.db).await
            }
            None => Err(sea_orm::DbErr::RecordNotFound(format!(
                "User with id {} not found",
                user_id
            ))),
        }
    }
}

pub async fn create_user(db: &DatabaseConnection) -> Result<i32, DbErr> {
    let user = entity::auth_user::ActiveModel {
        admin: Set(false),
        created: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    let user: User = user.insert(db).await?;

    Ok(user.id)
}

pub async fn get_user(db: &DatabaseConnection, user_id: i32) -> Result<Option<User>, DbErr> {
    entity::prelude::AuthUser::find()
        .filter(entity::auth_user::Column::Id.eq(user_id))
        .one(db)
        .await
}

pub async fn get_user_main_character(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<Option<UserCharacterOwnership>, DbErr> {
    let characters = get_user_character_ownerships(db, user_id).await?;

    if characters.is_empty() {
        return Ok(None);
    }

    let main_character_ownership = characters.into_iter().find(|ownership| ownership.main);

    match main_character_ownership {
        Some(main_character_ownership) => Ok(Some(main_character_ownership)),
        None => Ok(None),
    }
}

pub async fn update_ownership(
    db: &DatabaseConnection,
    user_id: i32,
    character_id: i32,
    ownerhash: String,
) -> Result<UserCharacterOwnership, DbErr> {
    let existing_ownership = get_character_ownership(db, character_id).await?;

    match existing_ownership {
        Some(existing_ownership) => {
            if (existing_ownership.ownerhash == ownerhash)
                && (existing_ownership.user_id == user_id)
            {
                return Ok(existing_ownership);
            }

            let owned_characters =
                get_user_character_ownerships(db, existing_ownership.user_id).await?;

            if owned_characters.len() > 1 && existing_ownership.main {
                let character = owned_characters.iter().find(|&character| !character.main);

                if let Some(character) = character {
                    let mut character: entity::auth_user_character_ownership::ActiveModel =
                        character.clone().into();

                    character.main = Set(true.to_owned());

                    let _ = character.update(db).await?;
                }
            }

            let main = get_user_character_ownerships(db, user_id).await?.is_empty();

            let mut existing_ownership: entity::auth_user_character_ownership::ActiveModel =
                existing_ownership.into();

            existing_ownership.user_id = Set(user_id);
            existing_ownership.ownerhash = Set(ownerhash);
            existing_ownership.main = Set(main);

            Ok(existing_ownership.update(db).await?)
        }
        None => {
            let main = get_user_character_ownerships(db, user_id).await?.is_empty();

            let new_ownership = entity::auth_user_character_ownership::ActiveModel {
                user_id: Set(user_id),
                character_id: Set(character_id),
                ownerhash: Set(ownerhash),
                main: Set(main),
                ..Default::default()
            };

            Ok(new_ownership.insert(db).await?)
        }
    }
}

pub async fn get_character_ownership(
    db: &DatabaseConnection,
    character_id: i32,
) -> Result<Option<UserCharacterOwnership>, DbErr> {
    let ownership: Option<UserCharacterOwnership> =
        entity::prelude::AuthUserCharacterOwnership::find()
            .filter(entity::auth_user_character_ownership::Column::CharacterId.eq(character_id))
            .one(db)
            .await?;

    Ok(ownership)
}

pub async fn get_user_character_ownerships(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<Vec<UserCharacterOwnership>, DbErr> {
    entity::prelude::AuthUserCharacterOwnership::find()
        .filter(entity::auth_user_character_ownership::Column::UserId.eq(user_id))
        .all(db)
        .await
}

pub async fn bulk_get_character_ownerships(
    db: &DatabaseConnection,
    user_ids: Vec<i32>,
) -> Result<Vec<UserCharacterOwnership>, DbErr> {
    entity::prelude::AuthUserCharacterOwnership::find()
        .filter(entity::auth_user_character_ownership::Column::UserId.is_in(user_ids))
        .all(db)
        .await
}

pub async fn get_user_character_ownership_by_ownerhash(
    db: &DatabaseConnection,
    ownerhash: String,
) -> Result<Option<UserCharacterOwnership>, DbErr> {
    entity::prelude::AuthUserCharacterOwnership::find()
        .filter(entity::auth_user_character_ownership::Column::Ownerhash.eq(ownerhash))
        .one(db)
        .await
}

pub async fn bulk_get_user_affiliations(
    db: &DatabaseConnection,
    user_ids: Vec<i32>,
) -> Result<Vec<UserAffiliations>, DbErr> {
    let ownerships = bulk_get_character_ownerships(db, user_ids).await?;
    let character_ids: Vec<i32> = ownerships.iter().map(|char| char.character_id).collect();
    let affiliations = get_character_affiliations(db, character_ids.clone()).await?;

    let mut user_affiliations: HashMap<i32, UserAffiliations> = HashMap::new();
    let ownerships_map: HashMap<i32, &entity::auth_user_character_ownership::Model> = ownerships
        .iter()
        .map(|ownership| (ownership.character_id, ownership))
        .collect();

    for ownership in &ownerships {
        user_affiliations
            .entry(ownership.user_id)
            .or_insert(UserAffiliations {
                user_id: ownership.user_id,
                characters: Vec::new(),
                corporations: Vec::new(),
                alliances: Vec::new(),
            });
    }

    for affiliation in &affiliations {
        if let Some(ownership) = ownerships_map.get(&affiliation.character_id) {
            if let Some(user_affiliation) = user_affiliations.get_mut(&ownership.user_id) {
                user_affiliation.characters.push(affiliation.character_id);
                user_affiliation
                    .corporations
                    .push(affiliation.corporation_id);
                if let Some(alliance_id) = affiliation.alliance_id {
                    user_affiliation.alliances.push(alliance_id);
                }
            }
        }
    }

    let user_affiliations = user_affiliations.into_values().collect();

    Ok(user_affiliations)
}

pub async fn bulk_get_user_groups(
    db: &DatabaseConnection,
    user_ids: Vec<i32>,
) -> Result<Vec<UserGroups>, DbErr> {
    let user_groups: Vec<entity::auth_group_user::Model> = entity::prelude::AuthGroupUser::find()
        .filter(entity::auth_group_user::Column::UserId.is_in(user_ids))
        .all(db)
        .await?;

    let mut user_groups_map: HashMap<i32, UserGroups> = HashMap::new();

    for user_group in user_groups {
        user_groups_map
            .entry(user_group.user_id)
            .or_insert_with(|| UserGroups {
                user_id: user_group.user_id,
                groups: Vec::new(),
            })
            .groups
            .push(user_group.group_id);
    }

    let user_groups: Vec<UserGroups> = user_groups_map.into_values().collect();

    Ok(user_groups)
}

pub async fn bulk_get_user_main_characters(
    db: &DatabaseConnection,
    user_ids: Vec<i32>,
) -> Result<Vec<UserCharacterOwnership>, DbErr> {
    entity::prelude::AuthUserCharacterOwnership::find()
        .filter(entity::auth_user_character_ownership::Column::UserId.is_in(user_ids))
        .filter(entity::auth_user_character_ownership::Column::Main.eq(true))
        .all(db)
        .await
}

pub async fn update_user_main(
    db: &DatabaseConnection,
    character_id: i32,
) -> Result<Option<UserCharacterOwnership>, DbErr> {
    let get_new_main = get_character_ownership(db, character_id).await?;

    if let Some(new_main) = get_new_main {
        let get_old_main = entity::prelude::AuthUserCharacterOwnership::find()
            .filter(entity::auth_user_character_ownership::Column::UserId.eq(new_main.user_id))
            .filter(entity::auth_user_character_ownership::Column::Main.eq(true))
            .one(db)
            .await?;

        let mut new_main: entity::auth_user_character_ownership::ActiveModel = new_main.into();
        new_main.main = Set(true);

        if let Some(old_main) = get_old_main {
            let mut old_main: entity::auth_user_character_ownership::ActiveModel = old_main.into();
            old_main.main = Set(false);

            let old_main_result = old_main.update(db).await;
            if old_main_result.is_ok() {
                let new_main_result = new_main.update(db).await;

                if let Ok(new_main) = new_main_result {
                    return Ok(Some(new_main));
                }
            }
        }
    }

    Ok(None)
}

pub async fn update_user_as_admin(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<Option<User>, DbErr> {
    let user = entity::prelude::AuthUser::find_by_id(user_id)
        .one(db)
        .await?;

    match user {
        Some(user) => {
            let mut user: entity::auth_user::ActiveModel = user.into();

            user.admin = Set(true);

            let user = user.update(db).await?;

            Ok(Some(user))
        }
        None => Ok(None),
    }
}

pub async fn get_users_with_admin(db: &DatabaseConnection) -> Result<Vec<User>, DbErr> {
    entity::prelude::AuthUser::find()
        .filter(entity::auth_user::Column::Admin.eq(true))
        .all(db)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ColumnTrait, ConnectionTrait, Database, DbBackend, Schema};

    async fn initialize_test(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        let schema = Schema::new(DbBackend::Sqlite);

        let stmts = vec![schema.create_table_from_entity(entity::prelude::AuthUser)];

        for stmt in stmts {
            let _ = db.execute(db.get_database_backend().build(&stmt)).await?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn create_user() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        initialize_test(&db).await?;
        let user_repo = UserRepository::new(&db);

        let admin = true;

        let created_user = user_repo.create(admin).await?;

        assert_eq!(admin, created_user.admin);

        Ok(())
    }

    #[tokio::test]
    async fn get_one_user() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        initialize_test(&db).await?;
        let user_repo = UserRepository::new(&db);

        let admin = true;

        let created_user = user_repo.create(admin).await?;

        let retrieved_user = user_repo.get_one(created_user.id).await?;

        assert_eq!(retrieved_user.unwrap(), created_user);

        Ok(())
    }

    #[tokio::test]
    async fn get_filtered_users() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        initialize_test(&db).await?;
        let user_repo = UserRepository::new(&db);

        let mut created_users = Vec::new();

        for _ in 0..5 {
            let admin = false;

            let created_user = user_repo.create(admin).await?;

            created_users.push(created_user);
        }

        let filters = vec![entity::auth_user::Column::Id.eq(created_users[0].id)];

        let retrieved_users = user_repo.get_by_filtered(filters, 0, 5).await?;

        assert_eq!(retrieved_users.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn update_user() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        initialize_test(&db).await?;
        let user_repo = UserRepository::new(&db);

        let admin = true;

        let created_user = user_repo.create(admin).await?;

        let updated_user = user_repo.update(created_user.id, false).await?;

        assert_ne!(updated_user, created_user);

        Ok(())
    }
}
