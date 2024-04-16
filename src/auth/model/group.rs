use serde::Deserialize;

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct NewGroupFilterRuleDto {
    pub criteria: FilterCriteria,
    pub criteria_type: FilterCriteriaType,
    pub criteria_value: String,
}

#[derive(Deserialize)]
pub struct NewFilterGroupDto {
    pub filter_type: FilterType,
    pub rules: Vec<NewGroupFilterRuleDto>,
}

#[derive(Deserialize)]
pub struct NewGroupDto {
    pub name: String,
    pub confidential: bool,
    pub group_type: GroupType,
    pub filter_type: FilterType,
    pub filter_rules: Vec<NewGroupFilterRuleDto>,
    pub filter_groups: Vec<NewFilterGroupDto>,
}
