use crate::config::mongo_manager::MongoManager;
use anyhow::Result;

pub struct TenantService;

impl TenantService {
    /// Create collections, indexes and optionally copy templates from main DB
    pub async fn initialize_school_db(mongo: &MongoManager, school_db_name: &str) -> Result<()> {
        // get handle to target school db
        let db = mongo.get_db(school_db_name);

        // 1) Create collections (idempotent)
        // CreateCollection will error if exists, but caller can ignore
        let _ = db.create_collection("students").await;
        let _ = db.create_collection("teachers").await;
        let _ = db.create_collection("classes").await;
        let _ = db.create_collection("subjects").await;
        let _ = db.create_collection("notes").await;
        let _ = db.create_collection("assessments").await;
        let _ = db.create_collection("posts").await;

        Ok(())
    }
}
