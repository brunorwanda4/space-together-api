use crate::config::mongo_manager::MongoManager;
use crate::config::postgres_manager::PgManager;
use anyhow::{anyhow, Context};
use std::env;

pub async fn init_mongo_manager() -> MongoManager {
    let uri = env::var("MONGO_URI").expect("❌ MONGO_URI not set in .env");
    let main_db_name = env::var("MAIN_DB_NAME").unwrap_or_else(|_| "space_together".to_string());
    MongoManager::new(&uri, &main_db_name)
        .await
        .expect("Failed to init MongoManager")
}

pub async fn init_postgres_manager() -> anyhow::Result<PgManager> {
    let database_url = postgres_url_from_env()?;
    PgManager::new(&database_url).await
}

fn postgres_url_from_env() -> anyhow::Result<String> {
    if let Ok(database_url) = env::var("POSTGRES_URL") {
        if !database_url.trim().is_empty() {
            return Ok(database_url);
        }
    }

    let driver = env::var("DB_DRIVER").unwrap_or_default();
    if driver != "postgres" {
        return Err(anyhow!(
            "POSTGRES_URL not set and DB_DRIVER is not configured as postgres"
        ));
    }

    let host = env::var("DB_HOST").context("DB_HOST not set")?;
    let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
    let username = env::var("DB_USERNAME").context("DB_USERNAME not set")?;
    let password = env::var("DB_PASSWORD").context("DB_PASSWORD not set")?;
    let database = env::var("DB_NAME")
        .or_else(|_| env::var("DB_DATABASE"))
        .or_else(|_| env::var("POSTGRES_DB"))
        .unwrap_or_else(|_| username.clone());

    Ok(format!(
        "postgres://{username}:{password}@{host}:{port}/{database}"
    ))
}
