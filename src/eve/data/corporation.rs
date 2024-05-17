use entity::prelude::EveCorporation;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
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

    pub async fn get_one(&self, id: i32) -> Result<Option<Corporation>, sea_orm::DbErr> {
        EveCorporation::find_by_id(id).one(self.db).await
    }

    pub async fn get_many(
        &self,
        ids: &[i32],
        page: u64,
        page_size: u64,
    ) -> Result<Vec<Corporation>, sea_orm::DbErr> {
        let ids: Vec<sea_orm::Value> = ids.iter().map(|&id| id.into()).collect();

        EveCorporation::find()
            .filter(entity::eve_corporation::Column::Id.is_in(ids))
            .paginate(self.db, page_size)
            .fetch_page(page)
            .await
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
    use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};

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
    async fn get_one_corporation() -> Result<(), sea_orm::DbErr> {
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

        let retrieved_corporation = repo.get_one(created_corporation.id).await?;

        assert_eq!(retrieved_corporation.unwrap(), created_corporation);

        Ok(())
    }

    #[tokio::test]
    async fn get_many_corporations() -> Result<(), sea_orm::DbErr> {
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

        let created_corporation_ids = created_corporations
            .iter()
            .map(|c| c.id)
            .collect::<Vec<i32>>();

        let mut retrieved_corporations = repo.get_many(&created_corporation_ids, 0, 5).await?;

        created_corporations.sort_by_key(|c| c.id);
        retrieved_corporations.sort_by_key(|c| c.id);

        assert_eq!(retrieved_corporations, created_corporations);

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
