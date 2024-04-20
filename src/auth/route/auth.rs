use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Extension, Router,
};
use chrono::{Duration, Utc};
use eve_oauth2::{create_login_url, get_access_token, validate_token};
use oauth2::TokenResponse;
use redis::Commands;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::env;
use tower_sessions::Session;

use crate::auth::data::user::{
    create_user, get_user_character_ownership_by_ownerhash, update_ownership,
};
use crate::auth::data::user::{update_user_as_admin, update_user_main};
use crate::eve::data::character::create_character;
use crate::eve::data::character::update_affiliation;
use entity::auth_user_character_ownership::Model as CharacterOwnership;

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

#[derive(Deserialize)]
pub struct LoginParams {
    set_main: Option<bool>,
    admin_setup: Option<String>,
}

pub fn auth_routes() -> Router {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/logout", get(logout))
}

#[utoipa::path(
    get,
    path = "/auth/login",
    responses(
        (status = 307, description = "Redirect to EVE Online login page"),
        (status = 403, description = "Forbidden", body = String)
    )
)]
pub async fn login(session: Session, params: Query<LoginParams>) -> Response {
    let set_main = params.0.set_main.unwrap_or(false);
    let admin_code = &params.0.admin_setup;

    let backend_domain = env::var("BACKEND_DOMAIN").expect("BACKEND_DOMAIN must be set");
    let esi_client_id = env::var("ESI_CLIENT_ID").expect("ESI_CLIENT_ID must be set");
    let esi_client_secret = env::var("ESI_CLIENT_SECRET").expect("ESI_CLIENT_SECRET must be set");

    let scopes = vec!["".to_string()];
    let redirect_url = format!("http://{}/auth/callback", backend_domain);

    let auth_data = create_login_url(esi_client_id, esi_client_secret, redirect_url, scopes);

    session.insert("state", &auth_data.state).await.unwrap();

    if set_main {
        session.insert("set_main", set_main).await.unwrap();
    }

    match admin_code {
        Some(admin_code) => {
            let valkey_url = env::var("VALKEY_URL").expect("VALKEY_URL must be set!");

            let client = redis::Client::open(format!("redis://{}", valkey_url)).unwrap();
            let mut con = client.get_connection().unwrap();

            let admin_setup_code: Result<String, _> = con.get("admin_setup_code");

            match admin_setup_code {
                Ok(redis_admin_code) => {
                    if &redis_admin_code != admin_code {
                        return (
                            StatusCode::FORBIDDEN,
                            "Invalid admin authorization code, restart your application to get a new one.",
                        )
                            .into_response();
                    }

                    session.insert("set_as_admin", true).await.unwrap();
                }
                Err(_) => {
                    return (
                        StatusCode::FORBIDDEN,
                        "Invalid admin authorization code, restart your application to get a new one.",
                    )
                        .into_response();
                }
            }
        }
        None => (),
    }

    Redirect::temporary(&auth_data.login_url).into_response()
}

pub async fn callback(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    session: Session,
    params: Query<CallbackParams>,
) -> Response {
    async fn get_or_create_user(
        db: &DatabaseConnection,
        code: String,
        user_id: Option<i32>,
    ) -> Result<CharacterOwnership, anyhow::Error> {
        let esi_client_id = env::var("ESI_CLIENT_ID").expect("ESI_CLIENT_ID must be set");
        let esi_client_secret =
            env::var("ESI_CLIENT_SECRET").expect("ESI_CLIENT_SECRET must be set");

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

    let state: Option<String> = session.get("state").await.unwrap_or(None);
    let set_main: Option<bool> = session.get("set_main").await.unwrap_or(None);
    let set_as_admin: Option<bool> = session.get("set_as_admin").await.unwrap_or(None);

    if state.is_none() || Some(params.state.clone()) != state {
        return (
            StatusCode::BAD_REQUEST,
            "There was an issue logging you in, please try again.",
        )
            .into_response();
    }

    let _ = session.remove::<String>("state").await;
    let _ = session.remove::<bool>("set_main").await;

    let user: Option<String> = session.get("user").await.unwrap_or(None);
    let user: Option<i32> = user.map(|user| user.parse::<i32>().unwrap());

    let ownership_entry = match get_or_create_user(&db, params.0.code.clone(), user).await {
        Ok(entry) => entry,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an issue logging you in, please try again.",
            )
                .into_response();
        }
    };

    if let Some(true) = set_as_admin {
        let valkey_url = env::var("VALKEY_URL").expect("VALKEY_URL must be set!");

        let client = redis::Client::open(format!("redis://{}", valkey_url)).unwrap();
        let mut con = client.get_connection().unwrap();

        match update_user_as_admin(&db, ownership_entry.user_id).await {
            Ok(user) => match user {
                Some(_) => (),
                None => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "There was an issue logging you in, please try again.",
                    )
                        .into_response();
                }
            },
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "There was an issue logging you in, please try again.",
                )
                    .into_response();
            }
        };

        let _: () = redis::cmd("DEL")
            .arg("admin_setup_code")
            .query(&mut con)
            .unwrap();
    }

    let redirect_location = match env::var("FRONTEND_DOMAIN").ok().filter(|s| !s.is_empty()) {
        Some(frontend_domain) => {
            let mut redirect_location = format!("http://{}/", frontend_domain);

            if let Some(true) = set_main {
                if !ownership_entry.main {
                    let _ = update_user_main(&db, ownership_entry.character_id).await;

                    redirect_location = format!("http://{}/settings", frontend_domain)
                };
            };

            redirect_location
        }
        None => {
            if cfg!(debug_assertions) {
                let backend_domain =
                    env::var("BACKEND_DOMAIN").expect("BACKEND_DOMAIN must be set");

                format!("http://{}/docs", backend_domain)
            } else {
                panic!("FRONTEND_DOMAIN must be set")
            }
        }
    };

    session
        .insert("user", format!("{}", ownership_entry.user_id))
        .await
        .unwrap();

    Redirect::permanent(&redirect_location).into_response()
}

#[utoipa::path(
    get,
    path = "/auth/logout",
    responses(
        (status = 307, description = "Redirect to front end login page")
    )
)]
pub async fn logout(session: Session) -> Redirect {
    session.clear().await;

    let redirect_location = match env::var("FRONTEND_DOMAIN").ok().filter(|s| !s.is_empty()) {
        Some(frontend_domain) => {
            format!("http://{}/login", frontend_domain)
        }
        None => {
            if cfg!(debug_assertions) {
                let backend_domain =
                    env::var("BACKEND_DOMAIN").expect("BACKEND_DOMAIN must be set");

                format!("http://{}/docs", backend_domain)
            } else {
                panic!("FRONTEND_DOMAIN must be set")
            }
        }
    };

    Redirect::permanent(&redirect_location)
}
