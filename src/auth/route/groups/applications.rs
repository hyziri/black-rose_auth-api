use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum::{
    response::Response,
    routing::{delete, get, post, put},
    Extension, Router,
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use utoipa::ToSchema;

use crate::auth::data;
use crate::auth::data::groups::{add_group_members, delete_group_members};
use crate::auth::model::groups::{GroupApplicationStatus, GroupApplicationType};
use crate::auth::permissions::require_permissions;

pub fn group_application_routes() -> Router {
    Router::new()
        .route(
            "/:group_id/applications/:application_status/:application_type",
            get(get_group_applications),
        )
        .route(
            "/application/:application_id",
            put(update_group_application),
        )
        .route(
            "/application/:application_id",
            delete(delete_group_application),
        )
        .route(
            "/application/:application_id/:application_action",
            post(accept_reject_application),
        )
}

#[utoipa::path(
    get,
    path = "/groups/{group_id}/applications/{application_status}/{application_type}",
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
    Path(path): Path<(i32, GroupApplicationStatus, GroupApplicationType)>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match data::groups::get_group_application(
        &db,
        Some(path.1.into()),
        Some(path.2.into()),
        None,
        Some(path.0),
        None,
    )
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

            println!("{}", err);

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
    path = "/groups/application/{application_id}",
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
    application_request_message: Json<Option<String>>,
) -> Response {
    let user_id = match require_permissions(&db, session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    match data::groups::get_group_application(&db, None, None, Some(path.0), None, None).await {
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
        Err(err) => {
            if err.to_string() == "Not allowed to update a completed application" {
                return (StatusCode::FORBIDDEN, err.to_string()).into_response();
            }

            println!("{}", err);

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting group applications",
            )
                .into_response();
        }
    };

    let request_message = application_request_message.0.unwrap_or_default();

    match data::groups::update_group_application(&db, path.0, Some(request_message), None, None)
        .await
    {
        Ok(_) => (StatusCode::OK, "Successfully updated application").into_response(),
        Err(err) => {
            if err.to_string() == "Application not found" {
                return (StatusCode::NOT_FOUND, err.to_string()).into_response();
            }

            println!("{}", err);

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
    path = "/groups/application/{application_id}",
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

    match data::groups::get_group_application(&db, None, None, Some(path.0), None, None).await {
        Ok(application) => {
            if application.is_empty() {
                return (StatusCode::NOT_FOUND, "Application does not exist").into_response();
            };

            if application[0].user_id != user_id {
                return (
                    StatusCode::FORBIDDEN,
                    "Not allowed to delete other user's application",
                )
                    .into_response();
            };
        }
        Err(err) => {
            println!("{}", err);

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
        Err(err) => {
            if err.to_string() == "Not allowed to delete a completed application" {
                return (StatusCode::FORBIDDEN, err.to_string()).into_response();
            } else if err.to_string() == "Application not found" {
                return (StatusCode::NOT_FOUND, err.to_string()).into_response();
            }

            println!("{}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error deleting application",
            )
                .into_response()
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum ApplicationAction {
    Accept,
    Reject,
}

#[utoipa::path(
    post,
    path = "/groups/application/{application_id}/{application_action}",
    responses(
        (status = 200, description = "Successfully approved/rejected application", body = GroupDto),
        (status = 403, description = "Insufficient permissions", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(
        ("login" = [])
    )
)]
pub async fn accept_reject_application(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Path(path): Path<(i32, ApplicationAction)>,
    application_response_message: Json<Option<String>>,
) -> Response {
    let _ = match require_permissions(&db, session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let response_message = application_response_message.0.unwrap_or_default();

    let application_action = match path.1 {
        ApplicationAction::Accept => entity::sea_orm_active_enums::GroupApplicationStatus::Accepted,
        ApplicationAction::Reject => entity::sea_orm_active_enums::GroupApplicationStatus::Rejected,
    };

    let application = match data::groups::update_group_application(
        &db,
        path.0,
        None,
        Some(response_message),
        Some(application_action.clone()),
    )
    .await
    {
        Ok(application) => application,
        Err(err) => {
            if err.to_string() == "Not allowed to update a completed application" {
                return (StatusCode::FORBIDDEN, err.to_string()).into_response();
            } else if err.to_string() == "Application not found" {
                return (StatusCode::NOT_FOUND, err.to_string()).into_response();
            }

            println!("{}", err);

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error updating application",
            )
                .into_response();
        }
    };

    if application_action == entity::sea_orm_active_enums::GroupApplicationStatus::Rejected {
        return (StatusCode::OK, "Successfully rejected application").into_response();
    }

    match application.application_type {
        entity::sea_orm_active_enums::GroupApplicationType::Join => {
            match add_group_members(&db, application.group_id, vec![application.user_id]).await {
                Ok(_) => {
                    (StatusCode::OK, "Successfully approved group join request").into_response()
                }
                Err(err) => {
                    if err.to_string() == "Group does not exist" {
                        return (StatusCode::NOT_FOUND, err.to_string()).into_response();
                    }

                    println!("{}", err);

                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Error approving group join application",
                    )
                        .into_response()
                }
            }
        }
        entity::sea_orm_active_enums::GroupApplicationType::Leave => {
            match delete_group_members(&db, application.group_id, vec![application.user_id]).await {
                Ok(_) => {
                    (StatusCode::OK, "Successfully approved group leave request").into_response()
                }
                Err(err) => {
                    if err.to_string() == "Group does not exist" {
                        return (StatusCode::NOT_FOUND, err.to_string()).into_response();
                    }

                    println!("{}", err);

                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Error approving group leave request",
                    )
                        .into_response()
                }
            }
        }
    }
}
