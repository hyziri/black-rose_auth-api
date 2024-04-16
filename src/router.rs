use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::auth;
use crate::auth::route::login;
use crate::auth::route::user;
use crate::eve;

pub fn routes() -> Router {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            login::login,
            login::logout,
            user::get_user,
            user::get_user_main_character,
            user::get_user_characters
        ),
        components(schemas(auth::model::user::UserDto, eve::model::character::CharacterAffiliationDto)),
        tags(
            (name = "Black Rose Auth API", description = "Black Rose Auth API endpoints")
        )
    )]
    struct ApiDoc;

    use crate::auth::route::group::group_routes;
    use crate::auth::route::login::login_routes;
    use crate::auth::route::user::user_routes;

    let routes = Router::new()
        .nest("/auth", login_routes())
        .nest("/user", user_routes())
        .nest("/group", group_routes());

    if cfg!(debug_assertions) {
        routes.merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
    } else {
        routes
    }
}
