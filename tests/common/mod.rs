use std::env;

use black_rose_auth_api::{
    auth::data::{self, user::UserRepository},
    eve::service::{affiliation::update_affiliation, character::get_or_create_character},
};
use eve_esi::initialize_eve_esi;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Schema, Statement};

use black_rose_auth_api::auth::data::user::update_ownership;

pub async fn create_tables(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    dotenv::dotenv().ok();

    let application_name = env::var("APPLICATION_NAME").expect("APPLICATION_NAME must be set!");
    let application_email = env::var("APPLICATION_EMAIL").expect("APPLICATION_EMAIL must be set!");

    initialize_eve_esi(application_name, application_email);

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
        let _ = db.execute(db.get_database_backend().build(&stmt)).await?;
    }

    // Create index to prevent duplicate membership entries
    // Add group member filter tests will fail without this index due to the constraint not being found
    // If this can be created from entities instead switch to that method
    db.execute(Statement::from_string(DbBackend::Sqlite, "CREATE UNIQUE INDEX IF NOT EXISTS \"idx-auth_group_user-group_id-user_id\" ON \"auth_group_user\" (\"group_id\", \"user_id\");"))
        .await?;

    Ok(())
}

pub async fn create_user(
    db: &DatabaseConnection,
    character_id: i32,
    ownerhash: String,
) -> Result<i32, anyhow::Error> {
    let character = get_or_create_character(db, character_id).await?;

    update_affiliation(db, vec![character.character_id]).await?;

    let user_repo = UserRepository::new(db);

    let user = user_repo.create(false).await?;

    let ownership = update_ownership(db, user.id, character.character_id, ownerhash).await?;

    Ok(ownership.user_id)
}
