use core::panic;
use std::collections::HashSet;

use eve_esi::character;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, DeleteResult,
    EntityTrait, QueryFilter,
};

use crate::{
    auth::{
        data::user::{
            self, bulk_get_character_ownerships, bulk_get_user_affiliations, bulk_get_user_groups,
        },
        model::groups::{
            GroupFilterCriteriaType, GroupFilterGroupDto, GroupFilterRuleDto, GroupFilters,
            NewGroupDto, NewGroupFilterGroupDto, NewGroupFilterRuleDto, UpdateGroupDto,
            UpdateGroupFilterGroupDto, UpdateGroupFilterRuleDto,
        },
    },
    eve::data::{
        alliance::bulk_get_alliances, character::bulk_get_character_affiliations,
        corporation::bulk_get_corporations,
    },
};

use entity::{
    auth_group::Model as Group,
    sea_orm_active_enums::{GroupFilterCriteria, GroupFilterType},
};

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
    bulk_create_filter_rules(db, group.id, None, new_group.filter_rules).await?;

    // Queue update group members task

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

// Returns vec of character_ids added to the group
pub async fn update_membership(
    db: &DatabaseConnection,
    group_id: i32,
    user_ids: Vec<i32>,
) -> Result<(), DbErr> {
    async fn check_filters(
        db: &DatabaseConnection,
        rules: Vec<GroupFilterRuleDto>,
        filter_type: GroupFilterType,
        user_ids: Vec<i32>,
    ) -> Result<(), DbErr> {
        fn handle_affiliation_check(
            eligible_users: HashSet<i32>,
            filter_type: &GroupFilterType,
            filter: &GroupFilterRuleDto,
            ids_to_check: &Vec<i32>,
            criteria_id: i32,
            user_id: i32,
        ) -> HashSet<i32> {
            let mut eligible_users = eligible_users;

            match filter.criteria_type {
                GroupFilterCriteriaType::Is => {
                    if *filter_type == GroupFilterType::Any && ids_to_check.contains(&criteria_id) {
                        eligible_users.insert(user_id);
                    } else if *filter_type == GroupFilterType::All
                        && !ids_to_check.contains(&criteria_id)
                    {
                        eligible_users.remove(&user_id);
                    }
                }
                GroupFilterCriteriaType::IsNot => {
                    if *filter_type == GroupFilterType::Any && !ids_to_check.contains(&criteria_id)
                    {
                        eligible_users.insert(user_id);
                    } else if *filter_type == GroupFilterType::All
                        && ids_to_check.contains(&criteria_id)
                    {
                        eligible_users.remove(&user_id);
                    }
                }
                _ => {
                    panic!("Filter rule saved incorrectly, invalid criteria type inserted for filter rule {}", filter.id);
                }
            }

            eligible_users
        }

        let mut eligible_users: HashSet<i32> = HashSet::new();

        let mut user_affiliation = vec![];
        let mut user_groups = vec![];

        if filter_type == GroupFilterType::All {
            eligible_users.extend(user_ids.clone());
        }

        for filter in rules {
            match filter.criteria.into() {
                GroupFilterCriteria::Group => {
                    if user_groups.is_empty() {
                        user_groups = bulk_get_user_groups(db, user_ids.clone()).await?;
                    }

                    let group_id: i32 = filter.criteria_value.parse::<i32>().expect(&format!(
                        "Filter rule saved incorrectly, invalid criteria value insterted for filter rule {}",
                        filter.id
                    ));

                    for user_group in user_groups.iter() {
                        eligible_users = handle_affiliation_check(
                            eligible_users,
                            &filter_type,
                            &filter,
                            &user_group.groups,
                            group_id,
                            user_group.user_id,
                        );
                    }
                }
                GroupFilterCriteria::Corporation => {
                    if user_affiliation.is_empty() {
                        user_affiliation = bulk_get_user_affiliations(db, user_ids.clone()).await?;
                    }

                    let corporation_id = filter.criteria_value.parse::<i32>().expect(&format!(
                        "Filter rule saved incorrectly, invalid criteria value insterted for filter rule {}",
                        filter.id
                    ));

                    for affiliation in user_affiliation.iter() {
                        eligible_users = handle_affiliation_check(
                            eligible_users,
                            &filter_type,
                            &filter,
                            &affiliation.corporations,
                            corporation_id,
                            affiliation.user_id,
                        );
                    }
                }
                GroupFilterCriteria::Alliance => {
                    if user_affiliation.is_empty() {
                        user_affiliation = bulk_get_user_affiliations(db, user_ids.clone()).await?;
                    }

                    let alliance_id = filter.criteria_value.parse::<i32>().expect(&format!(
                        "Filter rule saved incorrectly, invalid criteria value insterted for filter rule {}",
                        filter.id
                    ));

                    for affiliation in user_affiliation.iter() {
                        eligible_users = handle_affiliation_check(
                            eligible_users,
                            &filter_type,
                            &filter,
                            &affiliation.alliances,
                            alliance_id,
                            affiliation.user_id,
                        );
                    }
                }
                GroupFilterCriteria::Role => {
                    if user_affiliation.is_empty() {
                        user_affiliation = bulk_get_user_affiliations(db, user_ids.clone()).await?;
                    }

                    let corporation_ids = user_affiliation
                        .iter()
                        .flat_map(|affiliation| affiliation.corporations.clone())
                        .collect::<Vec<i32>>();

                    let ceo_ids = bulk_get_corporations(db, corporation_ids.clone())
                        .await?
                        .iter()
                        .map(|corporation| corporation.ceo)
                        .collect::<Vec<i32>>();

                    let mut executor_ids = vec![];

                    for affiliation in user_affiliation.iter() {
                        if filter.criteria_value == "CEO" {
                            if filter_type == GroupFilterType::Any
                                && affiliation.characters.iter().any(|id| ceo_ids.contains(id))
                            {
                                eligible_users.insert(affiliation.user_id);
                            } else if filter_type == GroupFilterType::All {
                                eligible_users.remove(&affiliation.user_id);
                            }
                        } else if filter.criteria_value == "Executor" {
                            if executor_ids.is_empty() {
                                executor_ids = bulk_get_alliances(db, corporation_ids.clone())
                                    .await?
                                    .iter()
                                    .filter_map(|alliance: &entity::eve_alliance::Model| {
                                        alliance.executor
                                    })
                                    .collect::<Vec<i32>>();
                            }

                            match filter.criteria_type {
                                GroupFilterCriteriaType::Is => {
                                    if filter_type == GroupFilterType::Any
                                        && affiliation
                                            .characters
                                            .iter()
                                            .any(|id| executor_ids.contains(id))
                                    {
                                        eligible_users.insert(affiliation.user_id);
                                    } else if filter_type == GroupFilterType::All
                                        && !affiliation
                                            .characters
                                            .iter()
                                            .any(|id| executor_ids.contains(id))
                                    {
                                        eligible_users.remove(&affiliation.user_id);
                                    }
                                }
                                GroupFilterCriteriaType::IsNot => {
                                    if filter_type == GroupFilterType::Any
                                        && !affiliation
                                            .characters
                                            .iter()
                                            .any(|id| executor_ids.contains(id))
                                    {
                                        eligible_users.insert(affiliation.user_id);
                                    } else if filter_type == GroupFilterType::All
                                        && affiliation
                                            .characters
                                            .iter()
                                            .any(|id| executor_ids.contains(id))
                                    {
                                        eligible_users.remove(&affiliation.user_id);
                                    }
                                }
                                _ => {
                                    panic!("Filter rule saved incorrectly, invalid criteria type inserted for filter rule {}", filter.id);
                                }
                            }
                        } else {
                            panic!("Filter rule saved incorrectly, invalid criteria value insterted for filter rule {}", filter.id);
                        }
                    }
                }
            }
        }

        // Group
        // Corporation
        // Alliance

        Ok(())
    }

    let filters = get_group_filters(db, group_id).await?;

    // Check rules
    // Check filter group rules

    Ok(())
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

    // Queue update group members task

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
