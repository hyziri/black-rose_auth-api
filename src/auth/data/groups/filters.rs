use anyhow::anyhow;
use core::panic;
use std::collections::HashSet;

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, DeleteResult,
    EntityTrait, QueryFilter,
};

use crate::{
    auth::{
        data::user::{bulk_get_user_affiliations, bulk_get_user_groups},
        model::{
            groups::{
                GroupFilterCriteria, GroupFilterCriteriaType, GroupFilterGroupDto,
                GroupFilterRuleDto, GroupFilterType, GroupFiltersDto, NewGroupDto,
                NewGroupFilterGroupDto, NewGroupFilterRuleDto, UpdateGroupFilterGroupDto,
                UpdateGroupFilterRuleDto,
            },
            user::{UserAffiliations, UserGroups},
        },
    },
    eve::data::{alliance::bulk_get_alliances, corporation::bulk_get_corporations},
};

pub async fn validate_filter_rules(
    db: &DatabaseConnection,
    rules: &Vec<NewGroupFilterRuleDto>,
) -> Result<(), anyhow::Error> {
    for rule in rules {
        match rule.criteria {
            GroupFilterCriteria::Group => {
                use crate::auth::data::groups::get_group_by_id;

                if rule.criteria_type != GroupFilterCriteriaType::Is
                    && rule.criteria_type != GroupFilterCriteriaType::IsNot
                {
                    return Err(anyhow!(
                        "Invalid criteria type for group filter, must be either 'is' or 'is not'"
                    ));
                };

                let group_id: i32 = match rule.criteria_value.parse::<i32>() {
                    Ok(id) => id,
                    Err(_) => return Err(anyhow!("Invalid group id: {}", rule.criteria_value)),
                };

                match get_group_by_id(db, group_id).await? {
                    Some(_) => (),
                    None => return Err(anyhow!("Group not found: {}", group_id)),
                }
            }
            GroupFilterCriteria::Corporation => {
                use crate::eve::data::corporation::create_corporation;

                if rule.criteria_type != GroupFilterCriteriaType::Is
                    && rule.criteria_type != GroupFilterCriteriaType::IsNot
                {
                    return Err(anyhow!(
                        "Invalid criteria type for group filter, must be either 'is' or 'is not'"
                    ));
                };

                let corporation_id: i32 = match rule.criteria_value.parse::<i32>() {
                    Ok(id) => id,
                    Err(_) => {
                        return Err(anyhow!("Invalid corporation id: {}", rule.criteria_value))
                    }
                };

                match create_corporation(db, corporation_id).await {
                    Ok(_) => (),
                    Err(err) => {
                        if err.is::<reqwest::Error>() {
                            return Err(anyhow!("Corporation not found: {}", rule.criteria_value));
                        }

                        return Err(err);
                    }
                };
            }
            GroupFilterCriteria::Alliance => {
                use crate::eve::data::alliance::create_alliance;

                if rule.criteria_type != GroupFilterCriteriaType::Is
                    && rule.criteria_type != GroupFilterCriteriaType::IsNot
                {
                    return Err(anyhow!(
                        "Invalid criteria type for group filter, must be either 'is' or 'is not'"
                    ));
                };

                let alliance_id: i32 = match rule.criteria_value.parse::<i32>() {
                    Ok(id) => id,
                    Err(_) => return Err(anyhow!("Invalid alliance id: {}", rule.criteria_value)),
                };

                match create_alliance(db, alliance_id).await {
                    Ok(_) => (),
                    Err(err) => {
                        if err.is::<reqwest::Error>() {
                            return Err(anyhow!("Alliance not found: {}", rule.criteria_value));
                        }

                        return Err(err);
                    }
                };
            }
            GroupFilterCriteria::Role => {
                if rule.criteria_type != GroupFilterCriteriaType::Is
                    && rule.criteria_type != GroupFilterCriteriaType::IsNot
                {
                    return Err(anyhow!(
                        "Invalid criteria type for group filter, must be either 'is' or 'is not'"
                    ));
                };

                if rule.criteria_value != "CEO" && rule.criteria_value != "Executor" {
                    return Err(anyhow!("Role must be set to either CEO or Executor"));
                }
            }
        }
    }

    Ok(())
}

pub async fn validate_group_filters(
    db: &DatabaseConnection,
    group: &NewGroupDto,
) -> Result<(), anyhow::Error> {
    validate_filter_rules(db, &group.filter_rules).await?;

    for filter_group in &group.filter_groups {
        validate_filter_rules(db, &filter_group.rules).await?;
    }

    Ok(())
}

