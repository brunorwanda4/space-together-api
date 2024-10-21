use std::str::FromStr;

use mongodb::{
    bson::{doc, oid::ObjectId},
    results::InsertOneResult,
    Collection,
};

use crate::{
    error::class::reason_error::{ReasonErr, ReasonResult},
    models::class::reasons_model::{ReasonModel, ReasonModelNew},
};

#[derive(Debug, Clone)]
pub struct ReasonActionDb {
    pub reason: Collection<ReasonModel>,
}

impl ReasonActionDb {
    pub async fn create_reason(&self, reason: ReasonModelNew) -> ReasonResult<InsertOneResult> {
        let new = ReasonModel::new(reason);
        let create = self.reason.insert_one(new).await;
        match create {
            Ok(result) => Ok(result),
            Err(e) => Err(ReasonErr::CanNotCreateReason {
                error: e.to_string(),
            }),
        }
    }
    pub async fn get_reason_by_id(&self, id: String) -> ReasonResult<ReasonModel> {
        let id_obj = ObjectId::from_str(&id).unwrap();
        let get = self.reason.find_one(doc! {"_id": id_obj}).await;
        match get {
            Ok(Some(result)) => Ok(result),
            Ok(None) => Err(ReasonErr::NotFoundReason),
            Err(err) => Err(ReasonErr::CanNotGetReason {
                error: err.to_string(),
            }),
        }
    }
}
