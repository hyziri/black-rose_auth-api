use crate::common::{create_tables, create_user};
use black_rose_auth_api::auth::{
    data::groups::{create_group, members::add_group_members},
    model::groups::{
        GroupFilterCriteria, GroupFilterCriteriaType, GroupFilterType, GroupOwnerType, GroupType,
        NewGroupDto, NewGroupFilterRuleDto,
    },
};
use sea_orm::{Database, TryInsertResult};

#[tokio::test]
async fn group_type_open() -> Result<(), anyhow::Error> {
    let db = Database::connect("sqlite::memory:").await?;
    create_tables(&db).await?;
    let user_id = create_user(
        &db,
        2118500443,
        Some("Elite Drake Pilot".to_string()),
        "test".to_string(),
    )
    .await?;

    let groups = vec![
        NewGroupDto {
            name: "No Requirements All".to_string(),
            description: Some("No requirements group with all filter".to_string()),
            confidential: false,
            leave_applications: false,
            owner_type: GroupOwnerType::Auth,
            owner_id: None,
            group_type: GroupType::Open,
            filter_type: GroupFilterType::All,
            filter_rules: vec![],
            filter_groups: vec![],
        },
        NewGroupDto {
            name: "No Requirements Any".to_string(),
            description: Some("No requirements group with any filter".to_string()),
            confidential: false,
            leave_applications: false,
            owner_type: GroupOwnerType::Auth,
            owner_id: None,
            group_type: GroupType::Open,
            filter_type: GroupFilterType::Any,
            filter_rules: vec![],
            filter_groups: vec![],
        },
    ];

    let mut group_ids = vec![];

    for group in groups {
        group_ids.push(create_group(&db, group).await?.id);
    }

    for group_id in group_ids {
        let result = add_group_members(&db, group_id, vec![user_id]).await?;

        assert!(
            matches!(result, TryInsertResult::Inserted(_)),
            "User rejected when they should be accepted"
        );
    }

    Ok(())
}

#[tokio::test]
async fn filter_type_any() -> Result<(), anyhow::Error> {
    let db = Database::connect("sqlite::memory:").await?;
    create_tables(&db).await?;
    let eligible_user_id = create_user(
        &db,
        2118500443,
        Some("Elite Drake Pilot".to_string()),
        "test".to_string(),
    )
    .await?;
    let ineligible_user_id = create_user(
        &db,
        2122013871,
        Some("Rytsuki's Proctologist".to_string()),
        "test2".to_string(),
    )
    .await?;

    let group = NewGroupDto {
        name: "Nocturne or Black Rose Group".to_string(),
        description: Some("Must be in either Black Rose. or Nocturne. to join.".to_string()),
        confidential: false,
        leave_applications: false,
        owner_type: GroupOwnerType::Auth,
        owner_id: None,
        group_type: GroupType::Open,
        filter_type: GroupFilterType::Any,
        filter_rules: vec![
            NewGroupFilterRuleDto {
                criteria: GroupFilterCriteria::Alliance,
                criteria_type: GroupFilterCriteriaType::Is,
                criteria_value: "99012770".to_string(),
            },
            NewGroupFilterRuleDto {
                criteria: GroupFilterCriteria::Alliance,
                criteria_type: GroupFilterCriteriaType::Is,
                criteria_value: "99011657".to_string(),
            },
        ],
        filter_groups: vec![],
    };

    let group_id = create_group(&db, group).await?.id;

    let accept_result = add_group_members(&db, group_id, vec![eligible_user_id]).await?;
    let reject_result = add_group_members(&db, group_id, vec![ineligible_user_id]).await?;

    assert!(
        matches!(accept_result, TryInsertResult::Inserted(_)),
        "User rejected when they should be accepted"
    );

    assert!(
        matches!(reject_result, TryInsertResult::Empty),
        "User accepted when they should be rejected"
    );

    Ok(())
}

