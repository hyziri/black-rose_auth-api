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
                    .table(AuthUserPermission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthUserPermission::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AuthUserPermission::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AuthUserPermission::PermissionId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_user_permission-auth_user")
                    .from_tbl(AuthUserPermission::Table)
                    .from_col(AuthUserPermission::UserId)
                    .to_tbl(AuthUser::Table)
                    .to_col(AuthUser::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_user_permission-auth_permission")
                    .from_tbl(AuthUserPermission::Table)
                    .from_col(AuthUserPermission::UserId)
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
                    .name("fk-auth_user_permission-auth_permission")
                    .table(AuthUserPermission::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_user_permission-auth_user")
                    .table(AuthUserPermission::Table)
                    .to_owned(),
            )
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
pub enum AuthUserPermission {
    Table,
    Id,
    UserId,
    PermissionId,
}
