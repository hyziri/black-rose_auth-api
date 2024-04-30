use core::panic;
use std::collections::HashSet;

use sea_orm::{DatabaseConnection, DbErr, EntityTrait, Set};

use crate::{
    auth::{
        data::user::{bulk_get_user_affiliations, bulk_get_user_groups},
        model::{
            groups::{
                GroupFilterCriteria, GroupFilterCriteriaType, GroupFilterRuleDto, GroupFilterType,
            },
            user::{UserAffiliations, UserGroups},
        },
    },
    eve::data::{alliance::bulk_get_alliances, corporation::bulk_get_corporations},
};

use super::groups::get_group_filters;

pub async fn new_update_membership(
    db: &DatabaseConnection,
    group_id: i32,
    user_ids: Vec<i32>,
) -> Result<Vec<i32>, DbErr> {
    let filters = get_group_filters(db, group_id).await?;

    let mut result: Vec<HashSet<i32>> = vec![];

    if filters.is_none() {
        result.push(user_ids.clone().into_iter().collect());
    };

    struct RuleSet {
        filter_type: GroupFilterType,
        rules: Vec<GroupFilterRuleDto>,
    }

    let mut filter_groups: Vec<RuleSet> = vec![];

    if let Some(ref filters) = filters {
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
    let mut ceo_ids: Vec<i32> = vec![];
    let mut executor_ids: Vec<i32> = vec![];
    let mut corporation_ids: Vec<i32> = vec![];

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

                    let group_id: i32 = filter.criteria_value.parse::<i32>().expect(&format!(
                        "Filter rule saved incorrectly, invalid criteria value insterted for filter rule {}",
                        filter.id
                    ));

                    user_groups
                        .iter()
                        .map(|user| (user.user_id, user.groups.contains(&group_id)))
                        .collect()
                }
                GroupFilterCriteria::Corporation => {
                    if user_affiliation.is_empty() {
                        user_affiliation = bulk_get_user_affiliations(db, user_ids.clone()).await?;
                    }

                    let corporation_id = filter.criteria_value.parse::<i32>().expect(&format!(
                        "Filter rule saved incorrectly, invalid criteria value insterted for filter rule {}",
                        filter.id
                    ));

                    user_affiliation
                        .iter()
                        .map(|user| (user.user_id, user.corporations.contains(&corporation_id)))
                        .collect()
                }
                GroupFilterCriteria::Alliance => {
                    if user_affiliation.is_empty() {
                        user_affiliation = bulk_get_user_affiliations(db, user_ids.clone()).await?;
                    }

                    let alliance_id = filter.criteria_value.parse::<i32>().expect(&format!(
                        "Filter rule saved incorrectly, invalid criteria value insterted for filter rule {}",
                        filter.id
                    ));

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

                    let leadership_ids = match filter.criteria_value.as_str() {
                        "CEO" => {
                            if ceo_ids.is_empty() {
                                ceo_ids = bulk_get_corporations(db, corporation_ids.clone())
                                    .await?
                                    .iter()
                                    .map(|corporation| corporation.ceo)
                                    .collect();
                            }

                            ceo_ids.clone()
                        }
                        "Executor" => {
                            if executor_ids.is_empty() {
                                executor_ids = bulk_get_alliances(db, corporation_ids.clone())
                                    .await?
                                    .iter()
                                    .filter_map(|alliance: &entity::eve_alliance::Model| {
                                        alliance.executor
                                    })
                                    .collect::<Vec<i32>>();
                            }

                            executor_ids.clone()
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

    let new_members = match filters {
        Some(filters) => match filters.filter_type {
            GroupFilterType::All => result.iter().skip(1).fold(result[0].clone(), |acc, set| {
                acc.intersection(set).cloned().collect::<HashSet<i32>>()
            }),
            GroupFilterType::Any => result.iter().skip(1).fold(result[0].clone(), |acc, set| {
                acc.union(set).cloned().collect::<HashSet<i32>>()
            }),
        }
        .into_iter()
        .collect::<Vec<i32>>(),
        None => result[0].clone().into_iter().collect::<Vec<i32>>(),
    };

    let models: Vec<_> = new_members
        .clone()
        .into_iter()
        .map(|user_id| entity::auth_group_user::ActiveModel {
            group_id: Set(group_id),
            user_id: Set(user_id),
            ..Default::default()
        })
        .collect();

    entity::prelude::AuthGroupUser::insert_many(models)
        .exec(db)
        .await?;

    Ok(new_members)
}
