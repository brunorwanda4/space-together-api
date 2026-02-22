use mongodb::bson::{doc, Document};

pub fn learning_material_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        // ======================================================
        // MATCH
        // ======================================================
        doc! {
            "$match": match_stage
        },
        // ======================================================
        // FILTER OUT SOFT DELETED
        // ======================================================
        doc! {
            "$match": {
                "deleted_at": { "$eq": null }
            }
        },
        // ======================================================
        // NORMALIZE OBJECT IDS (string -> ObjectId safety)
        // ======================================================
        doc! {
            "$addFields": {
                "uploaded_by": {
                    "$cond": [
                        { "$eq": [{ "$type": "$uploaded_by" }, "string"] },
                        { "$toObjectId": "$uploaded_by" },
                        "$uploaded_by"
                    ]
                },
                "school_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$school_id" }, "string"] },
                        { "$toObjectId": "$school_id" },
                        "$school_id"
                    ]
                },
                "class_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$class_id" }, "string"] },
                        { "$toObjectId": "$class_id" },
                        "$class_id"
                    ]
                },
                "subject_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$subject_id" }, "string"] },
                        { "$toObjectId": "$subject_id" },
                        "$subject_id"
                    ]
                }
            }
        },
        // ======================================================
        // UPLOADER (user who uploaded)
        // ======================================================
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "uploaded_by",
                "foreignField": "_id",
                "as": "uploader"
            }
        },
        doc! {
            "$unwind": {
                "path": "$uploader",
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
        // CLASS
        // ======================================================
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
        // ======================================================
        // SUBJECT
        // ======================================================
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
        // ======================================================
        // SORT
        // ======================================================
        doc! {
            "$sort": { "created_at": -1 }
        },
    ]
}
