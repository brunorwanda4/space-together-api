use mongodb::bson::{doc, Document};

pub fn main_class_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        // ======================================================
        // MATCH
        // ======================================================
        doc! {
            "$match": match_stage
        },
        // ======================================================
        // NORMALIZE OBJECT IDS
        // ======================================================
        doc! {
            "$addFields": {
                "trade_id": {
                    "$cond": [
                        { "$eq": [{ "$type": "$trade_id" }, "string"] },
                        { "$toObjectId": "$trade_id" },
                        "$trade_id"
                    ]
                }
            }
        },
        // ======================================================
        // TRADE
        // ======================================================
        doc! {
            "$lookup": {
                "from": "trades",
                "localField": "trade_id",
                "foreignField": "_id",
                "as": "trade"
            }
        },
        doc! {
            "$unwind": {
                "path": "$trade",
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
