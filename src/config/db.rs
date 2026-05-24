use crate::config::mongo_manager::MongoManager;
use crate::config::postgres_manager::PgManager;
use std::env;

pub async fn init_mongo_manager() -> MongoManager {
    let uri = env::var("MONGO_URI").expect("❌ MONGO_URI not set in .env");
    let main_db_name = env::var("MAIN_DB_NAME").unwrap_or_else(|_| "space_together".to_string());
    MongoManager::new(&uri, &main_db_name)
        .await
        .expect("Failed to init MongoManager")
}

pub async fn init_postgres_manager() -> anyhow::Result<PgManager> {
    let database_url = env::var("POSTGRES_URL").expect("POSTGRES_URL not set in .env");
    PgManager::new(&database_url).await
}
