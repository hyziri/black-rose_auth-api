use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::auth::model::{
    group::{GroupType, NewGroupDto},
    user::UserDto,
};
use crate::auth::route::{auth, group, user};
use crate::eve::model::character::CharacterAffiliationDto;

pub fn routes() -> Router {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            auth::login,
            auth::logout,
            user::get_user,
            user::get_user_main_character,
            user::get_user_characters,
            group::create_group
        ),
        components(schemas(UserDto, NewGroupDto, GroupType, CharacterAffiliationDto)),
        tags(
            (name = "Black Rose Auth API", description = "Black Rose Auth API endpoints")
        )
    )]
    struct ApiDoc;

    use crate::auth::route::auth::auth_routes;
    use crate::auth::route::group::group_routes;
    use crate::auth::route::user::user_routes;

    let routes = Router::new()
        .nest("/auth", auth_routes())
        .nest("/user", user_routes())
        .nest("/group", group_routes());

    if cfg!(debug_assertions) {
        routes.merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
    } else {
        routes
    }
}
