use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: i32,
    pub character_id: i32,
    pub character_name: String,
}
