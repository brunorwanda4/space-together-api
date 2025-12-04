use mongodb::bson::{doc, Document};

pub fn class_subject_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        // Convert all string IDs to ObjectId
        doc! {
            "$addFields": {
                "created_by": {
                    "$cond": [
                        {
                            "$and": [
                                {"$ne": ["$created_by", null]},
                                {"$eq": [{"$type": "$created_by"}, "string"]}
                            ]
                        },
                        { "$toObjectId": "$created_by" },
                        "$created_by"
                    ]
                },
                "teacher_id": {
                    "$cond": [
                        {
                            "$and": [
                                {"$ne": ["$teacher_id", null]},
                                {"$eq": [{"$type": "$teacher_id"}, "string"]}
                            ]
                        },
                        { "$toObjectId": "$teacher_id" },
                        "$teacher_id"
                    ]
                },
                "school_id": {
                    "$cond": [
                        {
                            "$and": [
                                {"$ne": ["$school_id", null]},
                                {"$eq": [{"$type": "$school_id"}, "string"]}
                            ]
                        },
                        { "$toObjectId": "$school_id" },
                        "$school_id"
                    ]
                },
                "class_id": {
                    "$cond": [
                        {
                            "$and": [
                                {"$ne": ["$class_id", null]},
                                {"$eq": [{"$type": "$class_id"}, "string"]}
                            ]
                        },
                        { "$toObjectId": "$class_id" },
                        "$class_id"
                    ]
                },
                "main_subject_id": {
                    "$cond": [
                        {
                            "$and": [
                                {"$ne": ["$main_subject_id", null]},
                                {"$eq": [{"$type": "$main_subject_id"}, "string"]}
                            ]
                        },
                        { "$toObjectId": "$main_subject_id" },
                        "$main_subject_id"
                    ]
                }
            }
        },
        // JOIN teacher
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "teacher_id",
                "foreignField": "_id",
                "as": "teacher"
            }
        },
        doc! { "$unwind": { "path": "$teacher", "preserveNullAndEmptyArrays": true } },
        // JOIN class
        doc! {
            "$lookup": {
                "from": "classes",
                "localField": "class_id",
                "foreignField": "_id",
                "as": "class"
            }
        },
        doc! { "$unwind": { "path": "$class_info", "preserveNullAndEmptyArrays": true } },
        // JOIN school
        doc! {
            "$lookup": {
                "from": "schools",
                "localField": "school_id",
                "foreignField": "_id",
                "as": "school"
            }
        },
        doc! { "$unwind": { "path": "$school", "preserveNullAndEmptyArrays": true } },
        // JOIN template subject
        doc! {
            "$lookup": {
                "from": "template_subjects",
                "localField": "main_subject_id",
                "foreignField": "_id",
                "as": "main_template_subject"
            }
        },
        doc! { "$unwind": { "path": "$main_template_subject", "preserveNullAndEmptyArrays": true } },
        doc! { "$sort": { "updated_at": -1 } },
    ]
}
