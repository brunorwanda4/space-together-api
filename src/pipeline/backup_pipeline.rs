use mongodb::bson::{doc, Document};

pub fn backup_pipeline(match_stage: Document) -> Vec<Document> {
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
                "school_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$school_id" }, "string"] },
                        { "$toObjectId": "$school_id" },
                        "$school_id"
                    ]
                },
                "created_by": {
                    "$cond": [
                        { "$eq": [{ "$type": "$created_by" }, "string"] },
                        { "$toObjectId": "$created_by" },
                        "$created_by"
                    ]
                }
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
        // CREATOR (user who created backup)
        // ======================================================
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "created_by",
                "foreignField": "_id",
                "as": "creator"
            }
        },
        doc! {
            "$unwind": {
                "path": "$creator",
                "preserveNullAndEmptyArrays": true
            }
        },
        // ======================================================
        // SORT
        // ======================================================
        doc! {
            "$sort": { "created_at": -1 }
        },
    ]
}
