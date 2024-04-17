use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract, response::Response, routing::post, Extension, Router};
use sea_orm::DatabaseConnection;
use tower_sessions::Session;

use crate::auth::data::user::get_user;
use crate::auth::model::group::NewGroupDto;

pub fn group_routes() -> Router {
    Router::new().route("/create", post(create_group))
}

async fn require_permissions(db: &DatabaseConnection, session: Session) -> Result<(), Response> {
    let user: Option<String> = session.get("user").await.unwrap_or(None);
    let user_id: Option<i32> = user.map(|user| user.parse::<i32>().unwrap());

    let user_id = match user_id {
        Some(user_id) => user_id,
        None => return Err((StatusCode::NOT_FOUND, "User not found").into_response()),
    };

    match get_user(db, user_id).await {
        Ok(user) => match user {
            Some(user) => {
                if user.admin {
                    return Ok(());
                }
            }
            None => return Err((StatusCode::NOT_FOUND, "User not found").into_response()),
        },
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an issue getting user info",
            )
                .into_response())
        }
    };

    Err((StatusCode::FORBIDDEN, "Insufficient permissions").into_response())
}

#[utoipa::path(
    post,
    path = "/group/create",
    responses(
        (status = 200, description = "Group created successfully"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn create_group(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    extract::Json(payload): extract::Json<NewGroupDto>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    // Create group

    (StatusCode::OK, "You have correct permissions").into_response()
}
