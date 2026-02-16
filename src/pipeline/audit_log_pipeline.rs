use mongodb::bson::{doc, Document};

pub fn audit_log_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        // ======================================================
        // MATCH
        // ======================================================
        doc! {
            "$match": match_stage
        },
        // ======================================================
        // NORMALIZE OBJECT IDS (string -> ObjectId safety)
        // ======================================================
        doc! {
            "$addFields": {
                "user_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$user_id" }, "string"] },
                        { "$toObjectId": "$user_id" },
                        "$user_id"
                    ]
                },
                "school_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$school_id" }, "string"] },
                        { "$toObjectId": "$school_id" },
                        "$school_id"
                    ]
                },
                "entity_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$entity_id" }, "string"] },
                        { "$toObjectId": "$entity_id" },
                        "$entity_id"
                    ]
                }
            }
        },
        // ======================================================
        // USER (who performed the action)
        // ======================================================
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "user_id",
                "foreignField": "_id",
                "as": "user"
            }
        },
        doc! {
            "$unwind": {
                "path": "$user",
                "preserveNullAndEmptyArrays": true
            }
        },
        // ======================================================
        // SCHOOL
        // ======================================================
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
        // ======================================================
        // SORT (most recent first)
        // ======================================================
        doc! {
            "$sort": { "created_at": -1 }
        },
    ]
}
