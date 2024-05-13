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
        .route("/:group_id/join", post(join_group))
        .route("/:group_id/leave", delete(leave_group))
        .route("/:group_id/members", get(get_group_members))
        .route("/:group_id/members", post(add_group_members))
        .route("/:group_id/members", delete(delete_group_members))
}

#[utoipa::path(
    post,
    path = "/groups/{group_id}/join",
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

    match data::groups::members::join_group(&db, group_id.0, user_id, application_text.0).await {
        Ok(application) => match application {
            Some(application) => (StatusCode::OK, Json(application)).into_response(),
            None => (StatusCode::OK, "Joined group successfully").into_response(),
        },
        Err(err) => {
            if err.to_string() == "Application to join already exists"
                || err.to_string() == "Already a member"
            {
                return (StatusCode::CONFLICT, err.to_string()).into_response();
            } else if err.to_string() == "User does not meet group requirements"
                || err.to_string() == "Invalid application"
            {
                return (StatusCode::BAD_REQUEST, err.to_string()).into_response();
            } else if err.to_string() == "Group does not exist" {
                return (StatusCode::NOT_FOUND, err.to_string()).into_response();
            } else if err.to_string() == "There was an error returning group application details" {
                return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
            }

            println!("{}", err);

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
    path = "/groups/{group_id}/leave",
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

    match data::groups::members::leave_group(&db, group_id.0, user_id, application_text.0).await {
        Ok(application) => match application {
            Some(application) => (StatusCode::OK, Json(application)).into_response(),
            None => (StatusCode::OK, "Left group successfully").into_response(),
        },
        Err(err) => {
            if err.to_string() == "Application to leave already exists"
                || err.to_string() == "User is not a member of the group"
            {
                return (StatusCode::CONFLICT, err.to_string()).into_response();
            } else if err.to_string() == "Group does not exist" {
                return (StatusCode::NOT_FOUND, err.to_string()).into_response();
            }

            println!("{}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error leaving/requesting to leave group",
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/groups/{group_id}/members",
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

    match data::groups::members::get_group_members(&db, group_id.0).await {
        Ok(members) => (StatusCode::OK, Json(members)).into_response(),
        Err(err) => {
            println!("{}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting group members",
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/groups/{group_id}/members",
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

    match data::groups::members::add_group_members(&db, group_id.0, user_ids.to_vec()).await {
        Ok(_) => (StatusCode::OK, "Users added successfully").into_response(),
        Err(err) => {
            if err.to_string() == "Group does not exist" {
                return (StatusCode::NOT_FOUND, "Group does not exist").into_response();
            }

            println!("{}", err);

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
    path = "/groups/{group_id}/members",
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

    match data::groups::members::delete_group_members(&db, group_id.0, user_ids.to_vec()).await {
        Ok(_) => (StatusCode::OK, "Users removed successfully").into_response(),
        Err(err) => {
            println!("{}", err);

            (StatusCode::INTERNAL_SERVER_ERROR, "Error leaving group").into_response()
        }
    }
}
