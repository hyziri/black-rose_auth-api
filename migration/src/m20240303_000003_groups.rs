use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_query::extension::postgres::Type;

use crate::m20240222_000001_initial::AuthUser;
use crate::m20240302_000002_permissions::AuthPermission;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("group_type"))
                    .values([
                        Alias::new("Open"),
                        Alias::new("Apply"),
                        Alias::new("Auto"),
                        Alias::new("Hidden"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("group_filter_type"))
                    .values([Alias::new("All"), Alias::new("Any")])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthGroup::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthGroup::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuthGroup::Name).string().not_null())
                    .col(ColumnDef::new(AuthGroup::Description).text())
                    .col(
                        ColumnDef::new(AuthGroup::Confidential)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AuthGroup::GroupType)
                            .enumeration(
                                Alias::new("group_type"),
                                [
                                    Alias::new("Open"),
                                    Alias::new("Apply"),
                                    Alias::new("Auto"),
                                    Alias::new("Hidden"),
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AuthGroup::FilterType)
                            .enumeration(
                                Alias::new("group_filter_type"),
                                [Alias::new("All"), Alias::new("Any")],
                            )
                            .not_null()
                            .default("All"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthGroupUser::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthGroupUser::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuthGroupUser::GroupId).integer().not_null())
                    .col(ColumnDef::new(AuthGroupUser::UserId).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-auth_group_user-user_id")
                    .table(AuthGroupUser::Table)
                    .col(AuthGroupUser::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_group_user-auth_group")
                    .from_tbl(AuthGroupUser::Table)
                    .from_col(AuthGroupUser::GroupId)
                    .to_tbl(AuthGroup::Table)
                    .to_col(AuthGroup::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_group_user-auth_permission")
                    .from_tbl(AuthGroupUser::Table)
                    .from_col(AuthGroupUser::UserId)
                    .to_tbl(AuthUser::Table)
                    .to_col(AuthUser::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthGroupFilterGroup::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthGroupFilterGroup::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AuthGroupFilterGroup::GroupId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AuthGroupFilterGroup::FilterType)
                            .enumeration(
                                Alias::new("group_filter_type"),
                                [Alias::new("All"), Alias::new("Any")],
                            )
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_group_filter_group-auth_group")
                    .from_tbl(AuthGroupFilterGroup::Table)
                    .from_col(AuthGroupFilterGroup::GroupId)
                    .to_tbl(AuthGroup::Table)
                    .to_col(AuthGroup::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("group_filter_criteria"))
                    .values([
                        Alias::new("Group"),
                        Alias::new("Corporation"),
                        Alias::new("Alliance"),
                        Alias::new("Role"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("group_filter_criteria_type"))
                    .values([
                        Alias::new("Is"),
                        Alias::new("IsNot"),
                        Alias::new("GreaterThan"),
                        Alias::new("LessThan"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthGroupFilterRule::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthGroupFilterRule::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AuthGroupFilterRule::GroupId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AuthGroupFilterRule::FilterGroupId).integer())
                    .col(
                        ColumnDef::new(AuthGroupFilterRule::Criteria)
                            .enumeration(
                                Alias::new("group_filter_criteria"),
                                [
                                    Alias::new("Group"),
                                    Alias::new("Corporation"),
                                    Alias::new("Alliance"),
                                    Alias::new("Role"),
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AuthGroupFilterRule::CriteriaType)
                            .enumeration(
                                Alias::new("group_filter_criteria_type"),
                                [
                                    Alias::new("Is"),
                                    Alias::new("IsNot"),
                                    Alias::new("GreaterThan"),
                                    Alias::new("LessThan"),
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AuthGroupFilterRule::CriteriaValue)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-auth_group_filter_group-group_id")
                    .table(AuthGroupFilterRule::Table)
                    .col(AuthGroupFilterRule::GroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-auth_group_filter_group-filter_group_id")
                    .table(AuthGroupFilterRule::Table)
                    .col(AuthGroupFilterRule::FilterGroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_group_filter_rule-auth_group")
                    .from_tbl(AuthGroupFilterRule::Table)
                    .from_col(AuthGroupFilterRule::GroupId)
                    .to_tbl(AuthGroup::Table)
                    .to_col(AuthGroup::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_group_filter_rule-auth_group_filter_group")
                    .from_tbl(AuthGroupFilterRule::Table)
                    .from_col(AuthGroupFilterRule::FilterGroupId)
                    .to_tbl(AuthGroupFilterGroup::Table)
                    .to_col(AuthGroupFilterGroup::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthGroupPermission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthGroupPermission::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AuthGroupPermission::GroupId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AuthGroupPermission::PermissionId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_group_permission-auth_group")
                    .from_tbl(AuthGroupPermission::Table)
                    .from_col(AuthGroupPermission::GroupId)
                    .to_tbl(AuthGroup::Table)
                    .to_col(AuthGroup::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_group_permission-auth_permission")
                    .from_tbl(AuthGroupPermission::Table)
                    .from_col(AuthGroupPermission::PermissionId)
                    .to_tbl(AuthPermission::Table)
                    .to_col(AuthPermission::Id)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_group_permission-auth_permission")
                    .table(AuthGroupPermission::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_group_permission-auth_group")
                    .table(AuthGroupPermission::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AuthGroupPermission::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_group_filter_rule-auth_group_filter_group")
                    .table(AuthGroupFilterRule::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_group_filter_rule-auth_group")
                    .table(AuthGroupFilterRule::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                sea_query::Index::drop()
                    .name("idx-auth_group_filter_group-filter_group_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                sea_query::Index::drop()
                    .name("idx-auth_group_filter_group-group_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AuthGroupFilterRule::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("group_filter_criteria_type"))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("group_filter_criteria"))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_group_filter_group-auth_group")
                    .table(AuthGroupFilterGroup::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AuthGroupFilterGroup::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_group_user-auth_permission")
                    .table(AuthGroupUser::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_group_user-auth_group")
                    .table(AuthGroupUser::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                sea_query::Index::drop()
                    .name("idx-auth_group_user-user_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AuthGroupUser::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AuthGroup::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("group_filter_type"))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(Type::drop().name(Alias::new("group_type")).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AuthGroup {
    Table,
    Id,
    Name,
    Description,
    Confidential, // Whether or not members are hidden
    GroupType,    // Open, Auto, Apply, Hidden
    FilterType,   // All, Any
}

#[derive(DeriveIden)]
enum AuthGroupUser {
    Table,
    Id,
    GroupId,
    UserId,
}

#[derive(DeriveIden)]
enum AuthGroupFilterGroup {
    Table,
    Id,
    GroupId,
    FilterType, // All, Any
}

#[derive(DeriveIden)]
enum AuthGroupFilterRule {
    Table,
    Id,
    FilterGroupId,
    GroupId,       // If null then it is not part of a filter group
    Criteria,      // Group, Corporation, Alliance, Role
    CriteriaType,  // IS, IS NOT, GREATER THAN, LESS THAN
    CriteriaValue, // GroupId, CorporationId, AllianceId, Corp CEO/Executor
}

#[derive(DeriveIden)]
enum AuthGroupPermission {
    Table,
    Id,
    GroupId,
    PermissionId,
}
