use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Extension, Router,
};
use redis::Commands;
use serde::Deserialize;
use std::env;
use tower_sessions::Session;

use crate::auth::data::user::{change_main, set_user_as_admin};

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

pub fn login_routes() -> Router {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/logout", get(logout))
}

pub async fn login(session: Session, params: Query<LoginParams>) -> Response {
    let set_main = params.0.set_main.unwrap_or(false);
    let auth_data = crate::auth::service::login::login();
    let admin_code = &params.0.admin_setup;

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
    let state: Option<String> = session.get("state").await.unwrap_or(None);
    let set_main: Option<bool> = session.get("set_main").await.unwrap_or(None);
    let set_as_admin: Option<bool> = session.get("set_as_admin").await.unwrap_or(None);

    let frontend_domain = env::var("FRONTEND_DOMAIN").expect("FRONTEND_DOMAIN must be set!");

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

    let ownership_entry =
        match crate::auth::service::login::callback(&db, params.0.code.clone(), user).await {
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

        match set_user_as_admin(&db, ownership_entry.user_id).await {
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

    let mut redirect_location = format!("http://{}/", frontend_domain);

    if let Some(true) = set_main {
        if !ownership_entry.main {
            let _ = change_main(&db, ownership_entry.character_id).await;

            redirect_location = format!("http://{}/settings", frontend_domain)
        };
    };

    session
        .insert("user", format!("{}", ownership_entry.user_id))
        .await
        .unwrap();

    Redirect::permanent(&redirect_location).into_response()
}

pub async fn logout(session: Session) -> Redirect {
    session.clear().await;

    let frontend_domain = env::var("FRONTEND_DOMAIN").expect("FRONTEND_DOMAIN must be set!");

    let redirect_location = format!("http://{}/login", frontend_domain);

    Redirect::permanent(&redirect_location)
}
