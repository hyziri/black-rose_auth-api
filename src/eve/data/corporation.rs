use std::collections::HashSet;

use entity::prelude::EveCorporation;
use eve_esi::alliance::get_alliance;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};

use entity::eve_corporation::Model as Corporation;

use super::alliance::AllianceRepository;

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

pub async fn get_corporation(
    db: &DatabaseConnection,
    corporation_id: i32,
) -> Result<Option<Corporation>, sea_orm::DbErr> {
    EveCorporation::find()
        .filter(entity::eve_corporation::Column::CorporationId.eq(corporation_id))
        .one(db)
        .await
}

pub async fn create_corporation(
    db: &DatabaseConnection,
    corporation_id: i32,
) -> Result<Corporation, anyhow::Error> {
    match get_corporation(db, corporation_id).await? {
        Some(corporation) => Ok(corporation),
        None => {
            let corporation = eve_esi::corporation::get_corporation(corporation_id).await?;

            let new_corporation = entity::eve_corporation::ActiveModel {
                corporation_id: Set(corporation_id),
                corporation_name: Set(corporation.name),
                alliance_id: Set(corporation.alliance_id),
                ceo: Set(corporation.ceo_id),
                last_updated: Set(chrono::Utc::now().naive_utc()),
                ..Default::default()
            };

            if let Some(alliance_id) = corporation.alliance_id {
                let alliance = get_alliance(alliance_id).await?;

                let alliance_repo = AllianceRepository::new(db);

                let _ = alliance_repo
                    .create(alliance_id, alliance.name, alliance.executor_corporation_id)
                    .await?;
            }

            let corporation: Corporation = new_corporation.insert(db).await?;

            Ok(corporation)
        }
    }
}

pub async fn bulk_get_corporations(
    db: &DatabaseConnection,
    corporation_ids: Vec<i32>,
) -> Result<Vec<Corporation>, sea_orm::DbErr> {
    let unique_corp_ids: Vec<i32> = corporation_ids
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    entity::prelude::EveCorporation::find()
        .filter(entity::eve_corporation::Column::CorporationId.is_in(unique_corp_ids))
        .all(db)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, Rng};
    use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};

    #[tokio::test]
    async fn create_corporation() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        let schema = Schema::new(DbBackend::Sqlite);

        let stmts = vec![
            schema.create_table_from_entity(entity::prelude::EveCorporation),
            schema.create_table_from_entity(entity::prelude::EveAlliance),
        ];

        for stmt in stmts {
            let _ = db.execute(db.get_database_backend().build(&stmt)).await?;
        }

        let repo = CorporationRepository::new(&db);

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
        let schema = Schema::new(DbBackend::Sqlite);

        let stmts = vec![
            schema.create_table_from_entity(entity::prelude::EveCorporation),
            schema.create_table_from_entity(entity::prelude::EveAlliance),
        ];

        for stmt in stmts {
            let _ = db.execute(db.get_database_backend().build(&stmt)).await?;
        }

        let repo = CorporationRepository::new(&db);

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
        let schema = Schema::new(DbBackend::Sqlite);

        let stmts = vec![
            schema.create_table_from_entity(entity::prelude::EveCorporation),
            schema.create_table_from_entity(entity::prelude::EveAlliance),
        ];

        for stmt in stmts {
            let _ = db.execute(db.get_database_backend().build(&stmt)).await?;
        }

        let repo = CorporationRepository::new(&db);

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
        let schema = Schema::new(DbBackend::Sqlite);

        let stmts = vec![
            schema.create_table_from_entity(entity::prelude::EveCorporation),
            schema.create_table_from_entity(entity::prelude::EveAlliance),
        ];

        for stmt in stmts {
            let _ = db.execute(db.get_database_backend().build(&stmt)).await?;
        }

        let repo = CorporationRepository::new(&db);

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
