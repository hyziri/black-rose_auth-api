use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, DeleteResult,
    EntityTrait, QueryFilter,
};

use crate::auth::model::groups::{
    GroupFilterGroupDto, GroupFilters, NewGroupDto, NewGroupFilterGroupDto, NewGroupFilterRuleDto,
    UpdateGroupDto, UpdateGroupFilterGroupDto, UpdateGroupFilterRuleDto,
};

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

    // Prevent creation of duplicate rules

    create_filter_groups(db, group.id, new_group.filter_groups).await?;
    bulk_create_filter_rules(db, group.id, None, new_group.filter_rules).await?;

    // if group type is auto find all people who meet filters and add to group

    Ok(group)
}

pub async fn create_filter_groups(
    db: &DatabaseConnection,
    group_id: i32,
    filter_groups: Vec<NewGroupFilterGroupDto>,
) -> Result<(), DbErr> {
    for group in filter_groups {
        let new_group = entity::auth_group_filter_group::ActiveModel {
            group_id: Set(group_id),
            filter_type: Set(group.filter_type.into()),
            ..Default::default()
        };

        let filter_group = new_group.insert(db).await?;

        let _ = bulk_create_filter_rules(db, group_id, Some(filter_group.id), group.rules).await;
    }

    Ok(())
}

pub async fn bulk_create_filter_rules(
    db: &DatabaseConnection,
    group_id: i32,
    filter_group: Option<i32>,
    rules: Vec<NewGroupFilterRuleDto>,
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

pub async fn get_group_filters(
    db: &DatabaseConnection,
    id: i32,
) -> Result<Option<GroupFilters>, DbErr> {
    let group = entity::prelude::AuthGroup::find()
        .filter(entity::auth_group::Column::Id.eq(id))
        .one(db)
        .await?;

    match group {
        Some(group) => {
            let filter_rules = entity::prelude::AuthGroupFilterRule::find()
                .filter(entity::auth_group_filter_rule::Column::GroupId.eq(id))
                .filter(entity::auth_group_filter_rule::Column::FilterGroupId.is_null())
                .all(db)
                .await?;

            let filter_groups = entity::prelude::AuthGroupFilterGroup::find()
                .filter(entity::auth_group_filter_group::Column::GroupId.eq(id))
                .all(db)
                .await?;

            let mut groups: Vec<GroupFilterGroupDto> = vec![];

            for group in filter_groups {
                let rules = entity::prelude::AuthGroupFilterRule::find()
                    .filter(entity::auth_group_filter_rule::Column::GroupId.eq(id))
                    .filter(entity::auth_group_filter_rule::Column::FilterGroupId.eq(group.id))
                    .all(db)
                    .await?;

                let group = GroupFilterGroupDto {
                    id: group.id,
                    filter_type: group.filter_type.into(),
                    rules: rules.into_iter().map(|rule| rule.into()).collect(),
                };

                groups.push(group)
            }

            let result = GroupFilters {
                id: group.id,
                filter_type: group.filter_type.into(),
                filter_rules: filter_rules.into_iter().map(|rule| rule.into()).collect(),
                filter_groups: groups,
            };

            Ok(Some(result))
        }
        None => Ok(None),
    }
}

pub async fn update_group(
    db: &DatabaseConnection,
    id: i32,
    group: UpdateGroupDto,
) -> Result<Group, DbErr> {
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

    Ok(updated_group)
}

pub async fn update_filter_groups(
    db: &DatabaseConnection,
    group_id: i32,
    groups: Vec<UpdateGroupFilterGroupDto>,
) -> Result<(), DbErr> {
    let group_ids: Vec<i32> = groups
        .clone()
        .into_iter()
        .filter_map(|group| group.id)
        .collect();

    entity::prelude::AuthGroupFilterGroup::delete_many()
        .filter(entity::auth_group_filter_group::Column::GroupId.eq(group_id))
        .filter(entity::auth_group_filter_group::Column::Id.is_not_in(group_ids))
        .exec(db)
        .await
        .unwrap();

    for group in groups {
        if let Some(filter_group_id) = group.id {
            let updated_filter_group: entity::auth_group_filter_group::ActiveModel =
                entity::auth_group_filter_group::ActiveModel {
                    id: Set(filter_group_id),
                    filter_type: Set(group.filter_type.into()),
                    ..Default::default()
                };

            updated_filter_group.update(db).await?;

            update_filter_rules(db, group_id, Some(filter_group_id), group.rules).await?;
        } else {
            let new_filter_group = entity::auth_group_filter_group::ActiveModel {
                group_id: Set(group_id),
                filter_type: Set(group.filter_type.into()),
                ..Default::default()
            };

            let new_filter_group = new_filter_group.insert(db).await?;

            update_filter_rules(db, group_id, Some(new_filter_group.id), group.rules).await?;
        }
    }

    Ok(())
}

pub async fn update_filter_rules(
    db: &DatabaseConnection,
    group_id: i32,
    filter_group_id: Option<i32>,
    rules: Vec<UpdateGroupFilterRuleDto>,
) -> Result<(), DbErr> {
    let rule_ids: Vec<i32> = rules
        .clone()
        .into_iter()
        .filter_map(|rule| rule.id)
        .collect();

    // This may delete filter group rules? May be fine if it is feeding filter group id = null.
    entity::prelude::AuthGroupFilterRule::delete_many()
        .filter(entity::auth_group_filter_rule::Column::GroupId.eq(group_id))
        .filter(entity::auth_group_filter_rule::Column::FilterGroupId.eq(filter_group_id))
        .filter(entity::auth_group_filter_rule::Column::Id.is_not_in(rule_ids))
        .exec(db)
        .await?;

    let mut new_rules: Vec<entity::auth_group_filter_rule::ActiveModel> = vec![];

    for rule in rules {
        if let Some(id) = rule.id {
            let updated_rule = entity::auth_group_filter_rule::ActiveModel {
                id: Set(id),
                criteria: Set(rule.criteria.into()),
                criteria_type: Set(rule.criteria_type.into()),
                criteria_value: Set(rule.criteria_value),
                ..Default::default()
            };

            updated_rule.update(db).await?;
        } else {
            let new_rule = entity::auth_group_filter_rule::ActiveModel {
                group_id: Set(group_id),
                filter_group_id: Set(filter_group_id),
                criteria: Set(rule.criteria.into()),
                criteria_type: Set(rule.criteria_type.into()),
                criteria_value: Set(rule.criteria_value),
                ..Default::default()
            };

            new_rules.push(new_rule)
        }
    }

    entity::prelude::AuthGroupFilterRule::insert_many(new_rules)
        .exec(db)
        .await?;

    Ok(())
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

pub async fn delete_filter_groups(
    db: &DatabaseConnection,
    group_id: i32,
) -> Result<DeleteResult, DbErr> {
    entity::prelude::AuthGroupFilterGroup::delete_many()
        .filter(entity::auth_group_filter_group::Column::GroupId.eq(group_id))
        .exec(db)
        .await
}

pub async fn delete_filter_rules(
    db: &DatabaseConnection,
    group_id: i32,
) -> Result<DeleteResult, DbErr> {
    entity::prelude::AuthGroupFilterRule::delete_many()
        .filter(entity::auth_group_filter_rule::Column::GroupId.eq(group_id))
        .exec(db)
        .await
}
