use mongodb::bson::{doc, Document};

pub fn subject_with_relations_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        // Lookup class
        doc! {
            "$lookup": {
                "from": "classes",
                "localField": "class_id",
                "foreignField": "_id",
                "as": "class"
            }
        },
        doc! { "$unwind": { "path": "$class", "preserveNullAndEmptyArrays": true } },
        // Lookup class teacher (user)
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "class_teacher_id",
                "foreignField": "_id",
                "as": "class_teacher"
            }
        },
        doc! { "$unwind": { "path": "$class_teacher", "preserveNullAndEmptyArrays": true } },
        // Lookup main subject
        doc! {
            "$lookup": {
                "from": "main_subjects",
                "localField": "main_subject_id",
                "foreignField": "_id",
                "as": "main_subject"
            }
        },
        doc! { "$unwind": { "path": "$main_subject", "preserveNullAndEmptyArrays": true } },
    ]
}

// pub fn subject_with_class_pipeline(match_stage: Document) -> Vec<Document> {
//     vec![
//         doc! { "$match": match_stage },
//         doc! {
//             "$lookup": {
//                 "from": "classes",
//                 "localField": "class_id",
//                 "foreignField": "_id",
//                 "as": "class"
//             }
//         },
//         doc! { "$unwind": { "path": "$class", "preserveNullAndEmptyArrays": true } },
//     ]
// }
