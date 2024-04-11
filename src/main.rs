mod core;
mod eve;
mod router;

use sea_orm::{Database, DatabaseConnection};

use axum::{
    routing::{get, post},
    Router,
};
use core::seed::{create_admin, seed_auth_permissions};
use eve_esi::initialize_eve_esi;
use std::env;
use time::Duration;
use tower_sessions::{cookie::SameSite, Expiry, MemoryStore, Session, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let database_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");

    let db: DatabaseConnection = Database::connect(database_string).await.unwrap();

    let frontend_domain = env::var("FRONTEND_DOMAIN").expect("FRONTEND_DOMAIN must be set!");
    let application_port = env::var("APPLICATION_PORT").unwrap_or_else(|_| String::from("8080"));
    let application_master_key =
        env::var("APPLICATION_MASTER_KEY").expect("APPLICATION_MASTER_KEY must be set!");
    let valkey_url = env::var("VALKEY_URL").expect("VALKEY_URL must be set!");

    let application_name = env::var("APPLICATION_NAME").expect("APPLICATION_NAME must be set!");
    let application_email = env::var("APPLICATION_EMAIL").expect("APPLICATION_EMAIL must be set!");

    let pool = RedisPool::new(RedisConfig::default(), None, None, None, 6)?;

    let redis_conn = pool.connect();
    pool.wait_for_connect().await?;

    let session_store = RedisStore::new(pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(120)));

    initialize_eve_esi(application_name, application_email);

    let _ = seed_auth_permissions(&db).await;
    let _ = create_admin(&db).await;

    let app = router::routes().layer(session_layer);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    println!("Now listening on {}", "127.0.0.1:8080");
    axum::serve(listener, app).await.unwrap();

    redis_conn.await??;

    Ok(())
}
