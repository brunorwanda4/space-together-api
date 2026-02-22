use chrono::{DateTime, Utc};
use mongodb::bson::{doc, Document};

// ========== ENROLLMENT TRENDS PIPELINE ==========
pub fn enrollment_trends_pipeline(
    school_id: mongodb::bson::oid::ObjectId,
    year: Option<i32>,
) -> Vec<Document> {
    let mut pipeline = vec![
        // Match school and non-deleted students
        doc! {
            "$match": {
                "school_id": school_id,
                "deleted_at": { "$eq": null }
            }
        },
    ];

    // Optional year filter
    if let Some(y) = year {
        pipeline.push(doc! {
            "$match": {
                "$expr": {
                    "$eq": [{ "$year": "$created_at" }, y]
                }
            }
        });
    }

    pipeline.extend(vec![
        // Extract year and month
        doc! {
            "$project": {
                "year": { "$year": "$created_at" },
                "month": { "$month": "$created_at" }
            }
        },
        // Group by year-month
        doc! {
            "$group": {
                "_id": {
                    "year": "$year",
                    "month": "$month"
                },
                "total": { "$sum": 1 }
            }
        },
        // Format month as YYYY-MM
        doc! {
            "$project": {
                "_id": 0,
                "month": {
                    "$concat": [
                        { "$toString": "$_id.year" },
                        "-",
                        {
                            "$cond": [
                                { "$lt": ["$_id.month", 10] },
                                { "$concat": ["0", { "$toString": "$_id.month" }] },
                                { "$toString": "$_id.month" }
                            ]
                        }
                    ]
                },
                "total": 1
            }
        },
        // Sort by month
        doc! {
            "$sort": { "month": 1 }
        },
    ]);

    pipeline
}

// ========== ATTENDANCE RATE PIPELINE ==========
pub fn attendance_rate_pipeline(
    school_id: mongodb::bson::oid::ObjectId,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
) -> Vec<Document> {
    let mut match_doc = doc! {
        "school_id": school_id
    };

    // Add date range filter if provided
    if from.is_some() || to.is_some() {
        let mut date_filter = doc! {};
        if let Some(from_date) = from {
            date_filter.insert("$gte", mongodb::bson::to_bson(&from_date).unwrap());
        }
        if let Some(to_date) = to {
            date_filter.insert("$lte", mongodb::bson::to_bson(&to_date).unwrap());
        }
        match_doc.insert("date", date_filter);
    }

    vec![
        doc! {
            "$match": match_doc
        },
        doc! {
            "$group": {
                "_id": null,
                "total_records": { "$sum": 1 },
                "present_count": {
                    "$sum": {
                        "$cond": [
                            { "$eq": ["$status", "Present"] },
                            1,
                            0
                        ]
                    }
                }
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "total_records": 1,
                "present_count": 1,
                "attendance_rate": {
                    "$cond": [
                        { "$eq": ["$total_records", 0] },
                        0.0,
                        {
                            "$multiply": [
                                { "$divide": ["$present_count", "$total_records"] },
                                100.0
                            ]
                        }
                    ]
                }
            }
        },
    ]
}

// ========== PASS/FAIL DISTRIBUTION PIPELINE ==========
pub fn pass_fail_distribution_pipeline(
    school_id: mongodb::bson::oid::ObjectId,
    passing_mark: f64,
) -> Vec<Document> {
    vec![
        doc! {
            "$match": {
                "school_id": school_id,
                "is_deleted": { "$ne": true }
            }
        },
        doc! {
            "$group": {
                "_id": null,
                "pass": {
                    "$sum": {
                        "$cond": [
                            { "$gte": ["$percentage", passing_mark] },
                            1,
                            0
                        ]
                    }
                },
                "fail": {
                    "$sum": {
                        "$cond": [
                            { "$lt": ["$percentage", passing_mark] },
                            1,
                            0
                        ]
                    }
                },
                "total": { "$sum": 1 }
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "pass": 1,
                "fail": 1,
                "total": 1,
                "pass_rate": {
                    "$cond": [
                        { "$eq": ["$total", 0] },
                        0.0,
                        {
                            "$multiply": [
                                { "$divide": ["$pass", "$total"] },
                                100.0
                            ]
                        }
                    ]
                }
            }
        },
    ]
}

// ========== FEE COLLECTION SUMMARY PIPELINE ==========
pub fn fee_collection_summary_pipeline(
    school_id: mongodb::bson::oid::ObjectId,
) -> Vec<Document> {
    vec![
        doc! {
            "$match": {
                "school_id": school_id
            }
        },
        doc! {
            "$group": {
                "_id": null,
                "total_expected": { "$sum": "$amount" },
                "total_collected": {
                    "$sum": {
                        "$cond": [
                            { "$eq": ["$status", "Paid"] },
                            "$amount",
                            0.0
                        ]
                    }
                }
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "total_expected": 1,
                "total_collected": 1,
                "total_outstanding": {
                    "$subtract": ["$total_expected", "$total_collected"]
                },
                "collection_rate": {
                    "$cond": [
                        { "$eq": ["$total_expected", 0.0] },
                        0.0,
                        {
                            "$multiply": [
                                { "$divide": ["$total_collected", "$total_expected"] },
                                100.0
                            ]
                        }
                    ]
                }
            }
        },
    ]
}

// ========== TEACHER WORKLOAD DISTRIBUTION PIPELINE ==========
pub fn teacher_workload_pipeline(school_id: mongodb::bson::oid::ObjectId) -> Vec<Document> {
    vec![
        // Match teachers in the school
        doc! {
            "$match": {
                "school_id": school_id,
                "is_active": true
            }
        },
        // Normalize teacher_id
        doc! {
            "$addFields": {
                "teacher_id_obj": {
                    "$cond": [
                        { "$eq": [{ "$type": "$_id" }, "string"] },
                        { "$toObjectId": "$_id" },
                        "$_id"
                    ]
                }
            }
        },
        // Lookup classes taught
        doc! {
            "$lookup": {
                "from": "class_subjects",
                "let": { "teacher_id": "$teacher_id_obj" },
                "pipeline": [
                    {
                        "$match": {
                            "$expr": {
                                "$and": [
                                    { "$eq": ["$teacher_id", "$$teacher_id"] },
                                    { "$ne": ["$disable", true] }
                                ]
                            }
                        }
                    }
                ],
                "as": "subjects"
            }
        },
        // Get unique class IDs
        doc! {
            "$addFields": {
                "unique_classes": {
                    "$setUnion": ["$subjects.class_id", []]
                }
            }
        },
        // Lookup students in those classes
        doc! {
            "$lookup": {
                "from": "students",
                "let": { "class_ids": "$unique_classes" },
                "pipeline": [
                    {
                        "$match": {
                            "$expr": {
                                "$and": [
                                    { "$in": ["$class_id", "$$class_ids"] },
                                    { "$eq": ["$deleted_at", null] }
                                ]
                            }
                        }
                    }
                ],
                "as": "students"
            }
        },
        // Project final result
        doc! {
            "$project": {
                "_id": 0,
                "teacher_id": { "$toString": "$_id" },
                "teacher_name": "$name",
                "classes": { "$size": "$unique_classes" },
                "subjects": { "$size": "$subjects" },
                "total_students": { "$size": "$students" }
            }
        },
        // Sort by total students descending
        doc! {
            "$sort": { "total_students": -1 }
        },
    ]
}
