use chrono::Utc;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(EveAlliance::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EveAlliance::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(EveAlliance::AllianceId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(EveAlliance::AllianceName)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(EveCorporation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EveCorporation::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(EveCorporation::CorporationId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(EveCorporation::CorporationName)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(EveCorporation::AllianceId).integer())
                    .col(
                        ColumnDef::new(EveCorporation::LastUpdated)
                            .timestamp()
                            .not_null()
                            .default(Utc::now().naive_utc()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-eve_corporation-alliance_id")
                    .table(EveCorporation::Table)
                    .col(EveCorporation::AllianceId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-eve_corporation-eve_alliance")
                    .from_tbl(EveCorporation::Table)
                    .from_col(EveCorporation::AllianceId)
                    .to_tbl(EveAlliance::Table)
                    .to_col(EveAlliance::AllianceId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(EveCharacter::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EveCharacter::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(EveCharacter::CharacterId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(EveCharacter::CharacterName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EveCharacter::CorporationId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EveCharacter::LastUpdated)
                            .timestamp()
                            .not_null()
                            .default(Utc::now().naive_utc()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-eve_character-corporation_id")
                    .table(EveCharacter::Table)
                    .col(EveCharacter::CharacterId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-eve_character-eve_corporation")
                    .from_tbl(EveCharacter::Table)
                    .from_col(EveCharacter::CorporationId)
                    .to_tbl(EveCorporation::Table)
                    .to_col(EveCorporation::CorporationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthUser::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthUser::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthUserCharacterOwnership::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthUserCharacterOwnership::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AuthUserCharacterOwnership::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AuthUserCharacterOwnership::CharacterId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(AuthUserCharacterOwnership::Ownerhash)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(AuthUserCharacterOwnership::Main)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-auth_user_character_ownership-user_id")
                    .table(AuthUserCharacterOwnership::Table)
                    .col(AuthUserCharacterOwnership::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_user_character_ownership-auth_user")
                    .from_tbl(AuthUserCharacterOwnership::Table)
                    .from_col(AuthUserCharacterOwnership::UserId)
                    .to_tbl(AuthUser::Table)
                    .to_col(AuthUser::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("fk-auth_user_character_ownership-eve_character")
                    .from_tbl(AuthUserCharacterOwnership::Table)
                    .from_col(AuthUserCharacterOwnership::CharacterId)
                    .to_tbl(EveCharacter::Table)
                    .to_col(EveCharacter::CharacterId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_user_character_ownership-eve_character")
                    .table(AuthUserCharacterOwnership::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-auth_user_character_ownership-auth_user")
                    .table(AuthUserCharacterOwnership::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                sea_query::Index::drop()
                    .name("idx-auth_user_character_ownership-user_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(AuthUserCharacterOwnership::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AuthUser::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-eve_character-eve_corporation")
                    .table(EveCharacter::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                sea_query::Index::drop()
                    .name("idx-eve_character-corporation_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(EveCharacter::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("fk-eve_corporation-eve_alliance")
                    .table(EveCorporation::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                sea_query::Index::drop()
                    .name("idx-eve_corporation-alliance_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(EveCorporation::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(EveAlliance::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum EveAlliance {
    Table,
    Id,
    AllianceId,
    AllianceName,
}

#[derive(DeriveIden)]
pub enum EveCorporation {
    Table,
    Id,
    CorporationId,
    CorporationName,
    AllianceId,
    LastUpdated,
}

#[derive(DeriveIden)]
enum EveCharacter {
    Table,
    Id,
    CharacterId,
    CharacterName,
    CorporationId,
    LastUpdated,
}

#[derive(DeriveIden)]
pub enum AuthUser {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum AuthUserCharacterOwnership {
    Table,
    Id,
    UserId,
    CharacterId,
    Ownerhash,
    Main,
}
