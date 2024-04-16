use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CharacterAffiliationDto {
    pub character_id: i32,
    pub character_name: String,
    pub corporation_id: i32,
    pub corporation_name: String,
    pub alliance_id: Option<i32>,
    pub alliance_name: Option<String>,
}
