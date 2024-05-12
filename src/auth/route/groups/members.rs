use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum::{
    response::Response,
    routing::{delete, get, post},
    Extension, Router,
};
use sea_orm::DatabaseConnection;
use tower_sessions::Session;

use crate::auth::data;
use crate::auth::permissions::require_permissions;

pub fn group_member_routes() -> Router {
    Router::new()
        .route("/:id/join", post(join_group))
        .route("/:id/leave", delete(leave_group))
        .route("/:id/members", get(get_group_members))
        .route("/:id/members", post(add_group_members))
        .route("/:id/members", delete(delete_group_members))
}

#[utoipa::path(
    post,
    path = "/groups/{id}/join",
    responses(
        (status = 200, description = "Joined/applied successfully", body = GroupDto),
        (status = 403, description = "Forbidden", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 409, description = "Application to join already exists", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn join_group(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(group_id): Path<(i32,)>,
    application_text: Json<Option<String>>,
) -> Response {
    let user_id = match require_permissions(&db, session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    match data::groups::join_group(&db, group_id.0, user_id, application_text.0).await {
        Ok(message) => (StatusCode::OK, message).into_response(),
        Err(err) => {
            if err.to_string() == "Application to join already exists"
                || err.to_string() == "Already a member"
            {
                return (StatusCode::CONFLICT, err.to_string()).into_response();
            } else if err.to_string() == "User does not meet group requirements" {
                return (StatusCode::BAD_REQUEST, err.to_string()).into_response();
            } else if err.to_string() == "Group does not exist" {
                return (StatusCode::NOT_FOUND, err.to_string()).into_response();
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error adding user to group",
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/groups/{id}/leave",
    responses(
        (status = 200, description = "Left/sent request to leave successfully", body = GroupDto),
        (status = 403, description = "Forbidden", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 409, description = "Application already exists", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn leave_group(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(group_id): Path<(i32,)>,
    application_text: Json<Option<String>>,
) -> Response {
    let user_id = match require_permissions(&db, session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    match data::groups::leave_group(&db, group_id.0, user_id, application_text.0).await {
        Ok(_) => (StatusCode::OK, "Left group successfully").into_response(),
        Err(err) => {
            if err.to_string() == "Application to leave already exists"
                || err.to_string() == "User is not a member of the group"
            {
                return (StatusCode::CONFLICT, err.to_string()).into_response();
            } else if err.to_string() == "Group does not exist" {
                return (StatusCode::NOT_FOUND, err.to_string()).into_response();
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error adding user to group",
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/groups/{id}/members",
    responses(
        (status = 200, description = "Group members", body = Vec<UserDto>),
        (status = 403, description = "Insufficient permissions", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn get_group_members(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(group_id): Path<(i32,)>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match data::groups::get_group_members(&db, group_id.0).await {
        Ok(members) => (StatusCode::OK, Json(members)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error getting group members",
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/groups/{id}/members",
    responses(
        (status = 200, description = "Users added successfully", body = GroupDto),
        (status = 403, description = "Forbidden", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn add_group_members(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(group_id): Path<(i32,)>,
    user_ids: Json<Vec<i32>>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match data::groups::add_group_members(&db, group_id.0, user_ids.to_vec()).await {
        Ok(_) => (StatusCode::OK, "Users added successfully").into_response(),
        Err(err) => {
            if err.to_string() == "Group does not exist" {
                return (StatusCode::NOT_FOUND, "Group does not exist").into_response();
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error adding user to group",
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/groups/{id}/members",
    responses(
        (status = 200, description = "Users removed successfully", body = GroupDto),
        (status = 403, description = "Insufficient permissions", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn delete_group_members(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(group_id): Path<(i32,)>,
    user_ids: Json<Vec<i32>>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    let user_ids = user_ids.to_vec();

    match data::groups::delete_group_members(&db, group_id.0, user_ids.to_vec()).await {
        Ok(_) => (StatusCode::OK, "Users removed successfully").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error leaving group").into_response(),
    }
}
