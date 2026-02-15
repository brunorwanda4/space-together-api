use mongodb::bson::{doc, Document};

pub fn assignment_with_teacher_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! {
            "$match": match_stage
        },
        // Normalize ObjectIds
        doc! {
            "$addFields": {
                "teacher_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$teacher_id" }, "string"] },
                        { "$toObjectId": "$teacher_id" },
                        "$teacher_id"
                    ]
                },
                "subject_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$subject_id" }, "string"] },
                        { "$toObjectId": "$subject_id" },
                        "$subject_id"
                    ]
                },
                "class_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$class_id" }, "string"] },
                        { "$toObjectId": "$class_id" },
                        "$class_id"
                    ]
                },
                "school_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$school_id" }, "string"] },
                        { "$toObjectId": "$school_id" },
                        "$school_id"
                    ]
                }
            }
        },
        // Lookup teacher
        doc! {
            "$lookup": {
                "from": "teachers",
                "localField": "teacher_id",
                "foreignField": "_id",
                "as": "teacher"
            }
        },
        doc! {
            "$unwind": {
                "path": "$teacher",
                "preserveNullAndEmptyArrays": true
            }
        },
        // Lookup subject
        doc! {
            "$lookup": {
                "from": "class_subjects",
                "localField": "subject_id",
                "foreignField": "_id",
                "as": "subject"
            }
        },
        doc! {
            "$unwind": {
                "path": "$subject",
                "preserveNullAndEmptyArrays": true
            }
        },
        // Lookup class
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
        // Count submissions
        doc! {
            "$lookup": {
                "from": "submissions",
                "localField": "_id",
                "foreignField": "assignment_id",
                "as": "submissions"
            }
        },
        doc! {
            "$addFields": {
                "submission_count": { "$size": "$submissions" }
            }
        },
        // Count total students in class
        doc! {
            "$lookup": {
                "from": "students",
                "let": { "classId": "$class_id" },
                "pipeline": [
                    {
                        "$match": {
                            "$expr": { "$eq": ["$class_id", "$$classId"] },
                            "is_active": true
                        }
                    }
                ],
                "as": "students"
            }
        },
        doc! {
            "$addFields": {
                "total_students": { "$size": "$students" }
            }
        },
        // Remove temporary fields
        doc! {
            "$project": {
                "submissions": 0,
                "students": 0
            }
        },
        doc! {
            "$sort": { "created_at": -1 }
        },
    ]
}

pub fn submission_with_relations_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! {
            "$match": match_stage
        },
        // Normalize ObjectIds
        doc! {
            "$addFields": {
                "student_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$student_id" }, "string"] },
                        { "$toObjectId": "$student_id" },
                        "$student_id"
                    ]
                },
                "assignment_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$assignment_id" }, "string"] },
                        { "$toObjectId": "$assignment_id" },
                        "$assignment_id"
                    ]
                },
                "graded_by": {
                    "$cond": [
                        { "$eq": [{ "$type": "$graded_by" }, "string"] },
                        { "$toObjectId": "$graded_by" },
                        "$graded_by"
                    ]
                }
            }
        },
        // Lookup student
        doc! {
            "$lookup": {
                "from": "students",
                "localField": "student_id",
                "foreignField": "_id",
                "as": "student"
            }
        },
        doc! {
            "$unwind": {
                "path": "$student",
                "preserveNullAndEmptyArrays": true
            }
        },
        // Lookup assignment
        doc! {
            "$lookup": {
                "from": "assignments",
                "localField": "assignment_id",
                "foreignField": "_id",
                "as": "assignment"
            }
        },
        doc! {
            "$unwind": {
                "path": "$assignment",
                "preserveNullAndEmptyArrays": true
            }
        },
        // Lookup graded_by teacher
        doc! {
            "$lookup": {
                "from": "teachers",
                "localField": "graded_by",
                "foreignField": "_id",
                "as": "graded_by_teacher"
            }
        },
        doc! {
            "$unwind": {
                "path": "$graded_by_teacher",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$sort": { "submitted_at": -1 }
        },
    ]
}
