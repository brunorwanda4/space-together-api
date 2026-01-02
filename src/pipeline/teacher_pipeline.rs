use mongodb::bson::{doc, Document};

pub fn teacher_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        // ======================================================
        // MATCH
        // ======================================================
        doc! { "$match": match_stage },
        // ======================================================
        // NORMALIZE IDS
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
                "creator_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$creator_id" }, "string"] },
                        { "$toObjectId": "$creator_id" },
                        "$creator_id"
                    ]
                },
                "school_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$school_id" }, "string"] },
                        { "$toObjectId": "$school_id" },
                        "$school_id"
                    ]
                },
                "class_ids": {
                    "$map": {
                        "input": { "$ifNull": ["$class_ids", []] },
                        "as": "cid",
                        "in": {
                            "$cond": [
                                { "$eq": [{ "$type": "$$cid" }, "string"] },
                                { "$toObjectId": "$$cid" },
                                "$$cid"
                            ]
                        }
                    }
                },
                "subject_ids": {
                    "$map": {
                        "input": { "$ifNull": ["$subject_ids", []] },
                        "as": "sid",
                        "in": {
                            "$cond": [
                                { "$eq": [{ "$type": "$$sid" }, "string"] },
                                { "$toObjectId": "$$sid" },
                                "$$sid"
                            ]
                        }
                    }
                }
            }
        },
        // ======================================================
        // USER LOOKUP (ACCOUNT)
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
        // CREATOR LOOKUP
        // ======================================================
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "creator_id",
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
        // SCHOOL LOOKUP
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
        // CLASSES LOOKUP
        // ======================================================
        doc! {
            "$lookup": {
                "from": "classes",
                "localField": "class_ids",
                "foreignField": "_id",
                "as": "classes"
            }
        },
        // ======================================================
        // SUBJECTS LOOKUP
        // ======================================================
        doc! {
            "$lookup": {
                "from": "subjects",
                "localField": "subject_ids",
                "foreignField": "_id",
                "as": "subjects"
            }
        },
        // ======================================================
        // SORT
        // ======================================================
        doc! { "$sort": { "created_at": -1 } },
    ]
}
