use mongodb::bson::{doc, Document};

pub fn template_subject_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "created_by",
                "foreignField": "_id",
                "as": "creator"
            }
        },
        doc! { "$unwind": { "path": "$creator", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$lookup": {
                "from": "main_classes",
                "localField": "prerequisites",
                "foreignField": "_id",
                "as": "prerequisites"
            }
        },
        // (No unwind because prerequisites is an array — keep it as array)

        // 4. Optionally sort (remove if not needed)
        doc! { "$sort": { "updated_at": -1 } },
    ]
}
