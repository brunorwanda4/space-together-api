use crate::config::mongo_manager::MongoManager;
use std::env;

pub async fn init_mongo_manager() -> MongoManager {
    let uri = env::var("MONGO_URI").expect("❌ MONGO_URI not set in .env");
    let main_db_name = env::var("MAIN_DB_NAME").unwrap_or_else(|_| "space_together".to_string());
    MongoManager::new(&uri, &main_db_name)
        .await
        .expect("Failed to init MongoManager")
}
