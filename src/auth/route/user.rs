use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use std::collections::HashSet;
use tower_sessions::Session;

use crate::{
    auth::{data::user::get_user_character_ownerships, model::user::UserDto},
    eve::data::character::{bulk_get_character_affiliations, get_character},
};

pub fn user_routes() -> Router {
    Router::new()
        .route("/", get(get_user))
        .route("/main", get(get_user_main_character))
        .route("/characters", get(get_user_characters))
}

async fn get_user_id_from_session(session: Session) -> Result<i32, Response> {
    let user: Option<String> = session.get("user").await.unwrap_or(None);
    let user_id: Option<i32> = user.map(|user| user.parse::<i32>().unwrap());

    match user_id {
        Some(user_id) => Ok(user_id),
        None => Err((StatusCode::NOT_FOUND, "User not found").into_response()),
    }
}

#[utoipa::path(
    get,
    path = "/user",
    responses(
        (status = 200, description = "Current user info", body = UserDto),
        (status = 404, description = "User not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn get_user(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    session: Session,
) -> Response {
    let user_id = match get_user_id_from_session(session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let main_character = match crate::auth::data::user::get_user_main_character(&db, user_id).await
    {
        Ok(main_character) => match main_character {
            Some(main_character) => main_character,
            None => return (StatusCode::NOT_FOUND, "Main character not found.").into_response(),
        },
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting user info.",
            )
                .into_response()
        }
    };

    match get_character(&db, main_character.character_id).await {
        Ok(character) => match character {
            Some(character) => {
                let user_info = UserDto {
                    id: user_id,
                    character_id: character.character_id,
                    character_name: character.character_name,
                };

                (StatusCode::OK, Json(user_info)).into_response()
            }
            None => (StatusCode::NOT_FOUND, "Character info not found.").into_response(),
        },
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error getting character info.",
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/user/main",
    responses(
        (status = 200, description = "Returns user's main character info", body = CharacterAffiliationDto),
        (status = 404, description = "User not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn get_user_main_character(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    session: Session,
) -> Response {
    let user_id = match get_user_id_from_session(session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let main_character = match crate::auth::data::user::get_user_main_character(&db, user_id).await
    {
        Ok(ownership) => match ownership {
            Some(ownership) => ownership,
            None => return (StatusCode::NOT_FOUND, "Main character not found.").into_response(),
        },
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting user's main character.",
            )
                .into_response()
        }
    };

    match bulk_get_character_affiliations(&db, vec![main_character.character_id]).await {
        Ok(affiliation) => {
            if affiliation.is_empty() {
                (StatusCode::NOT_FOUND, "Character info not found.").into_response()
            } else {
                (StatusCode::OK, Json(&affiliation[0])).into_response()
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error getting user info.",
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/user/characters",
    responses(
        (status = 200, description = "Returns list of all user characters", body = [CharacterAffiliationDto]),
        (status = 404, description = "User not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn get_user_characters(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    session: Session,
) -> Response {
    let user_id = match get_user_id_from_session(session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let characters = match get_user_character_ownerships(&db, user_id).await {
        Ok(characters) => characters,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting user characters.",
            )
                .into_response();
        }
    };

    if characters.is_empty() {
        return (StatusCode::NOT_FOUND, "No characters found for user").into_response();
    }

    let character_ids: HashSet<i32> = characters
        .clone()
        .into_iter()
        .map(|char| char.character_id)
        .collect();
    let unique_character_ids: Vec<i32> = character_ids.into_iter().collect();

    match bulk_get_character_affiliations(&db, unique_character_ids).await {
        Ok(character_affiliations) => {
            if character_affiliations.is_empty() {
                (StatusCode::NOT_FOUND, "No characters found for user").into_response()
            } else {
                (StatusCode::OK, Json(character_affiliations)).into_response()
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error getting user info.",
        )
            .into_response(),
    }
}
