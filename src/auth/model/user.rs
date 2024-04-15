use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserDto {
    pub id: i32,
    pub character_id: i32,
    pub character_name: String,
}
