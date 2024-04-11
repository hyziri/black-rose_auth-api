use axum::{routing::get, Router};

pub fn routes() -> Router {
    use crate::core::controller::login;
    use crate::core::controller::user;

    let auth_routes = Router::new()
        .route("/login", get(login::login))
        .route("/callback", get(login::callback))
        .route("/logout", get(login::logout));
    let user_routes = Router::new()
        .route("/", get(user::get_user))
        .route("/main", get(user::get_user_main_character))
        .route("/characters", get(user::get_user_characters));

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/user", user_routes)
}