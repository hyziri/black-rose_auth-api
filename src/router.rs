use axum::Router;

pub fn routes() -> Router {
    use crate::auth::routes::login::login_routes;
    use crate::auth::routes::user::user_routes;

    Router::new()
        .nest("/auth", login_routes())
        .nest("/user", user_routes())
}
