use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Schema};

use black_rose_auth_api::auth::data::groups::{create_group, update_group_members};
use black_rose_auth_api::{
    auth::{
        data::user::{create_user, update_ownership},
        model::groups::{
            GroupFilterCriteria, GroupFilterCriteriaType, GroupFilterType, GroupType, NewGroupDto,
            NewGroupFilterRuleDto,
        },
    },
    eve::data::character::{create_character, update_affiliation},
};

async fn create_tables(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let mut stmts = vec![];

    let schema = Schema::new(DbBackend::Sqlite);

    stmts.push(schema.create_table_from_entity(entity::prelude::EveAlliance));
    stmts.push(schema.create_table_from_entity(entity::prelude::EveCorporation));
    stmts.push(schema.create_table_from_entity(entity::prelude::EveCharacter));
    stmts.push(schema.create_table_from_entity(entity::prelude::AuthUser));
    stmts.push(schema.create_table_from_entity(entity::prelude::AuthUserCharacterOwnership));
    stmts.push(schema.create_table_from_entity(entity::prelude::AuthGroup));
    stmts.push(schema.create_table_from_entity(entity::prelude::AuthGroupFilterGroup));
    stmts.push(schema.create_table_from_entity(entity::prelude::AuthGroupFilterRule));
    stmts.push(schema.create_table_from_entity(entity::prelude::AuthGroupUser));

    for stmt in stmts {
        let _ = db.execute(db.get_database_backend().build(&stmt)).await;
    }

    Ok(())
}

#[tokio::test]
async fn join_group() -> Result<(), anyhow::Error> {
    let db = Database::connect("sqlite::memory:").await?;

    create_tables(&db).await?;

    let user_id = create_user(&db).await?;

    let character =
        create_character(&db, 2118500443, Some("Elite Drake Pilot".to_string())).await?;

    update_affiliation(&db, vec![character.character_id]).await?;
    update_ownership(&db, user_id, character.character_id, "test".to_string()).await?;

    let groups = vec![
        NewGroupDto {
            name: "No requirements group".to_string(),
            description: Some("No requirements to join".to_string()),
            confidential: false,
            group_type: GroupType::Open,
            filter_type: GroupFilterType::Any,
            filter_rules: vec![],
            filter_groups: vec![],
        },
        NewGroupDto {
            name: "Black Rose. group".to_string(),
            description: Some("Open group requires you to be in Black Rose to join".to_string()),
            confidential: false,
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
        },
        NewGroupDto {
            name: "Black Rose. group".to_string(),
            description: Some("Open group requires you to be in Black Rose to join".to_string()),
            confidential: false,
            group_type: GroupType::Open,
            filter_type: GroupFilterType::All,
            filter_rules: vec![NewGroupFilterRuleDto {
                criteria: GroupFilterCriteria::Role,
                criteria_type: GroupFilterCriteriaType::Is,
                criteria_value: "Executor".to_string(),
            }],
            filter_groups: vec![],
        },
    ];

    let mut group_ids = vec![];

    for group in groups {
        group_ids.push(create_group(&db, group).await?.id)
    }

    for group_id in group_ids {
        let result = update_group_members(&db, group_id, vec![user_id]).await?;

        println!("Group_id: {}, success: {:#?}", group_id, !result.is_empty());
    }

    Ok(())
}
