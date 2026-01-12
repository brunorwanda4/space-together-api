use mongodb::bson::{doc, Document};

pub fn announcement_pipeline(match_stage: Document) -> Vec<Document> {
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
                "published.id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$published.id" }, "string"] },
                        { "$toObjectId": "$published.id" },
                        "$published.id"
                    ]
                },

                "classes_ids": {
                    "$map": {
                        "input": { "$ifNull": ["$classes_ids", []] },
                        "as": "cid",
                        "in": {
                            "$cond": [
                                { "$eq": [{ "$type": "$$cid" }, "string"] },
                                { "$toObjectId": "$$cid" },
                                "$$cid"
                            ]
                        }
                    }
                }
            }
        },

        // ======================================================
        // NORMALIZE MENTIONS
        // ======================================================
        doc! {
            "$addFields": {
                "mention": {
                    "$map": {
                        "input": { "$ifNull": ["$mention", []] },
                        "as": "m",
                        "in": {
                            "id": {
                                "$cond": [
                                    { "$eq": [{ "$type": "$$m.id" }, "string"] },
                                    { "$toObjectId": "$$m.id" },
                                    "$$m.id"
                                ]
                            },
                            "role": "$$m.role"
                        }
                    }
                }
            }
        },

        // ======================================================
        // PUBLISHED USER (ROLE-AWARE)
        // ======================================================
        doc! {
            "$lookup": {
                "from": "students",
                "let": { "uid": "$published.id", "role": "$published.role" },
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
                "as": "published_student"
            }
        },
        doc! {
            "$lookup": {
                "from": "teachers",
                "let": { "uid": "$published.id", "role": "$published.role" },
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
                "as": "published_teacher"
            }
        },
        doc! {
            "$lookup": {
                "from": "school_staff",
                "let": { "uid": "$published.id", "role": "$published.role" },
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
                "as": "published_staff"
            }
        },

        // ======================================================
        // MERGE PUBLISHED USER
        // ======================================================
        doc! {
            "$addFields": {
                "published_user": {
                    "$first": {
                        "$concatArrays": [
                            "$published_student",
                            "$published_teacher",
                            "$published_staff"
                        ]
                    }
                }
            }
        },

        // ======================================================
        // MENTIONED USERS
        // ======================================================
        doc! {
            "$lookup": {
                "from": "students",
                "localField": "mention.id",
                "foreignField": "_id",
                "pipeline": [{ "$addFields": { "user_type": "STUDENT" } }],
                "as": "mentioned_students"
            }
        },
        doc! {
            "$lookup": {
                "from": "teachers",
                "localField": "mention.id",
                "foreignField": "_id",
                "pipeline": [{ "$addFields": { "user_type": "TEACHER" } }],
                "as": "mentioned_teachers"
            }
        },
        doc! {
            "$lookup": {
                "from": "school_staff",
                "localField": "mention.id",
                "foreignField": "_id",
                "pipeline": [{ "$addFields": { "user_type": "SCHOOLSTAFF" } }],
                "as": "mentioned_staff"
            }
        },
        doc! {
            "$addFields": {
                "mentioned_users": {
                    "$concatArrays": [
                        "$mentioned_students",
                        "$mentioned_teachers",
                        "$mentioned_staff"
                    ]
                }
            }
        },

        // ======================================================
        // CLASSES LOOKUP (ARRAY)
        // ======================================================
        doc! {
            "$lookup": {
                "from": "classes",
                "localField": "classes_ids",
                "foreignField": "_id",
                "as": "classes"
            }
        },

        // ======================================================
        // CLEANUP
        // ======================================================
        doc! {
            "$project": {
                "published_student": 0,
                "published_teacher": 0,
                "published_staff": 0,
                "mentioned_students": 0,
                "mentioned_teachers": 0,
                "mentioned_staff": 0
            }
        },

        // ======================================================
        // SORT
        // ======================================================
        doc! { "$sort": { "updated_at": -1 } },
    ]
}
