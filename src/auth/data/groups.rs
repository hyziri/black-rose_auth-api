use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};

use crate::auth::model::groups::{GroupFilterGroupDto, GroupFilterRuleDto, NewGroupDto};

use entity::auth_group::Model as Group;

pub async fn create_group(db: &DatabaseConnection, new_group: NewGroupDto) -> Result<Group, DbErr> {
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
    create_filter_rules(db, group.id, None, new_group.filter_rules).await?;

    Ok(group)
}

pub async fn create_filter_groups(
    db: &DatabaseConnection,
    group_id: i32,
    filter_groups: Vec<GroupFilterGroupDto>,
) -> Result<(), DbErr> {
    for group in filter_groups {
        let new_group = entity::auth_group_filter_group::ActiveModel {
            group_id: Set(group_id),
            filter_type: Set(group.filter_type.into()),
            ..Default::default()
        };

        let filter_group = new_group.insert(db).await?;

        let _ = create_filter_rules(db, group_id, Some(filter_group.id), group.rules).await;
    }

    Ok(())
}

pub async fn create_filter_rules(
    db: &DatabaseConnection,
    group_id: i32,
    filter_group: Option<i32>,
    rules: Vec<GroupFilterRuleDto>,
) -> Result<(), DbErr> {
    if rules.is_empty() {
        return Ok(());
    }

    let mut new_rules: Vec<entity::auth_group_filter_rule::ActiveModel> = vec![];

    for rule in rules {
        let new_rule = entity::auth_group_filter_rule::ActiveModel {
            group_id: Set(group_id),
            filter_group_id: Set(filter_group),
            criteria: Set(rule.criteria.into()),
            criteria_type: Set(rule.criteria_type.into()),
            criteria_value: Set(rule.criteria_value),
            ..Default::default()
        };

        new_rules.push(new_rule)
    }

    entity::prelude::AuthGroupFilterRule::insert_many(new_rules)
        .exec(db)
        .await?;

    Ok(())
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
        filter_type: Set(updated_group.filter_type.into()),
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
