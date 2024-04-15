use chrono::{Duration, Utc};
use eve_oauth2::{create_login_url, get_access_token, validate_token, AuthenticationData};
use oauth2::TokenResponse;
use sea_orm::DatabaseConnection;
use std::env;

use entity::auth_user_character_ownership::Model as CharacterOwnership;

use crate::auth::data::user::{
    create_user, get_user_character_ownership_by_ownerhash, update_ownership,
};
use crate::eve::data::character::create_character;
use crate::eve::data::character::update_affiliation;

pub fn login() -> AuthenticationData {
    let backend_domain = env::var("BACKEND_DOMAIN").expect("BACKEND_DOMAIN must be set");
    let esi_client_id = env::var("ESI_CLIENT_ID").expect("ESI_CLIENT_ID must be set");
    let esi_client_secret = env::var("ESI_CLIENT_SECRET").expect("ESI_CLIENT_SECRET must be set");

    let scopes = vec!["".to_string()];
    let redirect_url = format!("http://{}/auth/callback", backend_domain);

    create_login_url(esi_client_id, esi_client_secret, redirect_url, scopes)
}

pub async fn callback(
    db: &DatabaseConnection,
    code: String,
    user_id: Option<i32>,
) -> Result<CharacterOwnership, anyhow::Error> {
    let esi_client_id = env::var("ESI_CLIENT_ID").expect("ESI_CLIENT_ID must be set");
    let esi_client_secret = env::var("ESI_CLIENT_SECRET").expect("ESI_CLIENT_SECRET must be set");

    let token = get_access_token(esi_client_id, esi_client_secret, code).await;
    let token_claims = validate_token(token.access_token().secret().to_string()).await;

    let id_str = token_claims.claims.sub.split(':').collect::<Vec<&str>>()[2];
    let character_id: i32 = id_str.parse().expect("Failed to parse id to i32");

    let character = create_character(db, character_id, Some(token_claims.claims.name)).await?;

    if Utc::now().naive_utc() - character.last_updated > Duration::hours(1) {
        update_affiliation(db, vec![character_id]).await?
    }

    let ownerhash = token_claims.claims.owner;

    match user_id {
        Some(user_id) => Ok(update_ownership(db, user_id, character_id, ownerhash).await?),
        None => {
            let ownership =
                get_user_character_ownership_by_ownerhash(db, ownerhash.clone()).await?;

            match ownership {
                Some(ownership) => Ok(ownership),
                None => {
                    let user_id = create_user(db).await?;

                    Ok(update_ownership(db, user_id, character_id, ownerhash).await?)
                }
            }
        }
    }
}
