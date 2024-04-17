use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait};

use crate::auth::model::group::NewGroupDto;

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