#[tokio::test]
async fn filter_type_all() -> Result<(), anyhow::Error> {
    let db = Database::connect("sqlite::memory:").await?;
    create_tables(&db).await?;
    let eligible_user_id = create_user(
        &db,
        2118500443,
        Some("Elite Drake Pilot".to_string()),
        "test".to_string(),
    )
    .await?;
    let ineligible_user_id = create_user(
        &db,
        2122013871,
        Some("Rytsuki's Proctologist".to_string()),
        "test2".to_string(),
    )
    .await?;

    let group = NewGroupDto {
        name: "Alliance & Corp Group".to_string(),
        description: Some(
            "Alliance must be Black Rose. & corporation must be Killers of the Flower Moon"
                .to_string(),
        ),
        confidential: false,
        leave_applications: false,
        owner_type: GroupOwnerType::Auth,
        owner_id: None,
        group_type: GroupType::Open,
        filter_type: GroupFilterType::All,
        filter_rules: vec![
            NewGroupFilterRuleDto {
                criteria: GroupFilterCriteria::Alliance,
                criteria_type: GroupFilterCriteriaType::Is,
                criteria_value: "99012770".to_string(),
            },
            NewGroupFilterRuleDto {
                criteria: GroupFilterCriteria::Corporation,
                criteria_type: GroupFilterCriteriaType::Is,
                criteria_value: "98755820".to_string(),
            },
        ],
        filter_groups: vec![],
    };

    let group_id = create_group(&db, group).await?.id;
    let accept_result = add_group_members(&db, group_id, vec![eligible_user_id]).await?;
    let reject_result = add_group_members(&db, group_id, vec![ineligible_user_id]).await?;

    assert!(
        matches!(accept_result, TryInsertResult::Inserted(_)),
        "User rejected when they should be accepted"
    );

    assert!(
        matches!(reject_result, TryInsertResult::Empty),
        "User accepted when they should be rejected"
    );

    Ok(())
}

async fn test_filter(group: NewGroupDto) -> Result<(), anyhow::Error> {
    let db = Database::connect("sqlite::memory:").await?;
    create_tables(&db).await?;
    let eligible_user_id = create_user(
        &db,
        2118500443,
        Some("Elite Drake Pilot".to_string()),
        "test".to_string(),
    )
    .await?;
    let ineligible_user_id = create_user(
        &db,
        2122013871,
        Some("Rytsuki's Proctologist".to_string()),
        "test2".to_string(),
    )
    .await?;

    let group_id = create_group(&db, group).await?.id;
    let accept_result = add_group_members(&db, group_id, vec![eligible_user_id]).await?;
    let reject_result = add_group_members(&db, group_id, vec![ineligible_user_id]).await?;

    assert!(
        matches!(accept_result, TryInsertResult::Inserted(_)),
        "User rejected when they should be accepted"
    );

    assert!(
        matches!(reject_result, TryInsertResult::Empty),
        "User accepted when they should be rejected"
    );

    Ok(())
}

#[tokio::test]
async fn group_filter() -> Result<(), anyhow::Error> {
    let db = Database::connect("sqlite::memory:").await?;
    create_tables(&db).await?;
    let eligible_user_id = create_user(&db, 2118500443, "test".to_string()).await?;
    let ineligible_user_id = create_user(&db, 2122013871, "test2".to_string()).await?;

    let group_1 = NewGroupDto {
        name: "No Requirements".to_string(),
        description: Some("No requirements group".to_string()),
        confidential: false,
        leave_applications: false,
        owner_type: GroupOwnerType::Auth,
        owner_id: None,
        group_type: GroupType::Open,
        filter_type: GroupFilterType::Any,
        filter_rules: vec![],
        filter_groups: vec![],
    };

    let group_1_id = create_group(&db, group_1).await?.id;
    let _ = add_group_members(&db, group_1_id, vec![eligible_user_id]).await?;

    let group_2 = NewGroupDto {
        name: "No Requirements".to_string(),
        description: Some("No requirements group".to_string()),
        confidential: false,
        leave_applications: false,
        owner_type: GroupOwnerType::Auth,
        owner_id: None,
        group_type: GroupType::Open,
        filter_type: GroupFilterType::Any,
        filter_rules: vec![NewGroupFilterRuleDto {
            criteria: GroupFilterCriteria::Group,
            criteria_type: GroupFilterCriteriaType::Is,
            criteria_value: group_1_id.to_string(),
        }],
        filter_groups: vec![],
    };

    let group_2_id = create_group(&db, group_2).await?.id;
    let accept_result = add_group_members(&db, group_2_id, vec![eligible_user_id]).await?;
    let reject_result = add_group_members(&db, group_2_id, vec![ineligible_user_id]).await?;

    assert!(
        matches!(accept_result, TryInsertResult::Inserted(_)),
        "User rejected when they should be accepted"
    );

    assert!(
        matches!(reject_result, TryInsertResult::Empty),
        "User accepted when they should be rejected"
    );

    Ok(())
}

