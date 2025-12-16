use mongodb::bson::{doc, Document};

pub fn main_class_with_others_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        doc! {
            "$lookup": {
                "from": "trades",
                "localField": "trade_id",
                "foreignField": "_id",
                "as": "trade"
            }
        },
        doc! { "$unwind": { "path": "$trade", "preserveNullAndEmptyArrays": true } },
        // Include sector & parent_trade inside Trade
        doc! {
            "$lookup": {
                "from": "sectors",
                "localField": "trade.sector_id",
                "foreignField": "_id",
                "as": "trade.sector"
            }
        },
        doc! { "$unwind": { "path": "$trade.sector", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$lookup": {
                "from": "trades",
                "localField": "trade.trade_id",
                "foreignField": "_id",
                "as": "trade.parent_trade"
            }
        },
        doc! {
            "$addFields": {
                "trade.parent_trade": {
                    "$cond": {
                        "if": { "$gt": [{ "$size": "$trade.parent_trade" }, 0] },
                        "then": { "$arrayElemAt": ["$trade.parent_trade", 0] },
                        "else": null
                    }
                }
            }
        },
        doc! {
            "$lookup": {
                "from": "sectors",
                "localField": "trade.parent_trade.sector_id",
                "foreignField": "_id",
                "as": "trade.parent_trade.sector"
            }
        },
        doc! { "$unwind": { "path": "$trade.parent_trade.sector", "preserveNullAndEmptyArrays": true } },
    ]
}

pub fn main_class_with_trade_pipeline(match_stage: Document) -> Vec<Document> {
    vec![
        doc! { "$match": match_stage },
        doc! {
            "$lookup": {
                "from": "trades",
                "localField": "trade_id",
                "foreignField": "_id",
                "as": "trade"
            }
        },
        doc! { "$unwind": { "path": "$trade", "preserveNullAndEmptyArrays": true } },
    ]
}
