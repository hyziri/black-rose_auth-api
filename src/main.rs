mod core;
mod eve;

use sea_orm::{Database, DatabaseConnection};

use actix_cors::Cors;
use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::middleware::DefaultHeaders;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use core::routes::user_service;
use core::seed::{create_admin, seed_auth_permissions};
use eve_esi::initialize_eve_esi;
use std::env;

use crate::core::routes::auth_service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let database_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");

    let db: DatabaseConnection = Database::connect(database_string).await.unwrap();

    let frontend_domain = env::var("FRONTEND_DOMAIN").expect("FRONTEND_DOMAIN must be set!");
    let application_port = env::var("APPLICATION_PORT").unwrap_or_else(|_| String::from("8080"));
    let application_secret_key =
        env::var("APPLICATION_SECRET").expect("APPLICATION_SECRET must be set!");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set!");

    let secret_key = Key::derive_from(application_secret_key.as_bytes());

    let application_name = env::var("APPLICATION_NAME").expect("APPLICATION_NAME must be set!");
    let application_email = env::var("APPLICATION_EMAIL").expect("APPLICATION_EMAIL must be set!");

    initialize_eve_esi(application_name, application_email);

    let _ = seed_auth_permissions(&db).await;
    let _ = create_admin(&db).await;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&format!("http://{}", frontend_domain))
            .allow_any_header()
            .allow_any_method()
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(DefaultHeaders::new().add(("Referrer-Policy", "no-referrer")))
            .wrap(
                SessionMiddleware::builder(
                    RedisActorSessionStore::new(redis_url.clone()),
                    secret_key.clone(),
                )
                .build(),
            )
            .app_data(Data::new(db.clone()))
            .service(auth_service())
            .service(user_service())
    })
    .bind(format!("0.0.0.0:{}", application_port))?
    .run()
    .await
}
