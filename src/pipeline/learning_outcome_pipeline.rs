use mongodb::bson::{doc, Document};

pub fn learning_outcome_with_topics_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        // 🔹 Lookup all topics for the learning outcome
        doc! {
            "$lookup": {
                "from": "subject_topics",
                "localField": "_id",
                "foreignField": "learning_outcome_id",
                "as": "topics"
            }
        },
        // 🔹 Lookup subtopics for each topic by matching parent_topic_id
        doc! {
            "$lookup": {
                "from": "subject_topics",
                "let": { "topic_ids": "$topics._id" },
                "pipeline": [
                    { "$match": {
                        "$expr": { "$in": ["$parent_topic_id", "$$topic_ids"] }
                    }},
                    { "$sort": { "order": 1 } }
                ],
                "as": "sub_topics_flat"
            }
        },
        // 🔹 Lookup learning materials linked to all topics (main + sub)
        doc! {
            "$lookup": {
                "from": "subject_learning_materials",
                "let": {
                    "all_topic_ids": { "$concatArrays": ["$topics._id", "$sub_topics_flat._id"] }
                },
                "pipeline": [
                    { "$match": {
                        "$expr": {
                            "$and": [
                                { "$in": ["$reference_id", "$$all_topic_ids"] },
                                { "$eq": ["$role", "SubjectTopic"] }
                            ]
                        }
                    }}
                ],
                "as": "materials_flat"
            }
        },
        // 🔹 Combine main topics, attach materials + subtopics
        doc! {
            "$addFields": {
                "topics": {
                    "$map": {
                        "input": "$topics",
                        "as": "t",
                        "in": {
                            "$mergeObjects": [
                                "$$t",
                                {
                                    "learning_materials": {
                                        "$filter": {
                                            "input": "$materials_flat",
                                            "as": "m",
                                            "cond": { "$eq": ["$$m.reference_id", "$$t._id"] }
                                        }
                                    },
                                    "sub_topics": {
                                        "$map": {
                                            "input": {
                                                "$filter": {
                                                    "input": "$sub_topics_flat",
                                                    "as": "st",
                                                    "cond": { "$eq": ["$$st.parent_topic_id", "$$t._id"] }
                                                }
                                            },
                                            "as": "sub_t",
                                            "in": {
                                                "$mergeObjects": [
                                                    "$$sub_t",
                                                    {
                                                        "learning_materials": {
                                                            "$filter": {
                                                                "input": "$materials_flat",
                                                                "as": "m2",
                                                                "cond": { "$eq": ["$$m2.reference_id", "$$sub_t._id"] }
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
        doc! { "$project": { "materials_flat": 0, "sub_topics_flat": 0 } },
        doc! { "$sort": { "order": 1 } },
    ]
}
