use mongodb::bson::{doc, Document};

pub fn class_with_others_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        doc! {
            "$lookup": {
                "from": "schools",
                "localField": "school_id",
                "foreignField": "_id",
                "as": "school"
            }
        },
        doc! { "$unwind": { "path": "$school", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "creator_id",
                "foreignField": "_id",
                "as": "creator"
            }
        },
        doc! { "$unwind": { "path": "$creator", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$lookup": {
                "from": "teachers",
                "localField": "class_teacher_id",
                "foreignField": "_id",
                "as": "class_teacher"
            }
        },
        doc! { "$unwind": { "path": "$class_teacher", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$lookup": {
                "from": "main_classes",
                "localField": "main_class_id",
                "foreignField": "_id",
                "as": "main_class"
            }
        },
        doc! { "$unwind": { "path": "$main_class", "preserveNullAndEmptyArrays": true } },
    ]
}
