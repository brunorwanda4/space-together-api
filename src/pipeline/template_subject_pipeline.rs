use mongodb::bson::{doc, Document};

pub fn template_subject_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        // Convert created_by string → ObjectId
        doc! {
            "$addFields": {
                "created_by": {
                    "$cond": [
                        { "$and": [
                            { "$ne": ["$created_by", null] },
                            { "$eq": [ { "$type": "$created_by"}, "string" ] }
                        ]},
                        { "$toObjectId": "$created_by" },
                        "$created_by"
                    ]
                }
            }
        },
        // Convert prerequisites array of strings → array of ObjectId
        doc! {
            "$addFields": {
                "prerequisites": {
                    "$map": {
                        "input": "$prerequisites",
                        "as": "id",
                        "in": {
                            "$cond": [
                                { "$eq": [ { "$type": "$$id" }, "string" ] },
                                { "$toObjectId": "$$id" },
                                "$$id"
                            ]
                        }
                    }
                }
            }
        },
        // Join creator
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "created_by",
                "foreignField": "_id",
                "as": "creator_user"
            }
        },
        doc! { "$unwind": { "path": "$creator_user", "preserveNullAndEmptyArrays": true } },
        // Join prerequisites
        doc! {
            "$lookup": {
                "from": "main_classes",
                "localField": "prerequisites",
                "foreignField": "_id",
                "as": "prerequisite_classes"
            }
        },
        doc! { "$sort": { "updated_at": -1 } },
    ]
}
