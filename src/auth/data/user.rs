use sea_orm::DbErr;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use entity::auth_user::Model as User;
use entity::auth_user_character_ownership::Model as UserCharacterOwnership;

pub async fn create_user(db: &DatabaseConnection) -> Result<i32, DbErr> {
    let user = entity::auth_user::ActiveModel {
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

pub async fn get_user_character_ownership_by_ownerhash(
    db: &DatabaseConnection,
    ownerhash: String,
) -> Result<Option<UserCharacterOwnership>, DbErr> {
    entity::prelude::AuthUserCharacterOwnership::find()
        .filter(entity::auth_user_character_ownership::Column::Ownerhash.eq(ownerhash))
        .one(db)
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
