use anyhow::anyhow;
use migration::OnConflict;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, DeleteResult,
    EntityTrait, InsertResult, QueryFilter, TryInsertResult,
};

use crate::{
    auth::{
        data::groups::filters::validate_group_members,
        model::{groups::GroupApplicationDto, user::UserDto},
    },
    eve::data::character::bulk_get_characters,
};

use entity::sea_orm_active_enums::{GroupApplicationStatus, GroupApplicationType, GroupType};

use super::{applications::get_group_application, get_group_by_id};

use crate::auth::data::user::bulk_get_user_main_characters;

pub async fn get_group_members(
    db: &DatabaseConnection,
    group_id: i32,
) -> Result<Vec<UserDto>, sea_orm::DbErr> {
    let members = entity::prelude::AuthGroupUser::find()
        .filter(entity::auth_group_user::Column::GroupId.eq(group_id))
        .all(db)
        .await?;

    let user_ids = members
        .iter()
        .map(|member| member.user_id)
        .collect::<Vec<i32>>();

    let ownerships = bulk_get_user_main_characters(db, user_ids).await?;
    let character_ids = ownerships
        .iter()
        .map(|user| user.character_id)
        .collect::<Vec<i32>>();

    let characters = bulk_get_characters(db, character_ids).await?;

    let characters = characters
        .iter()
        .filter_map(|character| {
            ownerships
                .iter()
                .find(|&model| model.character_id == character.character_id)
                .map(|model| model.user_id)
                .map(|user_id| UserDto {
                    id: user_id,
                    character_name: character.character_name.clone(),
                    character_id: character.character_id,
                })
        })
        .collect::<Vec<UserDto>>();

    Ok(characters)
}

pub async fn add_group_members(
    db: &DatabaseConnection,
    group_id: i32,
    user_ids: Vec<i32>,
) -> Result<TryInsertResult<InsertResult<entity::auth_group_user::ActiveModel>>, anyhow::Error> {
    let _ = match get_group_by_id(db, group_id).await? {
        Some(group) => group,
        None => return Err(anyhow!("Group does not exist")),
    };

    let new_member_ids = validate_group_members(db, group_id, user_ids).await?;

    let new_members: Vec<entity::auth_group_user::ActiveModel> = new_member_ids
        .clone()
        .into_iter()
        .map(|user_id| entity::auth_group_user::ActiveModel {
            group_id: Set(group_id),
            user_id: Set(user_id),
            ..Default::default()
        })
        .collect();

    let result = entity::prelude::AuthGroupUser::insert_many(new_members)
        .on_empty_do_nothing()
        .on_conflict(
            OnConflict::columns(vec![
                entity::auth_group_user::Column::GroupId,
                entity::auth_group_user::Column::UserId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(result)
}

pub async fn join_group(
    db: &DatabaseConnection,
    group_id: i32,
    user_id: i32,
    request_message: Option<String>,
) -> Result<Option<GroupApplicationDto>, anyhow::Error> {
    let group = match get_group_by_id(db, group_id).await? {
        Some(group) => group,
        None => return Err(anyhow!("Group does not exist")),
    };

    match group.group_type {
        GroupType::Open | GroupType::Auto => {
            let result = add_group_members(db, group_id, vec![user_id]).await?;

            match result {
                TryInsertResult::Conflicted => Err(anyhow!("Already a member")),
                _ => Ok(None),
            }
        }
        GroupType::Apply | GroupType::Hidden => {
            let filter_result = validate_group_members(db, group_id, vec![user_id]).await?;

            if filter_result.is_empty() {
                return Err(anyhow!("User does not meet group requirements"));
            }

            let duplicate_application = entity::prelude::AuthGroupApplication::find()
                .filter(entity::auth_group_application::Column::GroupId.eq(group_id))
                .filter(entity::auth_group_application::Column::UserId.eq(user_id))
                .filter(
                    entity::auth_group_application::Column::Status
                        .eq(GroupApplicationStatus::Outstanding),
                )
                .filter(
                    entity::auth_group_application::Column::RequestType
                        .eq(GroupApplicationType::Join),
                )
                .one(db)
                .await?;

            if duplicate_application.is_some() {
                return Err(anyhow!("Application to join already exists"));
            }

            let application = entity::auth_group_application::ActiveModel {
                group_id: Set(group_id),
                user_id: Set(user_id),
                request_type: Set(GroupApplicationType::Join),
                request_message: Set(request_message),
                ..Default::default()
            };

            let application = application.insert(db).await?;

            let application =
                get_group_application(db, None, None, Some(application.id), None, None).await?;

            Ok(application.into_iter().next())
        }
    }
}
pub async fn delete_group_members(
    db: &DatabaseConnection,
    group_id: i32,
    user_ids: Vec<i32>,
) -> Result<DeleteResult, anyhow::Error> {
    let _ = match get_group_by_id(db, group_id).await? {
        Some(group) => group,
        None => return Err(anyhow!("Group does not exist")),
    };

    let result = entity::prelude::AuthGroupUser::delete_many()
        .filter(entity::auth_group_user::Column::GroupId.eq(group_id))
        .filter(entity::auth_group_user::Column::UserId.is_in(user_ids))
        .exec(db)
        .await?;

    Ok(result)
}

pub async fn delete_all_group_members(
    db: &DatabaseConnection,
    group_id: i32,
) -> Result<DeleteResult, DbErr> {
    let result = entity::prelude::AuthGroupUser::delete_many()
        .filter(entity::auth_group_user::Column::GroupId.eq(group_id))
        .exec(db)
        .await?;

    Ok(result)
}

pub async fn leave_group(
    db: &DatabaseConnection,
    group_id: i32,
    user_id: i32,
    request_message: Option<String>,
) -> Result<Option<GroupApplicationDto>, anyhow::Error> {
    let group = match get_group_by_id(db, group_id).await? {
        Some(group) => group,
        None => return Err(anyhow!("Group does not exist")),
    };

    let current_user = entity::prelude::AuthGroupUser::find()
        .filter(entity::auth_group_user::Column::GroupId.eq(group_id))
        .filter(entity::auth_group_user::Column::UserId.eq(user_id))
        .one(db)
        .await?;

    if current_user.is_none() {
        return Err(anyhow!("User is not a member of the group"));
    }

    if group.leave_applications {
        let duplicate_application = entity::prelude::AuthGroupApplication::find()
            .filter(entity::auth_group_application::Column::GroupId.eq(group_id))
            .filter(entity::auth_group_application::Column::UserId.eq(user_id))
            .filter(
                entity::auth_group_application::Column::Status
                    .eq(GroupApplicationStatus::Outstanding),
            )
            .filter(
                entity::auth_group_application::Column::RequestType.eq(GroupApplicationType::Leave),
            )
            .one(db)
            .await?;

        if duplicate_application.is_some() {
            return Err(anyhow!("Application to leave already exists"));
        }

        let application = entity::auth_group_application::ActiveModel {
            group_id: Set(group_id),
            user_id: Set(user_id),
            request_type: Set(GroupApplicationType::Leave),
            request_message: Set(request_message),
            ..Default::default()
        };

        let application = application.insert(db).await?;

        let application =
            get_group_application(db, None, None, Some(application.id), None, None).await?;

        Ok(application.into_iter().next())
    } else {
        let _ = delete_group_members(db, group_id, vec![user_id]).await?;

        Ok(None)
    }
}
