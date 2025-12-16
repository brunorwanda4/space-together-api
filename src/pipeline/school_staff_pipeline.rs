use mongodb::bson::{doc, Document};

pub fn school_staff_with_relations_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        // Lookup user
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "user_id",
                "foreignField": "_id",
                "as": "user"
            }
        },
        doc! { "$unwind": { "path": "$user", "preserveNullAndEmptyArrays": true } },
        // Lookup school
        doc! {
            "$lookup": {
                "from": "schools",
                "localField": "school_id",
                "foreignField": "_id",
                "as": "school"
            }
        },
        doc! { "$unwind": { "path": "$school", "preserveNullAndEmptyArrays": true } },
        // Lookup creator (user)
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "creator_id",
                "foreignField": "_id",
                "as": "creator"
            }
        },
        doc! { "$unwind": { "path": "$creator", "preserveNullAndEmptyArrays": true } },
    ]
}
