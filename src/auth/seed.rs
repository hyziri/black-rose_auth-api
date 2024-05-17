use rand::{distributions::Alphanumeric, Rng};
use redis::Commands;
use sea_orm::{ColumnTrait, DatabaseConnection};
use std::env;

use crate::auth::data::user::UserRepository;

pub async fn create_admin(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    fn generate_random_string() -> String {
        let length = rand::thread_rng().gen_range(20..=64);

        let random_string: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();

        random_string
    }

    let backend_domain = env::var("BACKEND_DOMAIN").expect("BACKEND_DOMAIN must be set!");

    let user_repo = UserRepository::new(db);
    let filters = vec![entity::auth_user::Column::Admin.eq(true)];

    let existing_admin = user_repo.get_by_filtered(filters, 0, 100).await?;

    if existing_admin.is_empty() {
        let valkey_url = env::var("VALKEY_URL").expect("VALKEY_URL must be set!");

        let client = redis::Client::open(format!("redis://{}", valkey_url)).unwrap();
        let mut con = client.get_connection().unwrap();

        let random_string = generate_random_string();

        let _: () = con.set_ex("admin_setup_code", &random_string, 300).unwrap();

        let login_link = format!(
            "http://{}/auth/login?admin_setup={}",
            backend_domain, random_string
        );

        println!(
            "\nCreate an admin account by logging in with EVE Online via: {}\nThe admin login link will expire if not used within 5 minutes.",
            login_link
        );
    } else if cfg!(debug_assertions) {
        println!("\nLogin at http://{}/auth/login", backend_domain)
    };

    Ok(())
}
