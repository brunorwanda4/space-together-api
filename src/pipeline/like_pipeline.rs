use mongodb::bson::{doc, Document};

pub fn like_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        doc! {
            "$addFields": {
                "actor.id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$actor.id" }, "string"] },
                        { "$toObjectId": "$actor.id" },
                        "$actor.id"
                    ]
                }
            }
        },
        doc! {
            "$lookup": {
                "from": "students",
                "let": { "uid": "$actor.id", "role": "$actor.role" },
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
                "as": "author_student"
            }
        },
        doc! {
            "$lookup": {
                "from": "teachers",
                "let": { "uid": "$actor.id", "role": "$actor.role" },
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
                "as": "author_teacher"
            }
        },
        doc! {
            "$lookup": {
                "from": "school_staff",
                "let": { "uid": "$actor.id", "role": "$actor.role" },
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
                "as": "author_staff"
            }
        },
        doc! {
            "$lookup": {
                "from": "parents",
                "let": { "uid": "$actor.id", "role": "$actor.role" },
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
                "as": "author_parent"
            }
        },
        doc! {
            "$addFields": {
                "author_user": {
                    "$first": {
                        "$concatArrays": [
                            "$author_student",
                            "$author_teacher",
                            "$author_staff",
                            "$author_parent"
                        ]
                    }
                }
            }
        },
        doc! {
            "$project": {
                "author_student": 0,
                "author_teacher": 0,
                "author_staff": 0,
                "author_parent": 0
            }
        },
        doc! { "$sort": { "created_at": -1 } },
    ]
}
