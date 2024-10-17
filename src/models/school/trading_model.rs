use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum TradingType {
    Primary,
    OLevel,
    ELevel,
}

#[derive(Debug, Deserialize)]
pub struct TradingModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub trading_type: TradingType,
    pub schools_id: Option<Vec<ObjectId>>,
    pub reasons: Option<Vec<ObjectId>>,
}
