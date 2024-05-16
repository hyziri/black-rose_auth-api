use axum::extract::{Path, Query};
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
use crate::auth::model::groups::{GroupApplicationStatus, GroupApplicationType};
use crate::auth::permissions::require_permissions;

pub fn group_application_routes() -> Router {
    Router::new()
        .route("/", get(get_group_applications))
        .route("/:application_id", put(update_group_application))
        .route("/:application_id", delete(delete_group_application))
        .route(
            "/:application_id/:application_action",
            post(accept_reject_application),
        )
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GetGroupApplicationParams {
    pub group_id: Option<i32>,
    pub user_id: Option<i32>,
    pub application_status: Option<GroupApplicationStatus>,
    pub application_type: Option<GroupApplicationType>,
}

#[utoipa::path(
    get,
    path = "/groups/applications",
    responses(
        (status = 200, description = "Applications", body = GroupDto),
        (status = 403, description = "Insufficient permissions", body = String),
        (status = 404, description = "Not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    params (
        ("group_id" = Option<i32>, Query, description = "Filter by group id"),
        ("user_id" = Option<i32>, Query, description = "Filter user by id"),
        ("application_status" = Option<GroupApplicationStatus>, Query, description = "Filter by application status"),
        ("application_type" = Option<GroupApplicationType>, Query, description = "Filter by application type"),
    ),
    security(
        ("login" = [])
    )
)]
pub async fn get_group_applications(
    Extension(db): Extension<DatabaseConnection>,
    session: Session,
    Query(params): Query<GetGroupApplicationParams>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match data::groups::applications::get_group_application(
        &db,
        params.application_status.map(|status| status.into()),
        params.application_type.map(|type_| type_.into()),
        None,
        params.group_id,
        params.user_id,
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
    path = "/groups/applications/{application_id}",
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

    match data::groups::applications::get_group_application(
        &db,
        None,
        None,
        Some(path.0),
        None,
        None,
    )
    .await
    {
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

    match data::groups::applications::update_group_application(
        &db,
        path.0,
        Some(request_message),
        None,
        None,
        None,
    )
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
    path = "/groups/applications/{application_id}",
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

    match data::groups::applications::get_group_application(
        &db,
        None,
        None,
        Some(path.0),
        None,
        None,
    )
    .await
    {
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

    match data::groups::applications::delete_group_application(&db, path.0).await {
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
    path = "/groups/applications/{application_id}/{application_action}",
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
    let responder_id = match require_permissions(&db, session).await {
        Ok(user_id) => user_id,
        Err(response) => return response,
    };

    let response_message = application_response_message.0.unwrap_or_default();

    let application_action = match path.1 {
        ApplicationAction::Accept => entity::sea_orm_active_enums::GroupApplicationStatus::Accepted,
        ApplicationAction::Reject => entity::sea_orm_active_enums::GroupApplicationStatus::Rejected,
    };

    let application = match data::groups::applications::update_group_application(
        &db,
        path.0,
        None,
        Some(response_message),
        Some(application_action.clone()),
        Some(responder_id),
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

    match application.request_type {
        entity::sea_orm_active_enums::GroupApplicationType::Join => {
            match data::groups::members::add_group_members(
                &db,
                application.group_id,
                vec![application.user_id],
            )
            .await
            {
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
            match data::groups::members::delete_group_members(
                &db,
                application.group_id,
                vec![application.user_id],
            )
            .await
            {
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
