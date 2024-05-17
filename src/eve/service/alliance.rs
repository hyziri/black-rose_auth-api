use sea_orm::{ColumnTrait, DatabaseConnection};

use entity::eve_alliance::Model as Alliance;

use crate::error::DbOrReqwestError;
use crate::eve::data::alliance::AllianceRepository;

#[cfg(not(test))]
use eve_esi::alliance::get_alliance;

#[cfg(test)]
use crate::mock::eve_esi_mock::get_alliance;

pub async fn get_or_create_alliance(
    db: &DatabaseConnection,
    alliance_id: i32,
) -> Result<Alliance, DbOrReqwestError> {
    let repo = AllianceRepository::new(db);

    let filters = vec![entity::eve_alliance::Column::AllianceId.eq(alliance_id)];
    let mut alliance = repo.get_by_filtered(filters, 0, 1).await?;

    let alliance = match alliance.pop() {
        Some(alliance) => return Ok(alliance),
        None => {
            let alliance = get_alliance(alliance_id).await?;

            repo.create(alliance_id, alliance.name, alliance.executor_corporation_id)
                .await?
        }
    };

    Ok(alliance)
}

#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};

    use crate::error::DbOrReqwestError;

    #[tokio::test]
    async fn get_or_create_alliance() -> Result<(), DbOrReqwestError> {
        use super::get_or_create_alliance;

        let db = Database::connect("sqlite::memory:").await?;
        let schema = Schema::new(DbBackend::Sqlite);

        let stmt = schema.create_table_from_entity(entity::prelude::EveAlliance);

        let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

        let corporation = get_or_create_alliance(&db, 434243723).await?;
        let corporation_2 = get_or_create_alliance(&db, 434243723).await?;

        assert_eq!(corporation, corporation_2);

        Ok(())
    }
}
