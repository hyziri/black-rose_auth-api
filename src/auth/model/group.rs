use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub enum GroupType {
    Open,
    Auto,
    Apply,
    Hidden,
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
