use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};

use crate::auth::model::groups::NewGroupDto;

use entity::auth_group::Model as Group;

pub async fn create_group(db: &DatabaseConnection, new_group: NewGroupDto) -> Result<Group, DbErr> {
    let new_group = entity::auth_group::ActiveModel {
        name: Set(new_group.name),
        description: Set(new_group.description),
        confidential: Set(new_group.confidential),
        group_type: Set(new_group.group_type.into()),
        ..Default::default()
    };

    let group = new_group.insert(db).await?;

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

pub async fn update_group(
    db: &DatabaseConnection,
    id: i32,
    updated_group: NewGroupDto,
) -> Result<Group, DbErr> {
    let updated_group = entity::auth_group::ActiveModel {
        id: Set(id),
        name: Set(updated_group.name),
        description: Set(updated_group.description),
        confidential: Set(updated_group.confidential),
        group_type: Set(updated_group.group_type.into()),
        ..Default::default()
    };

    updated_group.update(db).await
}

pub async fn delete_group(db: &DatabaseConnection, id: i32) -> Result<Option<i32>, DbErr> {
    let group = entity::auth_group::ActiveModel {
        id: Set(id),
        ..Default::default()
    };

    let result = entity::prelude::AuthGroup::delete(group).exec(db).await?;

    if result.rows_affected == 1 {
        Ok(Some(id))
    } else {
        Ok(None)
    }
}
