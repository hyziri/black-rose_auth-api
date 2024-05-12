pub mod members;

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

use self::members::group_member_routes;
use crate::auth::data;
use crate::auth::model::groups::{GroupDto, NewGroupDto, UpdateGroupDto};
use crate::auth::permissions::require_permissions;

pub use self::members::{
    __path_add_group_members, __path_delete_group_members, __path_get_group_join_applications,
    __path_get_group_members, __path_join_group, __path_leave_group,
};

pub fn group_routes() -> Router {
    Router::new()
        .route("/", post(create_group))
        .route("/", get(get_groups))
        .route("/:id", get(get_group_by_id))
        .route("/:id", put(update_group))
        .route("/:id", delete(delete_group))
        .route("/:id/filters", get(get_group_filters))
        .nest("", group_member_routes())
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
