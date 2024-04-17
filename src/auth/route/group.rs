use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum::{
    extract,
    response::Response,
    routing::{get, post},
    Extension, Router,
};
use sea_orm::DatabaseConnection;
use tower_sessions::Session;

use crate::auth::data;
use crate::auth::model::group::{GroupDto, NewGroupDto};

pub fn group_routes() -> Router {
    Router::new()
        .route("/", get(get_groups))
        .route("/create", post(create_group))
}

async fn require_permissions(db: &DatabaseConnection, session: Session) -> Result<(), Response> {
    let user: Option<String> = session.get("user").await.unwrap_or(None);
    let user_id: Option<i32> = user.map(|user| user.parse::<i32>().unwrap());

    let user_id = match user_id {
        Some(user_id) => user_id,
        None => return Err((StatusCode::NOT_FOUND, "User not found").into_response()),
    };

    match data::user::get_user(db, user_id).await {
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
    path = "/groups/create",
    responses(
        (status = 200, description = "Group created successfully"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found", body = String),
        (status = 500, description = "Internal server error", body = String)
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

    match data::group::create_group(&db, payload).await {
        Ok(group) => {
            let dto: GroupDto = group.into();

            (StatusCode::OK, Json(dto)).into_response()
        }
        Err(err) => {
            println!("{}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error creating new group",
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/groups",
    responses(
        (status = 200, description = "Returns a list of groups"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn get_groups(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match data::group::get_groups(&db).await {
        Ok(groups) => {
            let dto: Vec<GroupDto> = groups.into_iter().map(GroupDto::from).collect();

            (StatusCode::OK, Json(dto)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error getting groups").into_response(),
    }
}
