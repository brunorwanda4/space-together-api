use mongodb::bson::{doc, Document};

pub fn student_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        // ======================================================
        // MATCH
        // ======================================================
        doc! {
            "$match": match_stage
        },
        // ======================================================
        // FILTER OUT SOFT DELETED
        // ======================================================
        doc! {
            "$match": {
                "deleted_at": { "$eq": null }
            }
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
                "class_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$class_id" }, "string"] },
                        { "$toObjectId": "$class_id" },
                        "$class_id"
                    ]
                },
                "subclass_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$subclass_id" }, "string"] },
                        { "$toObjectId": "$subclass_id" },
                        "$subclass_id"
                    ]
                }
            }
        },
        // ======================================================
        // USER (linked account)
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
        // CREATOR (admin / staff who created student)
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
        // CLASS
        // ======================================================
        doc! {
            "$lookup": {
                "from": "classes",
                "localField": "class_id",
                "foreignField": "_id",
                "as": "class"
            }
        },
        doc! {
            "$unwind": {
                "path": "$class",
                "preserveNullAndEmptyArrays": true
            }
        },
        // ======================================================
        // SUBCLASS (optional)
        // ======================================================
        doc! {
            "$lookup": {
                "from": "classes",
                "localField": "subclass_id",
                "foreignField": "_id",
                "as": "subclass"
            }
        },
        doc! {
            "$unwind": {
                "path": "$subclass",
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
