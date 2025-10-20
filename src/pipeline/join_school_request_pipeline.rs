use mongodb::bson::{doc, Document};

pub fn join_school_request_with_relations_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
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
        doc! { "$sort": { "created_at": -1 } },
    ]
}
