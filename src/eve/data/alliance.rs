use std::collections::HashSet;

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use entity::eve_alliance::Model as Alliance;
use entity::prelude::EveAlliance;

pub struct AllianceRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> AllianceRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        alliance_id: i32,
        alliance_name: String,
        executor: Option<i32>,
    ) -> Result<Alliance, sea_orm::DbErr> {
        let new_alliance = entity::eve_alliance::ActiveModel {
            alliance_id: ActiveValue::Set(alliance_id),
            alliance_name: ActiveValue::Set(alliance_name),
            executor: ActiveValue::Set(executor),
            ..Default::default()
        };

        new_alliance.insert(self.db).await
    }

    pub async fn get_one(&self, alliance_id: i32) -> Result<Option<Alliance>, sea_orm::DbErr> {
        EveAlliance::find()
            .filter(entity::eve_alliance::Column::AllianceId.eq(alliance_id))
            .one(self.db)
            .await
    }

    pub async fn get_many(&self, alliance_ids: &[i32]) -> Result<Vec<Alliance>, sea_orm::DbErr> {
        let alliance_ids: Vec<sea_orm::Value> = alliance_ids.iter().map(|&id| id.into()).collect();

        EveAlliance::find()
            .filter(entity::eve_alliance::Column::AllianceId.is_in(alliance_ids))
            .all(self.db)
            .await
    }
}

pub async fn get_alliance(
    db: &DatabaseConnection,
    alliance_id: i32,
) -> Result<Option<Alliance>, sea_orm::DbErr> {
    EveAlliance::find()
        .filter(entity::eve_alliance::Column::AllianceId.eq(alliance_id))
        .one(db)
        .await
}

pub async fn create_alliance(
    db: &DatabaseConnection,
    alliance_id: i32,
) -> Result<Alliance, anyhow::Error> {
    match get_alliance(db, alliance_id).await? {
        Some(alliance) => Ok(alliance),
        None => {
            let alliance = eve_esi::alliance::get_alliance(alliance_id).await?;

            let alliance = entity::eve_alliance::ActiveModel {
                alliance_id: ActiveValue::Set(alliance_id),
                alliance_name: ActiveValue::Set(alliance.name),
                executor: ActiveValue::Set(alliance.executor_corporation_id),
                ..Default::default()
            };

            let alliance: Alliance = alliance.insert(db).await?;

            Ok(alliance)
        }
    }
}

pub async fn bulk_get_alliances(
    db: &DatabaseConnection,
    alliance_ids: Vec<i32>,
) -> Result<Vec<Alliance>, sea_orm::DbErr> {
    let unique_alliance_ids: Vec<i32> = alliance_ids
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    entity::prelude::EveAlliance::find()
        .filter(entity::eve_alliance::Column::AllianceId.is_in(unique_alliance_ids))
        .all(db)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, Rng};
    use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};

    #[tokio::test]
    async fn create_alliance() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;

        let schema = Schema::new(DbBackend::Sqlite);
        let stmt = schema.create_table_from_entity(entity::prelude::EveAlliance);

        let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

        let repo = AllianceRepository::new(&db);

        let mut rng = rand::thread_rng();

        let alliance_id = rng.gen::<i32>();
        let executor = Some(rng.gen::<i32>());
        let alliance_name: String = rng
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        let created_alliance = repo
            .create(alliance_id, alliance_name.clone(), executor)
            .await?;

        assert_eq!(created_alliance.alliance_id, alliance_id);
        assert_eq!(created_alliance.alliance_name, alliance_name);
        assert_eq!(created_alliance.executor, executor);

        Ok(())
    }

    #[tokio::test]
    async fn get_one_alliance() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;

        let schema = Schema::new(DbBackend::Sqlite);
        let stmt = schema.create_table_from_entity(entity::prelude::EveAlliance);

        let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

        let repo = AllianceRepository::new(&db);

        let mut rng = rand::thread_rng();
        let alliance_id = rng.gen::<i32>();
        let executor = Some(rng.gen::<i32>());
        let alliance_name: String = rng
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        let created_alliance = repo
            .create(alliance_id, alliance_name.clone(), executor)
            .await?;

        let retrieved_alliance = repo.get_one(alliance_id).await?;

        assert_eq!(retrieved_alliance.unwrap(), created_alliance);

        Ok(())
    }

    #[tokio::test]
    async fn get_many_alliances() -> Result<(), sea_orm::DbErr> {
        let db = Database::connect("sqlite::memory:").await?;

        let schema = Schema::new(DbBackend::Sqlite);
        let stmt = schema.create_table_from_entity(entity::prelude::EveAlliance);

        let _ = db.execute(db.get_database_backend().build(&stmt)).await?;

        let repo = AllianceRepository::new(&db);

        let mut rng = rand::thread_rng();
        let mut created_alliances = Vec::new();

        let mut generated_ids = std::collections::HashSet::new();
        for _ in 0..5 {
            let mut alliance_id = rng.gen::<i32>();
            while generated_ids.contains(&alliance_id) {
                alliance_id = rng.gen::<i32>();
            }
            generated_ids.insert(alliance_id);

            let executor = Some(rng.gen::<i32>());
            let alliance_name: String = (&mut rng)
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();

            let created_alliance = repo
                .create(alliance_id, alliance_name.clone(), executor)
                .await?;

            created_alliances.push(created_alliance);
        }

        let created_alliance_ids = created_alliances
            .iter()
            .map(|a| a.alliance_id)
            .collect::<Vec<i32>>();

        let mut retrieved_alliances = repo.get_many(&created_alliance_ids).await?;

        created_alliances.sort_by_key(|a| a.alliance_id);
        retrieved_alliances.sort_by_key(|a| a.alliance_id);

        assert_eq!(retrieved_alliances, created_alliances);

        Ok(())
    }
}
