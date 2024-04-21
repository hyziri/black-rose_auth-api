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

#[derive(Serialize, Deserialize, ToSchema)]
pub enum GroupFilterType {
    All,
    Any,
}

impl From<GroupFilterType> for entity::sea_orm_active_enums::GroupFilterType {
    fn from(item: GroupFilterType) -> Self {
        match item {
            GroupFilterType::All => entity::sea_orm_active_enums::GroupFilterType::All,
            GroupFilterType::Any => entity::sea_orm_active_enums::GroupFilterType::Any,
        }
    }
}

impl From<entity::sea_orm_active_enums::GroupFilterType> for GroupFilterType {
    fn from(item: entity::sea_orm_active_enums::GroupFilterType) -> Self {
        match item {
            entity::sea_orm_active_enums::GroupFilterType::All => GroupFilterType::All,
            entity::sea_orm_active_enums::GroupFilterType::Any => GroupFilterType::Any,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum GroupFilterCriteria {
    Group,
    Corporation,
    Alliance,
    Role,
}

impl From<GroupFilterCriteria> for entity::sea_orm_active_enums::GroupFilterCriteria {
    fn from(item: GroupFilterCriteria) -> Self {
        match item {
            GroupFilterCriteria::Group => entity::sea_orm_active_enums::GroupFilterCriteria::Group,
            GroupFilterCriteria::Corporation => {
                entity::sea_orm_active_enums::GroupFilterCriteria::Corporation
            }
            GroupFilterCriteria::Alliance => {
                entity::sea_orm_active_enums::GroupFilterCriteria::Alliance
            }
            GroupFilterCriteria::Role => entity::sea_orm_active_enums::GroupFilterCriteria::Role,
        }
    }
}

impl From<entity::sea_orm_active_enums::GroupFilterCriteria> for GroupFilterCriteria {
    fn from(item: entity::sea_orm_active_enums::GroupFilterCriteria) -> Self {
        match item {
            entity::sea_orm_active_enums::GroupFilterCriteria::Group => GroupFilterCriteria::Group,
            entity::sea_orm_active_enums::GroupFilterCriteria::Corporation => {
                GroupFilterCriteria::Corporation
            }
            entity::sea_orm_active_enums::GroupFilterCriteria::Alliance => {
                GroupFilterCriteria::Alliance
            }
            entity::sea_orm_active_enums::GroupFilterCriteria::Role => GroupFilterCriteria::Role,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum GroupFilterCriteriaType {
    Is,
    IsNot,
    GreaterThan,
    LessThan,
}

impl From<GroupFilterCriteriaType> for entity::sea_orm_active_enums::GroupFilterCriteriaType {
    fn from(item: GroupFilterCriteriaType) -> Self {
        match item {
            GroupFilterCriteriaType::Is => {
                entity::sea_orm_active_enums::GroupFilterCriteriaType::Is
            }
            GroupFilterCriteriaType::IsNot => {
                entity::sea_orm_active_enums::GroupFilterCriteriaType::IsNot
            }
            GroupFilterCriteriaType::GreaterThan => {
                entity::sea_orm_active_enums::GroupFilterCriteriaType::GreaterThan
            }
            GroupFilterCriteriaType::LessThan => {
                entity::sea_orm_active_enums::GroupFilterCriteriaType::LessThan
            }
        }
    }
}

impl From<entity::sea_orm_active_enums::GroupFilterCriteriaType> for GroupFilterCriteriaType {
    fn from(item: entity::sea_orm_active_enums::GroupFilterCriteriaType) -> Self {
        match item {
            entity::sea_orm_active_enums::GroupFilterCriteriaType::Is => {
                GroupFilterCriteriaType::Is
            }
            entity::sea_orm_active_enums::GroupFilterCriteriaType::IsNot => {
                GroupFilterCriteriaType::IsNot
            }
            entity::sea_orm_active_enums::GroupFilterCriteriaType::GreaterThan => {
                GroupFilterCriteriaType::GreaterThan
            }
            entity::sea_orm_active_enums::GroupFilterCriteriaType::LessThan => {
                GroupFilterCriteriaType::LessThan
            }
        }
    }
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
    pub filter_type: GroupFilterType,
    pub filter_rules: Vec<GroupFilterRuleDto>,
    pub filter_groups: Vec<GroupFilterGroupDto>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GroupFilterRuleDto {
    pub criteria: GroupFilterCriteria,
    pub criteria_type: GroupFilterCriteriaType,
    pub criteria_value: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GroupFilterGroupDto {
    pub filter_type: GroupFilterType,
    pub rules: Vec<GroupFilterRuleDto>,
}
