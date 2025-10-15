use mongodb::bson::{doc, Document};

pub fn teacher_with_relations_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
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
        doc! {
            "$lookup": {
                "from": "classes",
                "localField": "class_ids",
                "foreignField": "_id",
                "as": "classes"
            }
        },
        doc! {
            "$lookup": {
                "from": "subjects",
                "localField": "subject_ids",
                "foreignField": "_id",
                "as": "subjects"
            }
        },
    ]
}