// Checks a vec of user_ids against a group's filters and returns a vec of user_ids that are eligible to be in the group
pub async fn validate_group_members(
    db: &DatabaseConnection,
    group_id: i32,
    user_ids: Vec<i32>,
) -> Result<Vec<i32>, DbErr> {
    let filters = get_group_filters(db, group_id).await?;

    let mut result: Vec<HashSet<i32>> = vec![];
    struct RuleSet {
        filter_type: GroupFilterType,
        rules: Vec<GroupFilterRuleDto>,
    }

    let mut filter_groups = vec![];

    if let Some(ref filters) = filters {
        if filters.filter_rules.is_empty() && filters.filter_groups.is_empty() {
            result.push(user_ids.clone().into_iter().collect());
        }

        filter_groups = filters
            .filter_groups
            .iter()
            .map(|group| RuleSet {
                filter_type: group.filter_type.clone(),
                rules: group.rules.clone(),
            })
            .collect();

        filter_groups.push(RuleSet {
            filter_type: filters.filter_type.clone(),
            rules: filters.filter_rules.clone(),
        });
    }

    let mut user_affiliation: Vec<UserAffiliations> = vec![];
    let mut user_groups: Vec<UserGroups> = vec![];
    let mut corporation_ids: Vec<i32> = vec![];
    let mut corporations = vec![];
    let mut executor_ids: Vec<i32> = vec![];

    for group in filter_groups {
        let mut eligible_users: HashSet<i32> = HashSet::new();

        if group.filter_type == GroupFilterType::All {
            eligible_users.extend(user_ids.clone());
        }

        for filter in group.rules {
            let users: Vec<(i32, bool)> = match filter.criteria {
                GroupFilterCriteria::Group => {
                    if user_groups.is_empty() {
                        user_groups = bulk_get_user_groups(db, user_ids.clone()).await?;
                    }

                    let group_id: i32 = filter.criteria_value.parse::<i32>().unwrap_or_else(|_| panic!("Filter rule saved incorrectly, invalid criteria value insterted for filter rule {}",
                        filter.id));

                    user_groups
                        .iter()
                        .map(|user| (user.user_id, user.groups.contains(&group_id)))
                        .collect()
                }
                GroupFilterCriteria::Corporation => {
                    if user_affiliation.is_empty() {
                        user_affiliation = bulk_get_user_affiliations(db, user_ids.clone()).await?;
                    }

                    let corporation_id = filter.criteria_value.parse::<i32>().unwrap_or_else(|_| panic!("Filter rule saved incorrectly, invalid criteria value insterted for filter rule {}",
                        filter.id));

                    user_affiliation
                        .iter()
                        .map(|user| (user.user_id, user.corporations.contains(&corporation_id)))
                        .collect()
                }
                GroupFilterCriteria::Alliance => {
                    if user_affiliation.is_empty() {
                        user_affiliation = bulk_get_user_affiliations(db, user_ids.clone()).await?;
                    }

                    let alliance_id = filter.criteria_value.parse::<i32>().unwrap_or_else(|_| panic!("Filter rule saved incorrectly, invalid criteria value insterted for filter rule {}",
                        filter.id));

                    user_affiliation
                        .iter()
                        .map(|user| (user.user_id, user.alliances.contains(&alliance_id)))
                        .collect()
                }
                GroupFilterCriteria::Role => {
                    if user_affiliation.is_empty() {
                        user_affiliation = bulk_get_user_affiliations(db, user_ids.clone()).await?;
                    }

                    if corporation_ids.is_empty() {
                        corporation_ids = user_affiliation
                            .iter()
                            .flat_map(|affiliation| affiliation.corporations.clone())
                            .collect();
                    }

                    if corporations.is_empty() {
                        corporations = bulk_get_corporations(db, corporation_ids.clone()).await?;
                    }

                    let leadership_ids: Vec<i32> = match filter.criteria_value.as_str() {
                        "CEO" => {
                            corporations.iter()
                            .map(|corporation| corporation.ceo)
                            .collect()
                        }
                        "Executor" => {
                            if executor_ids.is_empty() {
                                let alliance_ids = corporations.iter()
                                    .filter_map(|corp| corp.alliance_id)
                                    .collect();

                                executor_ids = bulk_get_alliances(db, alliance_ids).await?.iter()
                                    .filter_map(|alliance| alliance.executor)
                                    .collect();
                            }

                            corporations
                            .iter()
                            .filter(|corp| executor_ids.contains(&corp.corporation_id))
                            .map(|corp| corp.ceo)
                            .collect()
                        }
                        _ => panic!("{}", format!("Filter rule saved incorrectly, invalid criteria value insterted for filter rule {}", filter.id))
                    };

                    user_affiliation
                        .iter()
                        .map(|user| {
                            (
                                user.user_id,
                                user.characters.iter().any(|id| leadership_ids.contains(id)),
                            )
                        })
                        .collect()
                }
            };

            for user in users {
                match (&group.filter_type, &filter.criteria_type, user.1) {
                    (GroupFilterType::All, GroupFilterCriteriaType::Is, false) => {
                        eligible_users.remove(&user.0);
                    }
                    (GroupFilterType::All, GroupFilterCriteriaType::IsNot, true) => {
                        eligible_users.remove(&user.0);
                    }
                    (GroupFilterType::Any, GroupFilterCriteriaType::Is, true) => {
                        eligible_users.insert(user.0);
                    }
                    (GroupFilterType::Any, GroupFilterCriteriaType::IsNot, false) => {
                        eligible_users.insert(user.0);
                    }
                    _ => (),
                }
            }
        }

        result.push(eligible_users);
    }

    let eligible_users = match filters {
        Some(filters) => match filters.filter_type {
            // If ALL filters are met add to new_members
            GroupFilterType::All => result.iter().skip(1).fold(result[0].clone(), |acc, set| {
                acc.intersection(set).cloned().collect::<HashSet<i32>>()
            }),
            // If ANY filters are met add to new_members
            GroupFilterType::Any => result.iter().skip(1).fold(result[0].clone(), |acc, set| {
                acc.union(set).cloned().collect::<HashSet<i32>>()
            }),
        }
        .into_iter()
        .collect::<Vec<i32>>(),
        // If no filters then add all users to new_members
        None => result[0].clone().into_iter().collect::<Vec<i32>>(),
    };

    Ok(eligible_users)
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

pub async fn get_group_filters(
    db: &DatabaseConnection,
    id: i32,
) -> Result<Option<GroupFiltersDto>, DbErr> {
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

            let result = GroupFiltersDto {
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
