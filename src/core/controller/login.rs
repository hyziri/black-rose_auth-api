use actix_session::Session;
use actix_web::{get, http::header, web, HttpResponse};
use redis::Commands;
use serde::Deserialize;
use std::env;

use crate::core::data::user::{change_main, set_user_as_admin};

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

#[derive(Deserialize)]
pub struct QueryParams {
    set_main: Option<bool>,
    admin_setup: Option<String>,
}

#[get("/login")]
async fn login(session: Session, params: web::Query<QueryParams>) -> HttpResponse {
    let set_main = params.set_main.unwrap_or(false);
    let auth_data = crate::core::service::login::login();
    let admin_code = &params.admin_setup;

    session.insert("state", &auth_data.state).unwrap();

    if set_main {
        session.insert("set_main", set_main).unwrap();
    }

    match admin_code {
        Some(admin_code) => {
            let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set!");

            let client = redis::Client::open(format!("redis://{}", redis_url)).unwrap();
            let mut con = client.get_connection().unwrap();

            let admin_setup_code: Result<String, _> = con.get("admin_setup_code");

            match admin_setup_code {
                Ok(redis_admin_code) => {
                    if &redis_admin_code != admin_code {
                        return HttpResponse::Forbidden().body("Invalid admin authorization code, restart your application to get a new one.");
                    }

                    session.insert("set_as_admin", true).unwrap();
                }
                Err(_) => {
                    return HttpResponse::Forbidden().body("Invalid admin authorization code, restart your application to get a new one.");
                }
            }
        }
        None => (),
    }

    HttpResponse::Found()
        .append_header((header::LOCATION, auth_data.login_url))
        .append_header((header::CACHE_CONTROL, "no-cache"))
        .finish()
}

#[get("/callback")]
async fn callback(
    db: web::Data<sea_orm::DatabaseConnection>,
    session: Session,
    params: web::Query<CallbackParams>,
) -> HttpResponse {
    let state: Option<String> = session.get("state").unwrap_or(None);
    let set_main: Option<bool> = session.get("set_main").unwrap_or(None);
    let set_as_admin: Option<bool> = session.get("set_as_admin").unwrap_or(None);

    let frontend_domain = env::var("FRONTEND_DOMAIN").expect("FRONTEND_DOMAIN must be set!");

    if state.is_none() || Some(params.state.clone()) != state {
        return HttpResponse::BadRequest()
            .body("There was an issue logging you in, please try again.");
    }

    session.remove("state");
    session.remove("set_main");

    let user: Option<String> = session.get("user").unwrap_or(None);
    let user: Option<i32> = user.map(|user| user.parse::<i32>().unwrap());

    let ownership_entry =
        match crate::core::service::login::callback(&db, params.code.clone(), user).await {
            Ok(entry) => entry,
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .body("There was an issue logging you in, please try again.");
            }
        };

    if let Some(true) = set_as_admin {
        let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set!");

        let client = redis::Client::open(format!("redis://{}", redis_url)).unwrap();
        let mut con = client.get_connection().unwrap();

        match set_user_as_admin(&db, ownership_entry.user_id).await {
            Ok(user) => match user {
                Some(_) => (),
                None => {
                    return HttpResponse::InternalServerError()
                        .body("There was an issue logging you in, please try again.")
                }
            },
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .body("There was an issue logging you in, please try again.")
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
        .unwrap();

    HttpResponse::PermanentRedirect()
        .append_header((header::LOCATION, redirect_location))
        .append_header((header::CACHE_CONTROL, "no-cache"))
        .finish()
}

#[get("/logout")]
async fn logout(session: Session) -> HttpResponse {
    session.clear();

    let frontend_domain = env::var("FRONTEND_DOMAIN").expect("FRONTEND_DOMAIN must be set!");

    HttpResponse::PermanentRedirect()
        .append_header((
            header::LOCATION,
            format!("http://{}/login", frontend_domain),
        ))
        .append_header((header::CACHE_CONTROL, "no-cache"))
        .finish()
}
