use mongodb::bson::{doc, Document};

pub fn class_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        // =========================
        // MATCH
        // =========================
        doc! { "$match": match_stage },
        // =========================
        // NORMALIZE IDS
        // =========================
        doc! {
            "$addFields": {
                "school_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$school_id" }, "string"] },
                        { "$toObjectId": "$school_id" },
                        "$school_id"
                    ]
                },
                "creator_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$creator_id" }, "string"] },
                        { "$toObjectId": "$creator_id" },
                        "$creator_id"
                    ]
                },
                "class_teacher_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$class_teacher_id" }, "string"] },
                        { "$toObjectId": "$class_teacher_id" },
                        "$class_teacher_id"
                    ]
                },
                "main_class_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$main_class_id" }, "string"] },
                        { "$toObjectId": "$main_class_id" },
                        "$main_class_id"
                    ]
                },
                "trade_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$trade_id" }, "string"] },
                        { "$toObjectId": "$trade_id" },
                        "$trade_id"
                    ]
                }
            }
        },
        // =========================
        // SCHOOL
        // =========================
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
        // =========================
        // CREATOR
        // =========================
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
        // =========================
        // CLASS TEACHER
        // =========================
        doc! {
            "$lookup": {
                "from": "teachers",
                "localField": "class_teacher_id",
                "foreignField": "_id",
                "as": "class_teacher"
            }
        },
        doc! {
            "$unwind": {
                "path": "$class_teacher",
                "preserveNullAndEmptyArrays": true
            }
        },
        // =========================
        // MAIN CLASS
        // =========================
        doc! {
            "$lookup": {
                "from": "main_classes",
                "localField": "main_class_id",
                "foreignField": "_id",
                "as": "main_class"
            }
        },
        doc! {
            "$unwind": {
                "path": "$main_class",
                "preserveNullAndEmptyArrays": true
            }
        },
        // =========================
        // TRADE
        // =========================
        doc! {
            "$lookup": {
                "from": "trades",
                "localField": "trade_id",
                "foreignField": "_id",
                "as": "trade"
            }
        },
        doc! {
            "$unwind": {
                "path": "$trade",
                "preserveNullAndEmptyArrays": true
            }
        },
        // =========================
        // SORT
        // =========================
        doc! { "$sort": { "created_at": -1 } },
    ]
}
