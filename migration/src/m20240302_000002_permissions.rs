use sea_orm_migration::prelude::*;

use crate::m20240222_000001_initial::AuthUser;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuthPermission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthPermission::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuthPermission::Module).string().not_null())
                    .col(ColumnDef::new(AuthPermission::Name).string().not_null())
                    .col(
                        ColumnDef::new(AuthPermission::Hidden)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthRole::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthRole::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuthRole::Name).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthRolePermissions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthRolePermissions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AuthRolePermissions::RoleId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AuthRolePermissions::PermissionId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_role_permissions-auth_role")
                    .from_tbl(AuthRolePermissions::Table)
                    .from_col(AuthRolePermissions::RoleId)
                    .to_tbl(AuthRole::Table)
                    .to_col(AuthRole::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_role_permissions-auth_permission")
                    .from_tbl(AuthRolePermissions::Table)
                    .from_col(AuthRolePermissions::PermissionId)
                    .to_tbl(AuthPermission::Table)
                    .to_col(AuthPermission::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthRoleUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthRoleUsers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuthRoleUsers::RoleId).integer().not_null())
                    .col(ColumnDef::new(AuthRoleUsers::UserId).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_role_users-auth_role")
                    .from_tbl(AuthRoleUsers::Table)
                    .from_col(AuthRoleUsers::RoleId)
                    .to_tbl(AuthRole::Table)
                    .to_col(AuthRole::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_role_users-auth_user")
                    .from_tbl(AuthRoleUsers::Table)
                    .from_col(AuthRoleUsers::UserId)
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
                    .name("fk-auth_role_users-auth_user")
                    .table(AuthRoleUsers::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_role_users-auth_role")
                    .table(AuthRoleUsers::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AuthRoleUsers::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_role_permissions-auth_permission")
                    .table(AuthRolePermissions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_role_permissions-auth_role")
                    .table(AuthRolePermissions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AuthRolePermissions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AuthRole::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AuthPermission::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum AuthPermission {
    Table,
    Id,
    Module,
    Name,
    Hidden,
}

#[derive(DeriveIden)]
pub enum AuthRole {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
enum AuthRolePermissions {
    Table,
    Id,
    RoleId,
    PermissionId,
}

#[derive(DeriveIden)]
enum AuthRoleUsers {
    Table,
    Id,
    UserId,
    RoleId,
}
