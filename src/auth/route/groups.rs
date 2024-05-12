use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum::{
    extract,
    response::Response,
    routing::{delete, get, post, put},
    Extension, Router,
};
use sea_orm::DatabaseConnection;
use tower_sessions::Session;

use crate::auth::data;
use crate::auth::model::groups::{GroupDto, NewGroupDto, UpdateGroupDto};

pub fn group_routes() -> Router {
    Router::new()
        .route("/", post(create_group))
        .route("/", get(get_groups))
        .route("/:id", get(get_group_by_id))
        .route("/:id/filters", get(get_group_filters))
        .route("/:id/members", get(get_group_members))
        .route("/:id/members", post(add_group_members))
        .route("/:id/members", delete(delete_group_members))
        .route("/:id/join", post(join_group))
        .route("/:id", put(update_group))
        .route("/:id", delete(delete_group))
}

async fn require_permissions(db: &DatabaseConnection, session: Session) -> Result<i32, Response> {
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
                    return Ok(user_id);
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
    path = "/groups",
    responses(
        (status = 200, description = "Created group info", body = GroupDto),
        (status = 403, description = "Insufficient permissions", body = String),
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

    match data::groups::create_group(&db, payload).await {
        Ok(group) => {
            let dto: GroupDto = group.into();

            (StatusCode::OK, Json(dto)).into_response()
        }
        Err(err) => {
            if err.is::<sea_orm::error::DbErr>() {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error creating new group",
                )
                    .into_response();
            }

            (StatusCode::BAD_REQUEST, err.to_string()).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/groups/{id}/join",
    responses(
        (status = 200, description = "Joined/applied successfully", body = GroupDto),
        (status = 403, description = "Forbidden", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 409, description = "Application already exists", body = String),
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
            if err.to_string() == "Application already exists"
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
    get,
    path = "/groups",
    responses(
        (status = 200, description = "List of groups", body = Vec<GroupDto>),
        (status = 403, description = "Insufficient permissions", body = String),
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

    match data::groups::get_groups(&db).await {
        Ok(groups) => {
            let dto: Vec<GroupDto> = groups.into_iter().map(GroupDto::from).collect();

            (StatusCode::OK, Json(dto)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error getting groups").into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/groups/{id}",
    responses(
        (status = 200, description = "Group info", body = GroupDto),
        (status = 403, description = "Insufficient permissions", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn get_group_by_id(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(group_id): Path<(i32,)>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match data::groups::get_group_by_id(&db, group_id.0).await {
        Ok(group) => match group {
            Some(group) => {
                let dto: GroupDto = group.into();

                (StatusCode::OK, Json(dto)).into_response()
            }
            None => (StatusCode::NOT_FOUND, "Group not found").into_response(),
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error getting groups").into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/groups/{id}/filters",
    responses(
        (status = 200, description = "Group filters", body = Vec<GroupFiltersDto>),
        (status = 403, description = "Insufficient permissions", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn get_group_filters(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(group_id): Path<(i32,)>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match data::groups::filters::get_group_filters(&db, group_id.0).await {
        Ok(filters) => match filters {
            Some(filters) => (StatusCode::OK, Json(filters)).into_response(),
            None => (StatusCode::NOT_FOUND, "Group filters not found").into_response(),
        },
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error getting group filters",
        )
            .into_response(),
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
    put,
    path = "/groups/{id}",
    responses(
        (status = 200, description = "Updated group info", body = GroupDto),
        (status = 403, description = "Insufficient permissions", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn update_group(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(group_id): Path<(i32,)>,
    extract::Json(payload): extract::Json<UpdateGroupDto>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match data::groups::update_group(&db, group_id.0, payload).await {
        Ok(group) => {
            let dto: GroupDto = group.into();

            (StatusCode::OK, Json(dto)).into_response()
        }
        Err(err) => {
            if err.is::<sea_orm::error::DbErr>() {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error creating new group",
                )
                    .into_response();
            }

            (StatusCode::BAD_REQUEST, err.to_string()).into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/groups/{id}",
    responses(
        (status = 200, description = "Group deleted successfully", body = GroupDto),
        (status = 403, description = "Insufficient permissions", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn delete_group(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(group_id): Path<(i32,)>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match data::groups::delete_group(&db, group_id.0).await {
        Ok(result) => match result {
            Some(id) => (StatusCode::OK, format!("Deleted group with id {}", id)).into_response(),
            None => (StatusCode::NOT_FOUND, "Group not found").into_response(),
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error getting groups").into_response(),
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
    let user_id = match require_permissions(&db, session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let user_ids = user_ids.to_vec();

    if user_ids.len() == 1 && user_ids[0] != user_id {
        // require permissions to remove other users
    }

    match data::groups::delete_group_members(&db, group_id.0, user_ids.to_vec()).await {
        Ok(_) => (StatusCode::OK, "Users removed successfully").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error leaving group").into_response(),
    }
}
