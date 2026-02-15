use mongodb::bson::{doc, Document};

pub fn parent_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        // ======================================================
        // MATCH
        // ======================================================
        doc! {
            "$match": match_stage
        },
        // ======================================================
        // NORMALIZE OBJECT IDS (string -> ObjectId safety)
        // ======================================================
        doc! {
            "$addFields": {
                "user_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$user_id" }, "string"] },
                        { "$toObjectId": "$user_id" },
                        "$user_id"
                    ]
                },
                "school_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$school_id" }, "string"] },
                        { "$toObjectId": "$school_id" },
                        "$school_id"
                    ]
                },
                "student_ids": {
                    "$map": {
                        "input": { "$ifNull": ["$student_ids", []] },
                        "as": "sid",
                        "in": {
                            "$cond": [
                                { "$eq": [{ "$type": "$$sid" }, "string"] },
                                { "$toObjectId": "$$sid" },
                                "$$sid"
                            ]
                        }
                    }
                }
            }
        },
        // ======================================================
        // USER (linked account)
        // ======================================================
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "user_id",
                "foreignField": "_id",
                "as": "user"
            }
        },
        doc! {
            "$unwind": {
                "path": "$user",
                "preserveNullAndEmptyArrays": true
            }
        },
        // ======================================================
        // SCHOOL
        // ======================================================
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
        // ======================================================
        // STUDENTS (children)
        // ======================================================
        doc! {
            "$lookup": {
                "from": "students",
                "localField": "student_ids",
                "foreignField": "_id",
                "as": "students"
            }
        },
        // ======================================================
        // SORT
        // ======================================================
        doc! {
            "$sort": { "created_at": -1 }
        },
    ]
}

/// Pipeline for parent dashboard aggregation
pub fn parent_dashboard_pipeline(parent_id: &str, school_id: &str) -> Vec<Document> {
    vec![
        doc! {
            "$match": {
                "_id": { "$toObjectId": parent_id },
                "school_id": { "$toObjectId": school_id }
            }
        },
        doc! {
            "$lookup": {
                "from": "students",
                "localField": "student_ids",
                "foreignField": "_id",
                "as": "children"
            }
        },
        doc! {
            "$unwind": {
                "path": "$children",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$lookup": {
                "from": "classes",
                "localField": "children.class_id",
                "foreignField": "_id",
                "as": "children.class"
            }
        },
        doc! {
            "$unwind": {
                "path": "$children.class",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$group": {
                "_id": "$_id",
                "parent": { "$first": "$$ROOT" },
                "children": { "$push": "$children" }
            }
        },
    ]
}

/// Pipeline for attendance summary
pub fn attendance_summary_pipeline(student_id: &str, school_id: &str) -> Vec<Document> {
    vec![
        doc! {
            "$match": {
                "student_id": { "$toObjectId": student_id },
                "school_id": { "$toObjectId": school_id }
            }
        },
        doc! {
            "$facet": {
                "summary": [
                    {
                        "$group": {
                            "_id": "$status",
                            "count": { "$sum": 1 }
                        }
                    }
                ],
                "recent": [
                    { "$sort": { "date": -1 } },
                    { "$limit": 10 },
                    {
                        "$project": {
                            "date": 1,
                            "status": 1,
                            "remarks": 1
                        }
                    }
                ]
            }
        },
    ]
}

/// Pipeline for student results
pub fn student_results_pipeline(
    student_id: &str,
    school_id: &str,
    education_year_id: Option<&str>,
    term_id: Option<&str>,
) -> Vec<Document> {
    let mut match_doc = doc! {
        "student_id": { "$toObjectId": student_id },
        "school_id": { "$toObjectId": school_id }
    };

    if let Some(year_id) = education_year_id {
        match_doc.insert("education_year_id", doc! { "$toObjectId": year_id });
    }

    if let Some(tid) = term_id {
        match_doc.insert("term_id", tid);
    }

    vec![
        doc! {
            "$match": match_doc
        },
        doc! {
            "$lookup": {
                "from": "exams",
                "localField": "exam_id",
                "foreignField": "_id",
                "as": "exam"
            }
        },
        doc! {
            "$unwind": {
                "path": "$exam",
                "preserveNullAndEmptyArrays": true
            }
        },
        doc! {
            "$sort": { "calculated_at": -1 }
        },
        doc! {
            "$limit": 1
        },
    ]
}

/// Pipeline for finance summary
pub fn finance_summary_pipeline(student_id: &str, school_id: &str) -> Vec<Document> {
    vec![
        doc! {
            "$match": {
                "student_id": { "$toObjectId": student_id },
                "school_id": { "$toObjectId": school_id }
            }
        },
        doc! {
            "$facet": {
                "payments": [
                    {
                        "$lookup": {
                            "from": "payments",
                            "localField": "_id",
                            "foreignField": "enrollment_id",
                            "as": "payment_records"
                        }
                    },
                    {
                        "$unwind": {
                            "path": "$payment_records",
                            "preserveNullAndEmptyArrays": true
                        }
                    },
                    {
                        "$group": {
                            "_id": null,
                            "total_paid": { "$sum": "$payment_records.amount" },
                            "payments": { "$push": "$payment_records" }
                        }
                    }
                ],
                "fees": [
                    {
                        "$lookup": {
                            "from": "fee_structures",
                            "localField": "fee_structure_id",
                            "foreignField": "_id",
                            "as": "fee_structure"
                        }
                    },
                    {
                        "$unwind": {
                            "path": "$fee_structure",
                            "preserveNullAndEmptyArrays": true
                        }
                    },
                    {
                        "$group": {
                            "_id": null,
                            "total_required": { "$sum": "$fee_structure.total_amount" }
                        }
                    }
                ]
            }
        },
    ]
}
