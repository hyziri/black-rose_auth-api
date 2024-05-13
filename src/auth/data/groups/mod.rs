pub mod applications;
pub mod filters;
pub mod members;

use anyhow::anyhow;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};

use crate::auth::model::groups::{GroupOwnerType, NewGroupDto, UpdateGroupDto};

use entity::auth_group::Model as Group;

use filters::validate_group_filters;

use self::{
    filters::{
        bulk_create_filter_rules, create_filter_groups, delete_filter_groups, delete_filter_rules,
        update_filter_groups, update_filter_rules,
    },
    members::delete_all_group_members,
};

async fn validate_group_owner(
    db: &DatabaseConnection,
    owner_type: &GroupOwnerType,
    owner_id: Option<i32>,
) -> Result<(), anyhow::Error> {
    use crate::eve::data;

    match owner_type {
        GroupOwnerType::Auth => (),
        GroupOwnerType::Alliance => {
            if let Some(owner_id) = owner_id {
                match data::alliance::create_alliance(db, owner_id).await {
                    Ok(_) => (),
                    Err(err) => {
                        if err.is::<reqwest::Error>() {
                            return Err(anyhow!("Alliance not found: {}", owner_id));
                        }

                        return Err(err);
                    }
                };
            }
        }
        GroupOwnerType::Corporation => {
            if let Some(owner_id) = owner_id {
                match data::corporation::create_corporation(db, owner_id).await {
                    Ok(_) => (),
                    Err(err) => {
                        if err.is::<reqwest::Error>() {
                            return Err(anyhow!("Corporation not found: {}", owner_id));
                        }

                        return Err(err);
                    }
                };
            }
        }
    }

    Ok(())
}

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

    validate_group_owner(db, &new_group.owner_type, new_group.owner_id).await?;

    let owner_id: Option<i32> = if new_group.owner_type == GroupOwnerType::Auth {
        None
    } else {
        new_group.owner_id
    };

    let group = entity::auth_group::ActiveModel {
        name: Set(new_group.name),
        description: Set(new_group.description),
        confidential: Set(new_group.confidential),
        leave_applications: Set(new_group.leave_applications),
        owner_type: Set(new_group.owner_type.into()),
        owner_id: Set(owner_id),
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

    validate_group_owner(db, &group.owner_type, group.owner_id).await?;

    let owner_id: Option<i32> = if group.owner_type == GroupOwnerType::Auth {
        None
    } else {
        group.owner_id
    };

    let updated_group = entity::auth_group::ActiveModel {
        id: Set(id),
        name: Set(group.name),
        description: Set(group.description),
        confidential: Set(group.confidential),
        leave_applications: Set(group.leave_applications),
        owner_type: Set(group.owner_type.into()),
        owner_id: Set(owner_id),
        group_type: Set(group.group_type.into()),
        filter_type: Set(group.filter_type.into()),
    };

    let updated_group = updated_group.update(db).await?;

    update_filter_rules(db, id, None, group.filter_rules).await?;
    update_filter_groups(db, id, group.filter_groups).await?;

    // Queue update group members task

    Ok(updated_group)
}

pub async fn delete_group(db: &DatabaseConnection, group_id: i32) -> Result<Option<i32>, DbErr> {
    let group = entity::auth_group::ActiveModel {
        id: Set(group_id),
        ..Default::default()
    };

    let _ = delete_filter_rules(db, group_id).await?;
    let _ = delete_filter_groups(db, group_id).await?;
    let _ = delete_all_group_members(db, group_id).await?;

    let result = entity::prelude::AuthGroup::delete(group).exec(db).await?;

    if result.rows_affected == 1 {
        Ok(Some(group_id))
    } else {
        Ok(None)
    }
}
