use std::env;

use black_rose_auth_api::auth::data;
use eve_esi::initialize_eve_esi;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Schema};

use black_rose_auth_api::{
    auth::data::user::update_ownership,
    eve::data::character::{create_character, update_affiliation},
};

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
        let _ = db.execute(db.get_database_backend().build(&stmt)).await;
    }

    Ok(())
}

pub async fn create_user(
    db: &DatabaseConnection,
    character_id: i32,
    name: Option<String>,
    ownerhash: String,
) -> Result<i32, anyhow::Error> {
    let character = create_character(db, character_id, name).await?;

    update_affiliation(db, vec![character.character_id]).await?;

    let user_id = data::user::create_user(db).await?;

    let ownership = update_ownership(db, user_id, character.character_id, ownerhash).await?;

    Ok(ownership.user_id)
}
