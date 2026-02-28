use mongodb::bson::{doc, Document};

pub fn conversation_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        // Unwind participants to lookup each one
        doc! { "$unwind": { "path": "$participants", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$addFields": {
                "participants.id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$participants.id" }, "string"] },
                        { "$toObjectId": "$participants.id" },
                        "$participants.id"
                    ]
                }
            }
        },
        // Lookup students
        doc! {
            "$lookup": {
                "from": "students",
                "let": { "uid": "$participants.id", "role": "$participants.role" },
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
                "as": "participant_student"
            }
        },
        // Lookup teachers
        doc! {
            "$lookup": {
                "from": "teachers",
                "let": { "uid": "$participants.id", "role": "$participants.role" },
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
                "as": "participant_teacher"
            }
        },
        // Lookup school staff
        doc! {
            "$lookup": {
                "from": "school_staff",
                "let": { "uid": "$participants.id", "role": "$participants.role" },
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
                "as": "participant_staff"
            }
        },
        // Lookup parents
        doc! {
            "$lookup": {
                "from": "parents",
                "let": { "uid": "$participants.id", "role": "$participants.role" },
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
                "as": "participant_parent"
            }
        },
        // Combine all participant lookups
        doc! {
            "$addFields": {
                "participant_user": {
                    "$first": {
                        "$concatArrays": [
                            "$participant_student",
                            "$participant_teacher",
                            "$participant_staff",
                            "$participant_parent"
                        ]
                    }
                }
            }
        },
        // Group back to reconstruct the conversation with all participants
        doc! {
            "$group": {
                "_id": "$_id",
                "school_id": { "$first": "$school_id" },
                "participants": { "$first": "$participants" },
                "is_group": { "$first": "$is_group" },
                "name": { "$first": "$name" },
                "encryption_key_version": { "$first": "$encryption_key_version" },
                "created_at": { "$first": "$created_at" },
                "updated_at": { "$first": "$updated_at" },
                "participants_users": { "$push": "$participant_user" }
            }
        },
        // Reconstruct the conversation document
        doc! {
            "$project": {
                "_id": 1,
                "school_id": 1,
                "participants": 1,
                "is_group": 1,
                "name": 1,
                "encryption_key_version": 1,
                "created_at": 1,
                "updated_at": 1,
                "participants_users": {
                    "$filter": {
                        "input": "$participants_users",
                        "as": "user",
                        "cond": { "$ne": ["$$user", null] }
                    }
                }
            }
        },
        doc! { "$sort": { "updated_at": -1 } },
    ]
}
