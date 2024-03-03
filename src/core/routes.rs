use actix_web::{web, Scope};

use super::controller::login::{callback, login, logout};
use super::controller::user::{
    get_user, get_user_characters, get_user_main_character, get_user_permissions,
};

pub fn auth_service() -> Scope {
    web::scope("/auth")
        .service(login)
        .service(callback)
        .service(logout)
}

pub fn user_service() -> Scope {
    web::scope("/user")
        .service(get_user)
        .service(get_user_main_character)
        .service(get_user_characters)
        .service(get_user_permissions)
}
