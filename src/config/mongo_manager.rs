
use dashmap::DashMap;
use mongodb::{Client, Database};
use std::sync::Arc;

#[derive(Clone)]
pub struct MongoManager {
    pub client: Client,
    // cache of Database handles keyed by db_name
    pub db_cache: Arc<DashMap<String, Database>>,
    pub main_db_name: String,
}

impl MongoManager {
    pub async fn new(uri: &str, main_db_name: &str) -> mongodb::error::Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        Ok(Self {
            client,
            db_cache: Arc::new(DashMap::new()),
            main_db_name: main_db_name.to_string(),
        })
    }

    /// Get handle to the control/main database
    pub fn main_db(&self) -> Database {
        self.client.database(&self.main_db_name)
    }

    /// Get or cache a database handle for the given school_db_name
    pub fn get_db(&self, db_name: &str) -> Database {
        if let Some(v) = self.db_cache.get(db_name) {
            return v.clone();
        }
        let db = self.client.database(db_name);
        self.db_cache.insert(db_name.to_string(), db.clone());
        db
    }

    /// helper to produce default school db name
    pub fn school_db_name_from_id(&self, school_id: &str) -> String {
        format!("school_{}", school_id)
    }
}
