mod auth;
mod error;
mod eve;
mod router;

#[cfg(test)]
mod mock;

use sea_orm::{Database, DatabaseConnection};

use auth::seed::create_admin;
use axum::Extension;
use eve_esi::initialize_eve_esi;
use std::env;
use time::Duration;
use tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let database_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");

    let db: DatabaseConnection = Database::connect(database_string).await.unwrap();

    let application_port = env::var("APPLICATION_PORT").unwrap_or_else(|_| String::from("8080"));

    let application_name = env::var("APPLICATION_NAME").expect("APPLICATION_NAME must be set!");
    let application_email = env::var("APPLICATION_EMAIL").expect("APPLICATION_EMAIL must be set!");

    let pool = RedisPool::new(RedisConfig::default(), None, None, None, 6)?;

    let redis_conn = pool.connect();
    pool.wait_for_connect().await?;

    let session_store = RedisStore::new(pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    initialize_eve_esi(application_name, application_email);

    let _ = create_admin(&db).await;

    let app = router::routes().layer(Extension(db)).layer(session_layer);

    let binding = format!("0.0.0.0:{}", application_port);

    let listener = tokio::net::TcpListener::bind(&binding).await.unwrap();
    println!("\nNow listening on {}", binding);

    axum::serve(listener, app).await.unwrap();

    redis_conn.await??;

    Ok(())
}
