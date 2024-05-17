use entity::prelude::EveCorporation;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Set,
};

use entity::eve_corporation::Model as Corporation;

pub struct CorporationRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> CorporationRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        corporation_id: i32,
        corporation_name: String,
        alliance_id: Option<i32>,
        ceo: i32,
    ) -> Result<Corporation, sea_orm::DbErr> {
        let new_corporation = entity::eve_corporation::ActiveModel {
            corporation_id: Set(corporation_id),
            corporation_name: Set(corporation_name),
            alliance_id: Set(alliance_id),
            ceo: Set(ceo),
            last_updated: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        new_corporation.insert(self.db).await
    }

    pub async fn get_by_filtered(
        &self,
        filters: Vec<migration::SimpleExpr>,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<Corporation>, sea_orm::DbErr> {
        let mut query = EveCorporation::find();

        for filter in filters {
            query = query.filter(filter);
        }

        query.paginate(self.db, page_size).fetch_page(page).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, Rng};
    use sea_orm::{ColumnTrait, ConnectionTrait, Database, DbBackend, Schema};

    async fn initialize_test(
        db: &DatabaseConnection,
    ) -> Result<CorporationRepository, sea_orm::DbErr> {
        let schema = Schema::new(DbBackend::Sqlite);

        let stmts = vec![
            schema.create_table_from_entity(entity::prelude::EveAlliance),
            schema.create_table_from_entity(entity::prelude::EveCorporation),
        ];

        for stmt in stmts {
            let _ = db.execute(db.get_database_backend().build(&stmt)).await?;
        }

        Ok(CorporationRepository::new(db))
    }

    #[tokio::test]
    async fn create_corporation() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        let repo = initialize_test(&db).await?;

        let mut rng = rand::thread_rng();

        let corporation_id = rng.gen::<i32>();
        let alliance_id = None;
        let ceo = rng.gen::<i32>();
        let corporation_name = rng
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect::<String>();

        let created_corporation = repo
            .create(corporation_id, corporation_name.clone(), alliance_id, ceo)
            .await?;

        assert_eq!(created_corporation.corporation_id, corporation_id);
        assert_eq!(created_corporation.corporation_name, corporation_name);
        assert_eq!(created_corporation.alliance_id, alliance_id);
        assert_eq!(created_corporation.ceo, ceo);

        Ok(())
    }

    #[tokio::test]
    async fn get_filtered_corporations() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        let repo = initialize_test(&db).await?;

        let mut rng = rand::thread_rng();
        let mut created_corporations = Vec::new();

        let mut generated_ids = std::collections::HashSet::new();
        for _ in 0..5 {
            let mut corporation_id = rng.gen::<i32>();
            while generated_ids.contains(&corporation_id) {
                corporation_id = rng.gen::<i32>();
            }
            generated_ids.insert(corporation_id);

            let alliance_id = None;
            let ceo = rng.gen::<i32>();
            let corporation_name = (&mut rng)
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect::<String>();

            let created_corporation = repo
                .create(corporation_id, corporation_name.clone(), alliance_id, ceo)
                .await?;

            created_corporations.push(created_corporation);
        }

        let filters = vec![entity::eve_corporation::Column::CorporationId
            .eq(created_corporations[0].corporation_id)];

        let retrieved_corporations = repo.get_by_filtered(filters, 0, 5).await?;

        assert_eq!(retrieved_corporations.len(), 1);

        Ok(())
    }
}
