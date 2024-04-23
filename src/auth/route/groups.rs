use anyhow::anyhow;
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
use crate::auth::model::groups::{
    GroupDto, GroupFilterCriteria, NewGroupDto, NewGroupFilterRuleDto, UpdateGroupDto,
};

pub fn group_routes() -> Router {
    Router::new()
        .route("/", post(create_group))
        .route("/", get(get_groups))
        .route("/:id", get(get_group_by_id))
        .route("/:id", put(update_group))
        .route("/:id", delete(delete_group))
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

async fn validate_filter_rules(
    db: &DatabaseConnection,
    rules: &Vec<NewGroupFilterRuleDto>,
) -> Result<(), anyhow::Error> {
    for rule in rules {
        match rule.criteria {
            GroupFilterCriteria::Group => {
                use crate::auth::data::groups::get_group_by_id;

                let group_id: i32 = match rule.criteria_value.parse::<i32>() {
                    Ok(id) => id,
                    Err(_) => return Err(anyhow!("Invalid group id: {}", rule.criteria_value)),
                };

                match get_group_by_id(db, group_id).await? {
                    Some(_) => (),
                    None => return Err(anyhow!("Group not found: {}", group_id)),
                }
            }
            GroupFilterCriteria::Corporation => {
                use crate::eve::data::corporation::create_corporation;

                let corporation_id: i32 = match rule.criteria_value.parse::<i32>() {
                    Ok(id) => id,
                    Err(_) => {
                        return Err(anyhow!("Invalid corporation id: {}", rule.criteria_value))
                    }
                };

                match create_corporation(db, corporation_id).await {
                    Ok(_) => (),
                    Err(err) => {
                        if err.is::<reqwest::Error>() {
                            return Err(anyhow!("Corporation not found: {}", rule.criteria_value));
                        }

                        return Err(err);
                    }
                };
            }
            GroupFilterCriteria::Alliance => {
                use crate::eve::data::alliance::create_alliance;

                let alliance_id: i32 = match rule.criteria_value.parse::<i32>() {
                    Ok(id) => id,
                    Err(_) => return Err(anyhow!("Invalid alliance id: {}", rule.criteria_value)),
                };

                match create_alliance(db, alliance_id).await {
                    Ok(_) => (),
                    Err(err) => {
                        if err.is::<reqwest::Error>() {
                            return Err(anyhow!("Alliance not found: {}", rule.criteria_value));
                        }

                        return Err(err);
                    }
                };
            }
            GroupFilterCriteria::Role => {
                if rule.criteria_value != "CEO" && rule.criteria_value != "Alliance Executor" {
                    return Err(anyhow!(
                        "Role must be set to either CEO or Alliance Executor"
                    ));
                }
            }
        }
    }

    Ok(())
}

async fn validate_group_filters(
    db: &DatabaseConnection,
    group: &NewGroupDto,
) -> Result<(), anyhow::Error> {
    validate_filter_rules(db, &group.filter_rules).await?;

    for filter_group in &group.filter_groups {
        validate_filter_rules(db, &filter_group.rules).await?;
    }

    Ok(())
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

    match validate_group_filters(&db, &payload).await {
        Ok(_) => (),
        Err(err) => {
            if err.is::<sea_orm::DbErr>() {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error creating new group",
                )
                    .into_response();
            }

            return (StatusCode::BAD_REQUEST, err.to_string()).into_response();
        }
    }

    match data::groups::create_group(&db, payload).await {
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
    Path(id): Path<(i32,)>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match data::groups::get_group_by_id(&db, id.0).await {
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
    Path(id): Path<(i32,)>,
    extract::Json(payload): extract::Json<UpdateGroupDto>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match validate_group_filters(&db, &payload.clone().into()).await {
        Ok(_) => (),
        Err(err) => {
            if err.is::<sea_orm::DbErr>() {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error creating new group",
                )
                    .into_response();
            }

            return (StatusCode::BAD_REQUEST, err.to_string()).into_response();
        }
    }

    match data::groups::update_group(&db, id.0, payload).await {
        Ok(group) => {
            let dto: GroupDto = group.into();

            (StatusCode::OK, Json(dto)).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error updating group").into_response(),
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
    Path(id): Path<(i32,)>,
) -> Response {
    match require_permissions(&db, session).await {
        Ok(_) => (),
        Err(response) => return response,
    };

    match data::groups::delete_group(&db, id.0).await {
        Ok(result) => match result {
            Some(id) => (StatusCode::OK, format!("Deleted group with id {}", id)).into_response(),
            None => (StatusCode::NOT_FOUND, "Group not found").into_response(),
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error getting groups").into_response(),
    }
}
