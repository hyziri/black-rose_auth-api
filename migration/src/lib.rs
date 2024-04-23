pub use sea_orm_migration::prelude::*;

mod m20240222_000001_initial;
mod m20240303_000002_groups;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240222_000001_initial::Migration),
            Box::new(m20240303_000002_groups::Migration),
        ]
    }
}
