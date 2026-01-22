use mongodb::bson::{doc, Document};

pub fn join_school_request_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        doc! {
            "$addFields": {
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
                "invited_user_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$invited_user_id" }, "string"] },
                        { "$toObjectId": "$invited_user_id" },
                        "$invited_user_id"
                    ]
                },
                "sent_by": {
                    "$cond": [
                        { "$eq": [{ "$type": "$sent_by" }, "string"] },
                        { "$toObjectId": "$sent_by" },
                        "$sent_by"
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
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "invited_user_id",
                "foreignField": "_id",
                "as": "invited_user"
            }
        },
        doc! {
            "$unwind": {
                "path": "$invited_user",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "sent_by",
                "foreignField": "_id",
                "as": "sender"
            }
        },
        doc! {
            "$unwind": {
                "path": "$sender",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! { "$sort": { "updated_at": -1 } },
    ]
}
