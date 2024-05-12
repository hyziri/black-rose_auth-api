pub mod filters;

use anyhow::anyhow;
use migration::OnConflict;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    InsertResult, QueryFilter, TryInsertResult,
};

use crate::{
    auth::{
        data::groups::filters::validate_group_members,
        model::{
            groups::{NewGroupDto, UpdateGroupDto},
            user::UserDto,
        },
    },
    eve::data::character::bulk_get_characters,
};

use entity::{
    auth_group::Model as Group,
    sea_orm_active_enums::{GroupApplicationType, GroupType},
};

use filters::validate_group_filters;

use self::filters::{
    bulk_create_filter_rules, create_filter_groups, delete_filter_groups, delete_filter_rules,
    update_filter_groups, update_filter_rules,
};

use super::user::bulk_get_user_main_characters;

pub async fn create_group(
    db: &DatabaseConnection,
    new_group: NewGroupDto,
) -> Result<Group, anyhow::Error> {
    match validate_group_filters(db, &new_group).await {
        Ok(_) => (),
        Err(err) => {
            if err.is::<sea_orm::DbErr>() {
                return Err(err);
            }

            return Err(err);
        }
    }

    let group = entity::auth_group::ActiveModel {
        name: Set(new_group.name),
        description: Set(new_group.description),
        confidential: Set(new_group.confidential),
        group_type: Set(new_group.group_type.into()),
        filter_type: Set(new_group.filter_type.into()),
        ..Default::default()
    };

    let group = group.insert(db).await?;

    create_filter_groups(db, group.id, new_group.filter_groups).await?;
    bulk_create_filter_rules(db, group.id, None, new_group.filter_rules).await?;

    // Queue update group members task

    Ok(group)
}

pub async fn get_groups(db: &DatabaseConnection) -> Result<Vec<Group>, DbErr> {
    entity::prelude::AuthGroup::find().all(db).await
}

pub async fn get_group_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Group>, DbErr> {
    entity::prelude::AuthGroup::find()
        .filter(entity::auth_group::Column::Id.eq(id))
        .one(db)
        .await
}

pub async fn bulk_get_groups_by_id(
    db: &DatabaseConnection,
    ids: Vec<i32>,
) -> Result<Vec<Group>, DbErr> {
    entity::prelude::AuthGroup::find()
        .filter(entity::auth_group::Column::Id.is_in(ids))
        .all(db)
        .await
}

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

pub async fn update_group(
    db: &DatabaseConnection,
    id: i32,
    group: UpdateGroupDto,
) -> Result<Group, anyhow::Error> {
    match validate_group_filters(db, &group.clone().into()).await {
        Ok(_) => (),
        Err(err) => {
            if err.is::<sea_orm::DbErr>() {
                return Err(err);
            }

            return Err(err);
        }
    }

    let updated_group = entity::auth_group::ActiveModel {
        id: Set(id),
        name: Set(group.name),
        description: Set(group.description),
        confidential: Set(group.confidential),
        group_type: Set(group.group_type.into()),
        filter_type: Set(group.filter_type.into()),
    };

    let updated_group = updated_group.update(db).await?;

    update_filter_rules(db, id, None, group.filter_rules).await?;
    update_filter_groups(db, id, group.filter_groups).await?;

    // Queue update group members task

    Ok(updated_group)
}

pub async fn join_group(
    db: &DatabaseConnection,
    group_id: i32,
    user_id: i32,
    application_text: Option<String>,
) -> Result<String, anyhow::Error> {
    let group = match get_group_by_id(db, group_id).await? {
        Some(group) => group,
        None => return Err(anyhow!("Group does not exist")),
    };

    match group.group_type {
        GroupType::Open | GroupType::Auto => {
            let result = add_group_members(db, group_id, vec![user_id]).await?;

            match result {
                TryInsertResult::Conflicted => Err(anyhow!("Already a member")),
                _ => Ok("Successfully joined group".to_string()),
            }
        }
        GroupType::Apply | GroupType::Hidden => {
            let filter_result = validate_group_members(db, group_id, vec![user_id]).await?;

            if filter_result.is_empty() {
                return Err(anyhow!("User does not meet group requirements"));
            }

            let application = entity::auth_group_application::ActiveModel {
                group_id: Set(group_id),
                user_id: Set(user_id),
                application_type: Set(GroupApplicationType::JoinRequest),
                application_text: Set(application_text),
                ..Default::default()
            };

            let result = entity::prelude::AuthGroupApplication::insert(application)
                .on_empty_do_nothing()
                .on_conflict(
                    OnConflict::columns(vec![
                        entity::auth_group_application::Column::GroupId,
                        entity::auth_group_application::Column::UserId,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec(db)
                .await?;

            match result {
                TryInsertResult::Conflicted => Err(anyhow!("Application already exists")),
                _ => Ok("Application submitted".to_string()),
            }
        }
    }
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

pub async fn delete_group_members(
    db: &DatabaseConnection,
    group_id: i32,
    user_ids: Vec<i32>,
) -> Result<u64, DbErr> {
    let result = entity::prelude::AuthGroupUser::delete_many()
        .filter(entity::auth_group_user::Column::GroupId.eq(group_id))
        .filter(entity::auth_group_user::Column::UserId.is_in(user_ids))
        .exec(db)
        .await?;

    Ok(result.rows_affected)
}

pub async fn delete_group(db: &DatabaseConnection, group_id: i32) -> Result<Option<i32>, DbErr> {
    let group = entity::auth_group::ActiveModel {
        id: Set(group_id),
        ..Default::default()
    };

    let _ = delete_filter_rules(db, group_id).await;
    let _ = delete_filter_groups(db, group_id).await;

    let result = entity::prelude::AuthGroup::delete(group).exec(db).await?;

    if result.rows_affected == 1 {
        Ok(Some(group_id))
    } else {
        Ok(None)
    }
}
