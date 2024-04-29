use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: i32,
    pub character_id: i32,
    pub character_name: String,
}

pub struct UserAffiliation {
    pub user_id: i32,
    pub characters: Vec<i32>,
    pub corporations: Vec<i32>,
    pub alliances: Vec<i32>,
}

pub struct UserGroups {
    pub user_id: i32,
    pub groups: Vec<i32>,
}
