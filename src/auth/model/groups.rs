use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub enum GroupType {
    Open,
    Auto,
    Apply,
    Hidden,
}

impl From<GroupType> for entity::sea_orm_active_enums::GroupType {
    fn from(item: GroupType) -> Self {
        match item {
            GroupType::Open => entity::sea_orm_active_enums::GroupType::Open,
            GroupType::Auto => entity::sea_orm_active_enums::GroupType::Auto,
            GroupType::Apply => entity::sea_orm_active_enums::GroupType::Apply,
            GroupType::Hidden => entity::sea_orm_active_enums::GroupType::Hidden,
        }
    }
}

impl From<entity::sea_orm_active_enums::GroupType> for GroupType {
    fn from(item: entity::sea_orm_active_enums::GroupType) -> Self {
        match item {
            entity::sea_orm_active_enums::GroupType::Open => GroupType::Open,
            entity::sea_orm_active_enums::GroupType::Auto => GroupType::Auto,
            entity::sea_orm_active_enums::GroupType::Apply => GroupType::Apply,
            entity::sea_orm_active_enums::GroupType::Hidden => GroupType::Hidden,
        }
    }
}

#[derive(Deserialize)]
pub enum FilterType {
    All,
    Any,
}

#[derive(Deserialize)]
pub enum FilterCriteria {
    Group,
    Corporation,
    Alliance,
    Role,
}

#[derive(Deserialize)]
pub enum FilterCriteriaType {
    Is,
    IsNot,
    GreaterThan,
    LessThan,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GroupDto {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub group_type: GroupType,
}

impl From<entity::auth_group::Model> for GroupDto {
    fn from(model: entity::auth_group::Model) -> Self {
        GroupDto {
            id: model.id,
            name: model.name,
            description: model.description,
            group_type: model.group_type.into(),
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct NewGroupDto {
    pub name: String,
    pub confidential: bool,
    pub description: Option<String>,
    pub group_type: GroupType,
}

#[derive(Deserialize)]
pub struct GroupFilterRuleDto {
    pub criteria: FilterCriteria,
    pub criteria_type: FilterCriteriaType,
    pub criteria_value: String,
}

#[derive(Deserialize)]
pub struct FilterGroupDto {
    pub filter_type: FilterType,
    pub rules: Vec<GroupFilterRuleDto>,
}

#[derive(Deserialize)]
pub struct UpdateGroupFilterDto {
    pub filter_type: FilterType,
    pub filter_rules: Vec<GroupFilterRuleDto>,
    pub filter_groups: Vec<FilterGroupDto>,
}
