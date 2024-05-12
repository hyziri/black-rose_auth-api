use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum::{
    response::Response,
    routing::{delete, get, put},
    Extension, Router,
};
use sea_orm::DatabaseConnection;
use tower_sessions::Session;

use crate::auth::data;
use crate::auth::model::groups::GroupApplicationType;
use crate::auth::permissions::require_permissions;

pub fn group_application_routes() -> Router {
    Router::new()
        .route(
            "/:id/applications/:application_type",
            get(get_group_applications),
        )
        .route("/application/:i32", put(update_group_application))
        .route("/application/:i32", delete(delete_group_application))
}

#[utoipa::path(
    get,
    path = "/groups/{id}/applications/{application_type}",
    responses(
        (status = 200, description = "Outstanding join applications", body = GroupDto),
        (status = 403, description = "Insufficient permissions", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn get_group_applications(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(path): Path<(i32, GroupApplicationType)>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match data::groups::get_group_application(&db, Some(path.1.into()), None, Some(path.0), None)
        .await
    {
        Ok(applications) => (StatusCode::OK, Json(applications)).into_response(),
        Err(err) => {
            if err.to_string() == "Group does not exist" || err.to_string() == "User does not exist"
            {
                return (StatusCode::NOT_FOUND, err.to_string()).into_response();
            }
            if err.to_string() == "Group does not require applications" {
                return (StatusCode::FORBIDDEN, err.to_string()).into_response();
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting group applications",
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    put,
    path = "/groups/application/{id}",
    responses(
        (status = 200, description = "Successfully updated application", body = GroupDto),
        (status = 403, description = "Insufficient permissions", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn update_group_application(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(path): Path<(i32,)>,
    application_text: Json<Option<String>>,
) -> Response {
    let user_id = match require_permissions(&db, session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    match data::groups::get_group_application(&db, None, Some(path.0), None, None).await {
        Ok(application) => {
            if application.is_empty() {
                return (StatusCode::NOT_FOUND, "Application does not exist").into_response();
            };

            if application[0].user_id != user_id {
                return (
                    StatusCode::FORBIDDEN,
                    "Not allowed to edit other user's application",
                )
                    .into_response();
            };
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting group applications",
            )
                .into_response();
        }
    };

    match data::groups::update_group_application(&db, path.0, application_text.0).await {
        Ok(_) => (StatusCode::OK, "Successfully updated application").into_response(),
        Err(err) => {
            if err.to_string() == "Application not found" {
                return (StatusCode::NOT_FOUND, err.to_string()).into_response();
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error updating application",
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/groups/application/{id}",
    responses(
        (status = 200, description = "Successfully deleted application", body = GroupDto),
        (status = 403, description = "Insufficient permissions", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn delete_group_application(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(path): Path<(i32,)>,
) -> Response {
    let user_id = match require_permissions(&db, session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    match data::groups::get_group_application(&db, None, Some(path.0), None, None).await {
        Ok(application) => {
            if application[0].user_id != user_id {
                return (
                    StatusCode::FORBIDDEN,
                    "Not allowed to delete other user's application",
                )
                    .into_response();
            };
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting group applications",
            )
                .into_response();
        }
    };

    match data::groups::delete_group_application(&db, path.0).await {
        Ok(result) => {
            if result.rows_affected == 0 {
                return (StatusCode::NOT_FOUND, "Application does not exist").into_response();
            }

            (StatusCode::OK, "Successfully deleted application").into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error deleting application",
        )
            .into_response(),
    }
}
