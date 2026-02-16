use mongodb::bson::{doc, Document};

pub fn role_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! {
            "$match": match_stage
        },
        doc! {
            "$addFields": {
                "school_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$school_id" }, "string"] },
                        { "$toObjectId": "$school_id" },
                        "$school_id"
                    ]
                }
            }
        },
        doc! {
            "$lookup": {
                "from": "schools",
                "localField": "school_id",
                "foreignField": "_id",
                "as": "school"
            }
        },
        doc! {
            "$unwind": {
                "path": "$school",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$sort": { "created_at": -1 }
        },
    ]
}
