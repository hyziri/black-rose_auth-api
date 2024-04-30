use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// TODO
// The impl for all the enums could be replaced with a macro as all fields are the exact same

#[derive(Serialize, Deserialize, ToSchema, Clone)]
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

#[derive(Serialize, Deserialize, ToSchema, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, ToSchema, Copy, Clone)]
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

#[derive(Serialize, Deserialize, ToSchema, Clone)]
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

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GroupFilters {
    pub id: i32,
    pub filter_type: GroupFilterType,
    pub filter_rules: Vec<GroupFilterRuleDto>,
    pub filter_groups: Vec<GroupFilterGroupDto>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GroupFilterGroupDto {
    pub id: i32,
    pub filter_type: GroupFilterType,
    pub rules: Vec<GroupFilterRuleDto>,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct GroupFilterRuleDto {
    pub id: i32,
    pub criteria: GroupFilterCriteria,
    pub criteria_type: GroupFilterCriteriaType,
    pub criteria_value: String,
}

impl From<entity::auth_group_filter_rule::Model> for GroupFilterRuleDto {
    fn from(model: entity::auth_group_filter_rule::Model) -> Self {
        GroupFilterRuleDto {
            id: model.id,
            criteria: model.criteria.into(),
            criteria_type: model.criteria_type.into(),
            criteria_value: model.criteria_value,
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
    pub filter_rules: Vec<NewGroupFilterRuleDto>,
    pub filter_groups: Vec<NewGroupFilterGroupDto>,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct NewGroupFilterRuleDto {
    pub criteria: GroupFilterCriteria,
    pub criteria_type: GroupFilterCriteriaType,
    pub criteria_value: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct NewGroupFilterGroupDto {
    pub filter_type: GroupFilterType,
    pub rules: Vec<NewGroupFilterRuleDto>,
}

#[derive(Deserialize, ToSchema, Clone)]
pub struct UpdateGroupDto {
    pub name: String,
    pub description: Option<String>,
    pub confidential: bool,
    pub group_type: GroupType,
    pub filter_type: GroupFilterType,
    pub filter_rules: Vec<UpdateGroupFilterRuleDto>,
    pub filter_groups: Vec<UpdateGroupFilterGroupDto>,
}

impl From<UpdateGroupDto> for NewGroupDto {
    fn from(model: UpdateGroupDto) -> Self {
        let new_rules: Vec<NewGroupFilterRuleDto> = model
            .filter_rules
            .into_iter()
            .map(|rule| rule.into())
            .collect();

        let new_groups: Vec<NewGroupFilterGroupDto> = model
            .filter_groups
            .into_iter()
            .map(|rule| rule.into())
            .collect();

        NewGroupDto {
            name: model.name,
            description: model.description,
            confidential: model.confidential,
            group_type: model.group_type,
            filter_type: model.filter_type,
            filter_groups: new_groups,
            filter_rules: new_rules,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct UpdateGroupFilterGroupDto {
    pub id: Option<i32>,
    pub filter_type: GroupFilterType,
    pub rules: Vec<UpdateGroupFilterRuleDto>,
}

impl From<UpdateGroupFilterGroupDto> for NewGroupFilterGroupDto {
    fn from(model: UpdateGroupFilterGroupDto) -> Self {
        let new_rules: Vec<NewGroupFilterRuleDto> =
            model.rules.into_iter().map(|rule| rule.into()).collect();

        NewGroupFilterGroupDto {
            filter_type: model.filter_type,
            rules: new_rules,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct UpdateGroupFilterRuleDto {
    pub id: Option<i32>,
    pub criteria: GroupFilterCriteria,
    pub criteria_type: GroupFilterCriteriaType,
    pub criteria_value: String,
}

impl From<UpdateGroupFilterRuleDto> for NewGroupFilterRuleDto {
    fn from(model: UpdateGroupFilterRuleDto) -> Self {
        NewGroupFilterRuleDto {
            criteria: model.criteria,
            criteria_type: model.criteria_type,
            criteria_value: model.criteria_value,
        }
    }
}
