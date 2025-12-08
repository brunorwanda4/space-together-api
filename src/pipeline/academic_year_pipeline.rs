use mongodb::bson::{doc, Document};

pub fn academic_year_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        // convert curriculum_id string → ObjectId
        doc! {
            "$addFields": {
                "curriculum_id": {
                    "$cond": [
                        { "$and": [
                            { "$ne": ["$curriculum_id", null] },
                            { "$eq": [ { "$type": "$curriculum_id"}, "string" ] }
                        ]},
                        { "$toObjectId": "$curriculum_id" },
                        "$curriculum_id"
                    ]
                }
            }
        },
        // lookup curriculum (Sector)
        doc! {
            "$lookup": {
                "from": "sectors",
                "localField": "curriculum_id",
                "foreignField": "_id",
                "as": "curriculum"
            }
        },
        doc! {
            "$unwind": {
                "path": "$curriculum",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! { "$sort": { "created_at": -1 } },
    ]
}
