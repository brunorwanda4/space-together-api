use mongodb::bson::{doc, Document};

pub fn message_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        doc! { "$sort": { "created_at": -1 } },
    ]
}
