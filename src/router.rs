use axum::Router;

pub fn routes() -> Router {
    use crate::auth::route::group::group_routes;
    use crate::auth::route::login::login_routes;
    use crate::auth::route::user::user_routes;

    Router::new()
        .nest("/auth", login_routes())
        .nest("/user", user_routes())
        .nest("/group", group_routes())
}
