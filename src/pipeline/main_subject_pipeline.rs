use mongodb::bson::{doc, Document};

pub fn main_subject_with_others_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        // Lookup learning outcomes
        doc! {
            "$lookup": {
                "from": "learning_outcomes",
                "localField": "_id",
                "foreignField": "subject_id",
                "as": "learning_outcome"
            }
        },
        // Lookup progress tracking config
        doc! {
            "$lookup": {
                "from": "subject_progress_configs",
                "let": { "subject_id": "$_id" },
                "pipeline": [
                    doc! {
                        "$match": {
                            "$expr": {
                                "$and": [
                                    { "$eq": ["$reference_id", "$$subject_id"] },
                                    { "$eq": ["$role", "MainSubject"] }
                                ]
                            }
                        }
                    }
                ],
                "as": "progress_tracking_config_arr"
            }
        },
        // Convert progress tracking config array to single object
        doc! {
            "$addFields": {
                "progress_tracking_config": {
                    "$arrayElemAt": ["$progress_tracking_config_arr", 0]
                }
            }
        },
        // Lookup grading schemes
        doc! {
            "$lookup": {
                "from": "subject_grading_schemes",
                "let": { "subject_id": "$_id" },
                "pipeline": [
                    doc! {
                        "$match": {
                            "$expr": {
                                "$and": [
                                    { "$eq": ["$reference_id", "$$subject_id"] },
                                    { "$eq": ["$role", "MainSubject"] }
                                ]
                            }
                        }
                    }
                ],
                "as": "grading_schemes_arr"
            }
        },
        // Convert grading schemes array to single object
        doc! {
            "$addFields": {
                "grading_schemes": {
                    "$arrayElemAt": ["$grading_schemes_arr", 0]
                }
            }
        },
        // Remove temporary arrays
        doc! {
            "$project": {
                "progress_tracking_config_arr": 0,
                "grading_schemes_arr": 0
            }
        },
        // Now lookup topics for each learning outcome and then learning materials for each topic
        doc! {
            "$addFields": {
                "learning_outcome": {
                    "$map": {
                        "input": "$learning_outcome",
                        "as": "lo",
                        "in": {
                            "$mergeObjects": [
                                "$$lo",
                                {
                                    "topics": {
                                        "$map": {
                                            "input": {
                                                "$filter": {
                                                    "input": {
                                                        "$let": {
                                                            "vars": {
                                                                "lo_id": "$$lo._id"
                                                            },
                                                            "in": {
                                                                "$arrayElemAt": [
                                                                    {
                                                                        "$lookup": {
                                                                            "from": "topics",
                                                                            "localField": "$$lo_id",
                                                                            "foreignField": "learning_outcome_id",
                                                                            "as": "topics"
                                                                        }
                                                                    },
                                                                    "topics"
                                                                ]
                                                            }
                                                        }
                                                    },
                                                    "as": "topic",
                                                    "cond": { "$ne": ["$$topic", null] }
                                                }
                                            },
                                            "as": "topic",
                                            "in": {
                                                "$mergeObjects": [
                                                    "$$topic",
                                                    {
                                                        "learning_materials": {
                                                            "$filter": {
                                                                "input": {
                                                                    "$let": {
                                                                        "vars": {
                                                                            "topic_id": "$$topic._id"
                                                                        },
                                                                        "in": {
                                                                            "$arrayElemAt": [
                                                                                {
                                                                                    "$lookup": {
                                                                                        "from": "subject_learning_materials",
                                                                                        "localField": "$$topic_id",
                                                                                        "foreignField": "reference_id",
                                                                                        "as": "learning_materials"
                                                                                    }
                                                                                },
                                                                                "learning_materials"
                                                                            ]
                                                                        }
                                                                    }
                                                                },
                                                                "as": "lm",
                                                                "cond": {
                                                                    "$and": [
                                                                        { "$eq": ["$$lm.role", "SubjectTopic"] },
                                                                        { "$ne": ["$$lm", null] }
                                                                    ]
                                                                }
                                                            }
                                                        }
                                                    }
                                                ]
                                            }
                                        }
                                    }
                                }
                            ]
                        }
                    }
                }
            }
        },
    ]
}
