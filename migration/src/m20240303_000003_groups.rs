use sea_orm_migration::prelude::*;

use crate::m20240222_000001_initial::{AuthUser, EveAlliance, EveCorporation};
use crate::m20240302_000002_permissions::AuthPermission;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuthGroupCategory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthGroupCategory::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuthGroupCategory::Name).string().not_null())
                    .col(ColumnDef::new(AuthGroupCategory::AllianceId).integer())
                    .col(ColumnDef::new(AuthGroupCategory::CorporationId).integer())
                    .col(
                        ColumnDef::new(AuthGroupCategory::ExclusiveGroups)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AuthGroupCategory::SharedCategory)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_group_category-eve_alliance")
                    .from_tbl(AuthGroupCategory::Table)
                    .from_col(AuthGroupCategory::AllianceId)
                    .to_tbl(EveAlliance::Table)
                    .to_col(EveAlliance::AllianceId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_group_category-eve_corporation")
                    .from_tbl(AuthGroupCategory::Table)
                    .from_col(AuthGroupCategory::CorporationId)
                    .to_tbl(EveCorporation::Table)
                    .to_col(EveCorporation::CorporationId)
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
                    .col(
                        ColumnDef::new(AuthGroup::CategoryId)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(AuthGroup::AllianceId).integer())
                    .col(ColumnDef::new(AuthGroup::CorporationId).integer())
                    .col(
                        ColumnDef::new(AuthGroup::Confidential)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AuthGroup::GroupType)
                            .string()
                            .not_null()
                            .default("Apply"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_group-auth_group_category")
                    .from_tbl(AuthGroup::Table)
                    .from_col(AuthGroup::CategoryId)
                    .to_tbl(AuthGroupCategory::Table)
                    .to_col(AuthGroupCategory::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_group-eve_alliance")
                    .from_tbl(AuthGroup::Table)
                    .from_col(AuthGroup::AllianceId)
                    .to_tbl(EveAlliance::Table)
                    .to_col(EveAlliance::AllianceId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_group-eve_corporation")
                    .from_tbl(AuthGroup::Table)
                    .from_col(AuthGroup::CorporationId)
                    .to_tbl(EveCorporation::Table)
                    .to_col(EveCorporation::CorporationId)
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
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-auth_group_permission-group_id")
                    .table(AuthGroupPermission::Table)
                    .col(AuthGroupPermission::GroupId)
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

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
            .drop_index(
                sea_query::Index::drop()
                    .name("idx-auth_group_permission-group_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AuthGroupPermission::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_group-eve_corporation")
                    .table(AuthGroup::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_group-eve_alliance")
                    .table(AuthGroup::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_group-auth_group_category")
                    .table(AuthGroup::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AuthGroup::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_group_category-eve_corporation")
                    .table(AuthGroupCategory::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_group_category-eve_alliance")
                    .table(AuthGroupCategory::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AuthGroupCategory::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AuthGroupCategory {
    Table,
    Id,
    Name,
    AllianceId,      // Managing alliance of the category
    CorporationId,   // Managing corporation of the category
    ExclusiveGroups, // You can only be part of 1 group in this category at a time
    SharedCategory,  // Alliances & corps can use this category too
}

#[derive(DeriveIden)]
enum AuthGroup {
    Table,
    Id,
    Name,
    CategoryId,
    AllianceId,    // Managing alliance of the group
    CorporationId, // Managing corporation of the group
    Confidential,  // Whether or not members are hidden
    GroupType,     // Open, Auto, Apply, Hidden
}

#[derive(DeriveIden)]
enum AuthGroupPermission {
    Table,
    Id,
    GroupId,
    PermissionId,
}

#[derive(DeriveIden)]
enum AuthGroupUser {
    Table,
    Id,
    GroupId,
    UserId,
}
