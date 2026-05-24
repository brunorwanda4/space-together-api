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
    let database_url = env::var("DB_URL").context("DB_URL not set")?;
    if database_url.trim().is_empty() {
        return Err(anyhow!("DB_URL is empty"));
    }

    Ok(database_url)
}
