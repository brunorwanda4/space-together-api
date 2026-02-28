use mongodb::bson::{doc, Document};

pub fn comment_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        doc! {
            "$addFields": {
                "author.id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$author.id" }, "string"] },
                        { "$toObjectId": "$author.id" },
                        "$author.id"
                    ]
                }
            }
        },
        doc! {
            "$lookup": {
                "from": "students",
                "let": { "uid": "$author.id", "role": "$author.role" },
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
                "let": { "uid": "$author.id", "role": "$author.role" },
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
                "let": { "uid": "$author.id", "role": "$author.role" },
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
                "let": { "uid": "$author.id", "role": "$author.role" },
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
        doc! { "$sort": { "updated_at": -1 } },
    ]
}
