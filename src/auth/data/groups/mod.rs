pub mod filters;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
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

use entity::auth_group::Model as Group;

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

pub async fn add_group_members(
    db: &DatabaseConnection,
    group_id: i32,
    user_ids: Vec<i32>,
) -> Result<Vec<i32>, DbErr> {
    // ensure group exists

    let new_members = validate_group_members(db, group_id, user_ids).await?;

    let models: Vec<entity::auth_group_user::ActiveModel> = new_members
        .clone()
        .into_iter()
        .map(|user_id| entity::auth_group_user::ActiveModel {
            group_id: Set(group_id),
            user_id: Set(user_id),
            ..Default::default()
        })
        .collect();

    for group in models {
        group.insert(db).await?;
    }

    Ok(new_members)
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
