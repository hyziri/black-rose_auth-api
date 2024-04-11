use std::collections::HashSet;

use crate::{
    core::{data::user::get_user_character_ownerships, model::user::UserDto},
    eve::data::character::{bulk_get_character_affiliations, get_character},
};

async fn get_user_id_from_session(session: Session) -> Result<i32, HttpResponse> {
    let user: Option<String> = session.get("user").unwrap_or(None);
    let user_id: Option<i32> = user.map(|user| user.parse::<i32>().unwrap());

    match user_id {
        Some(user_id) => Ok(user_id),
        None => Err(HttpResponse::NotFound().body("User not found.")),
    }
}

pub async fn get_user(
    db: web::Data<sea_orm::DatabaseConnection>,
    session: Session,
) -> HttpResponse {
    let user_id = match get_user_id_from_session(session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let main_character = match crate::core::data::user::get_user_main_character(&db, user_id).await
    {
        Ok(main_character) => match main_character {
            Some(main_character) => main_character,
            None => return HttpResponse::NotFound().body("Main character not found."),
        },
        Err(_) => return HttpResponse::InternalServerError().body("Error getting user info."),
    };

    match get_character(&db, main_character.character_id).await {
        Ok(character) => match character {
            Some(character) => {
                let user_info = UserDto {
                    id: user_id,
                    character_id: character.character_id,
                    character_name: character.character_name,
                };

                HttpResponse::Found().json(user_info)
            }
            None => HttpResponse::NotFound().body("Character info not found."),
        },
        Err(_) => HttpResponse::InternalServerError().body("Error getting character info."),
    }
}

pub async fn get_user_main_character(
    db: web::Data<sea_orm::DatabaseConnection>,
    session: Session,
) -> HttpResponse {
    let user_id = match get_user_id_from_session(session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let main_character = match crate::core::data::user::get_user_main_character(&db, user_id).await
    {
        Ok(ownership) => match ownership {
            Some(ownership) => ownership,
            None => return HttpResponse::NotFound().body("Main character not found."),
        },
        Err(_) => {
            return HttpResponse::InternalServerError().body("Error getting user's main character.")
        }
    };

    match bulk_get_character_affiliations(&db, vec![main_character.character_id]).await {
        Ok(affiliation) => {
            if affiliation.is_empty() {
                HttpResponse::NotFound().body("Character info not found.")
            } else {
                HttpResponse::Found().json(&affiliation[0])
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Error getting user info."),
    }
}

pub async fn get_user_characters(
    db: web::Data<sea_orm::DatabaseConnection>,
    session: Session,
) -> HttpResponse {
    let user_id = match get_user_id_from_session(session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let characters = match get_user_character_ownerships(&db, user_id).await {
        Ok(characters) => characters,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Error getting user characters.")
        }
    };

    if characters.is_empty() {
        return HttpResponse::NotFound().body("No characters found for user");
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
                HttpResponse::NotFound().body("No characters found for user.")
            } else {
                HttpResponse::Found().json(character_affiliations)
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Error getting user info."),
    }
}
