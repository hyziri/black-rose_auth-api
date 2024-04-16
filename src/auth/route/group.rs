use axum::{extract, response::Response, routing::post, Extension, Router};
use tower_sessions::Session;

use crate::auth::model::group::NewGroupDto;

pub fn group_routes() -> Router {
    Router::new().route("/create", post(create_group))
}

pub async fn create_group(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    session: Session,
    extract::Json(payload): extract::Json<NewGroupDto>,
) -> Response {
    todo!()
}