#[tokio::test]
async fn corp_filter() -> Result<(), anyhow::Error> {
    let group = NewGroupDto {
        name: "Corporation Group".to_string(),
        description: Some("Must be in required corporation to join".to_string()),
        confidential: false,
        leave_applications: false,
        owner_type: GroupOwnerType::Auth,
        owner_id: None,
        group_type: GroupType::Open,
        filter_type: GroupFilterType::All,
        filter_rules: vec![NewGroupFilterRuleDto {
            criteria: GroupFilterCriteria::Corporation,
            criteria_type: GroupFilterCriteriaType::Is,
            criteria_value: "98755820".to_string(),
        }],
        filter_groups: vec![],
    };

    test_filter(group).await
}

#[tokio::test]
async fn alliance_filter() -> Result<(), anyhow::Error> {
    let group = NewGroupDto {
        name: "Corporation CEO Group".to_string(),
        description: Some("Must be in required alliance to join".to_string()),
        confidential: false,
        leave_applications: false,
        owner_type: GroupOwnerType::Auth,
        owner_id: None,
        group_type: GroupType::Open,
        filter_type: GroupFilterType::All,
        filter_rules: vec![NewGroupFilterRuleDto {
            criteria: GroupFilterCriteria::Alliance,
            criteria_type: GroupFilterCriteriaType::Is,
            criteria_value: "99012770".to_string(),
        }],
        filter_groups: vec![],
    };

    test_filter(group).await
}

#[tokio::test]
async fn ceo_filter() -> Result<(), anyhow::Error> {
    let group = NewGroupDto {
        name: "Corporation CEO Group".to_string(),
        description: Some("Must be a corporation CEO to join".to_string()),
        confidential: false,
        leave_applications: false,
        owner_type: GroupOwnerType::Auth,
        owner_id: None,
        group_type: GroupType::Open,
        filter_type: GroupFilterType::All,
        filter_rules: vec![NewGroupFilterRuleDto {
            criteria: GroupFilterCriteria::Role,
            criteria_type: GroupFilterCriteriaType::Is,
            criteria_value: "CEO".to_string(),
        }],
        filter_groups: vec![],
    };

    test_filter(group).await
}

#[tokio::test]
async fn executor_filter() -> Result<(), anyhow::Error> {
    let db = Database::connect("sqlite::memory:").await?;
    create_tables(&db).await?;
    let eligible_user_id = create_user(
        &db,
        2118500443,
        Some("Elite Drake Pilot".to_string()),
        "test".to_string(),
    )
    .await?;

    let group = NewGroupDto {
        name: "Alliance Executor Group".to_string(),
        description: Some("Must be an alliance executor to join".to_string()),
        confidential: false,
        leave_applications: false,
        owner_type: GroupOwnerType::Auth,
        owner_id: None,
        group_type: GroupType::Open,
        filter_type: GroupFilterType::Any,
        filter_rules: vec![NewGroupFilterRuleDto {
            criteria: GroupFilterCriteria::Role,
            criteria_type: GroupFilterCriteriaType::Is,
            criteria_value: "Executor".to_string(),
        }],
        filter_groups: vec![],
    };

    let group_id = create_group(&db, group).await?.id;
    let accept_result = add_group_members(&db, group_id, vec![eligible_user_id]).await?;

    assert!(
        matches!(accept_result, TryInsertResult::Inserted(_)),
        "User rejected when they should be accepted"
    );

    Ok(())
}
