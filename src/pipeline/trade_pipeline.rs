use mongodb::bson::{doc, Document};

pub fn trade_with_others_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        doc! {
            "$lookup": {
                "from": "sectors",
                "localField": "sector_id",
                "foreignField": "_id",
                "as": "sector"
            }
        },
        doc! { "$unwind": { "path": "$sector", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$lookup": {
                "from": "trades",
                "localField": "trade_id",
                "foreignField": "_id",
                "as": "parent_trade"
            }
        },
        // ✅ FIXED: use $cond to ensure null instead of {}
        doc! {
            "$addFields": {
                "parent_trade": {
                    "$cond": {
                        "if": { "$gt": [{ "$size": "$parent_trade" }, 0] },
                        "then": { "$arrayElemAt": ["$parent_trade", 0] },
                        "else": null
                    }
                }
            }
        },
        doc! {
            "$lookup": {
                "from": "sectors",
                "localField": "parent_trade.sector_id",
                "foreignField": "_id",
                "as": "parent_trade.sector"
            }
        },
        doc! { "$unwind": { "path": "$parent_trade.sector", "preserveNullAndEmptyArrays": true } },
    ]
}
