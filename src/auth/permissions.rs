use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use sea_orm::DatabaseConnection;
use tower_sessions::Session;

use crate::auth::data;

use super::data::user::UserRepository;

pub async fn require_permissions(
    db: &DatabaseConnection,
    session: Session,
) -> Result<i32, Response> {
    let user: Option<String> = session.get("user").await.unwrap_or(None);
    let user_id: Option<i32> = user.map(|user| user.parse::<i32>().unwrap());

    let user_id = match user_id {
        Some(user_id) => user_id,
        None => return Err((StatusCode::NOT_FOUND, "User not found").into_response()),
    };

    let user_repo = UserRepository::new(db);

    match user_repo.get_one(user_id).await {
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
