use mongodb::bson::{doc, Document};

pub fn message_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        doc! {
            "$addFields": {
                "sender.id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$sender.id" }, "string"] },
                        { "$toObjectId": "$sender.id" },
                        "$sender.id"
                    ]
                }
            }
        },
        doc! {
            "$lookup": {
                "from": "students",
                "let": { "uid": "$sender.id", "role": "$sender.role" },
                "pipeline": [
                    {
                        "$match": {
                            "$expr": {
                                "$and": [
                                    { "$eq": ["$$role", "STUDENT"] },
                                    { "$eq": ["$_id", "$$uid"] }
                                ]
                            }
                        }
                    },
                    { "$addFields": { "user_type": "STUDENT" } }
                ],
                "as": "sender_student"
            }
        },
        doc! {
            "$lookup": {
                "from": "teachers",
                "let": { "uid": "$sender.id", "role": "$sender.role" },
                "pipeline": [
                    {
                        "$match": {
                            "$expr": {
                                "$and": [
                                    { "$eq": ["$$role", "TEACHER"] },
                                    { "$eq": ["$_id", "$$uid"] }
                                ]
                            }
                        }
                    },
                    { "$addFields": { "user_type": "TEACHER" } }
                ],
                "as": "sender_teacher"
            }
        },
        doc! {
            "$lookup": {
                "from": "school_staff",
                "let": { "uid": "$sender.id", "role": "$sender.role" },
                "pipeline": [
                    {
                        "$match": {
                            "$expr": {
                                "$and": [
                                    { "$eq": ["$$role", "SCHOOLSTAFF"] },
                                    { "$eq": ["$_id", "$$uid"] }
                                ]
                            }
                        }
                    },
                    { "$addFields": { "user_type": "SCHOOLSTAFF" } }
                ],
                "as": "sender_staff"
            }
        },
        doc! {
            "$lookup": {
                "from": "parents",
                "let": { "uid": "$sender.id", "role": "$sender.role" },
                "pipeline": [
                    {
                        "$match": {
                            "$expr": {
                                "$and": [
                                    { "$eq": ["$$role", "PARENT"] },
                                    { "$eq": ["$_id", "$$uid"] }
                                ]
                            }
                        }
                    },
                    { "$addFields": { "user_type": "PARENT" } }
                ],
                "as": "sender_parent"
            }
        },
        doc! {
            "$addFields": {
                "sender_user": {
                    "$first": {
                        "$concatArrays": [
                            "$sender_student",
                            "$sender_teacher",
                            "$sender_staff",
                            "$sender_parent"
                        ]
                    }
                }
            }
        },
        doc! {
            "$project": {
                "sender_student": 0,
                "sender_teacher": 0,
                "sender_staff": 0,
                "sender_parent": 0
            }
        },
        doc! { "$sort": { "created_at": -1 } },
    ]
}
