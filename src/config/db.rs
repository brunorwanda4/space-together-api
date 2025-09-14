use mongodb::{options::ClientOptions, Client, Database};
use std::env;

pub async fn init_db() -> Database {
    let uri = env::var("MONGO_URI").expect("❌ MONGO_URI not set in .env");
    let options = ClientOptions::parse(uri.clone()).await.unwrap();
    let client = Client::with_options(options).unwrap();

    println!("✅ Connected to MongoDB");
    client.database("space_together")
}
