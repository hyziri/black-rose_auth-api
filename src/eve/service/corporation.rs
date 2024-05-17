use sea_orm::{ColumnTrait, DatabaseConnection};

use entity::eve_corporation::Model as Corporation;

use crate::error::DbOrReqwestError;
use crate::eve::data::corporation::CorporationRepository;

use super::alliance::get_or_create_alliance;

#[cfg(not(test))]
use eve_esi::corporation::get_corporation;

#[cfg(test)]
use crate::mock::eve_esi_mock::get_corporation;

pub async fn get_or_create_corporation(
    db: &DatabaseConnection,
    corporation_id: i32,
) -> Result<Corporation, DbOrReqwestError> {
    let repo = CorporationRepository::new(db);

    let filters = vec![entity::eve_corporation::Column::CorporationId.eq(corporation_id)];
    let mut corporation = repo.get_by_filtered(filters, 0, 1).await?;

    let corporation = match corporation.pop() {
        Some(corporation) => return Ok(corporation),
        None => {
            let corporation = get_corporation(corporation_id).await?;

            if let Some(alliance_id) = corporation.alliance_id {
                get_or_create_alliance(db, alliance_id).await?;
            }

            repo.create(
                corporation_id,
                corporation.name,
                corporation.alliance_id,
                corporation.ceo_id,
            )
            .await?
        }
    };

    Ok(corporation)
}

#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};

    use crate::error::DbOrReqwestError;

    #[tokio::test]
    async fn get_or_create_corporation() -> Result<(), DbOrReqwestError> {
        use super::get_or_create_corporation;

        let db = Database::connect("sqlite::memory:").await?;
        let schema = Schema::new(DbBackend::Sqlite);

        let stmts = vec![
            schema.create_table_from_entity(entity::prelude::EveCorporation),
            schema.create_table_from_entity(entity::prelude::EveAlliance),
        ];

        for stmt in stmts {
            let _ = db.execute(db.get_database_backend().build(&stmt)).await?;
        }

        let corporation = get_or_create_corporation(&db, 109299958).await?;
        let corporation_2 = get_or_create_corporation(&db, 109299958).await?;

        assert_eq!(corporation, corporation_2);

        Ok(())
    }
}
