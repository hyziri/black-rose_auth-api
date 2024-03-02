use actix_session::Session;
use actix_web::{get, http::header, web, HttpResponse};
use serde::Deserialize;
use std::env;

use crate::core::data::user::change_main;

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

#[derive(Deserialize)]
pub struct QueryParams {
    set_main: Option<bool>,
}

#[get("/login")]
async fn login(session: Session, params: web::Query<QueryParams>) -> HttpResponse {
    let set_main = params.set_main.unwrap_or(false);
    let auth_data = crate::core::service::login::login();

    session.insert("state", &auth_data.state).unwrap();

    if set_main {
        session.insert("set_main", set_main).unwrap();
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
            Err(error) => {
                println!("{}", error);

                return HttpResponse::InternalServerError()
                    .body("There was an issue logging you in, please try again.");
            }
        };

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
