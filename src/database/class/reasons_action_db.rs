use std::str::FromStr;

use mongodb::{
    bson::{doc, oid::ObjectId, to_bson, DateTime},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};

use crate::{
    error::class::reason_error::{ReasonErr, ReasonResult},
    models::class::reasons_model::{ReasonModel, ReasonModelNew, ReasonModelUpdate},
};

#[derive(Debug, Clone)]
pub struct ReasonActionDb {
    pub reason: Collection<ReasonModel>,
}

impl ReasonActionDb {
    pub async fn create_reason(&self, reason: ReasonModelNew) -> ReasonResult<InsertOneResult> {
        let index = IndexModel::builder()
            .keys(doc! {
                "code" : 1
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        let one_index = self.reason.create_index(index).await;
        if one_index.is_err() {
            return Err(ReasonErr::CanNotCreateReasonIndex);
        }
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
    pub async fn update_reason_by_id(
        &self,
        id: String,
        reason: ReasonModelUpdate,
    ) -> ReasonResult<ReasonModel> {
        let obj_id = ObjectId::from_str(&id).unwrap();
        let mut update_doc = doc! {};
        let mut is_update = false;

        if let Some(name) = reason.name {
            update_doc.insert("name", name);
            is_update = true;
        }
        if let Some(reason_content) = reason.reason_content {
            // Convert ReasonContent to BSON using `to_bson`
            if let Ok(bson_content) = to_bson(&reason_content) {
                update_doc.insert("reason_content", bson_content);
                is_update = true;
            }
        }
        if let Some(code) = reason.code {
            update_doc.insert("code", code);
            is_update = true;
        }
        if let Some(severity) = reason.severity {
            update_doc.insert("severity", severity.to_string());
            is_update = true;
        }
        if let Some(hours) = reason.hours {
            update_doc.insert("hours", hours);
            is_update = true;
        }
        if let Some(is_active) = reason.is_active {
            update_doc.insert("is_active", is_active);
            is_update = true;
        }
        if let Some(is_public) = reason.is_public {
            update_doc.insert("is_public", is_public);
            is_update = true;
        }
        if let Some(follow_up_required) = reason.follow_up_required {
            update_doc.insert("follow_up_required", follow_up_required);
            is_update = true;
        }
        if let Some(reason_type) = reason.reason_type {
            update_doc.insert("reason_type", reason_type.to_string());
            is_update = true;
        }
        if let Some(trading) = reason.trading {
            update_doc.insert("trading", trading);
            is_update = true;
        }
        if let Some(teaches_id) = reason.teachers_id {
            update_doc.insert("teachers_id", teaches_id);
            is_update = true;
        }

        if update_doc.is_empty() {
            return Err(ReasonErr::NoFieldsToUpdate);
        }
        if is_update {
            update_doc.insert("updated_at", DateTime::now());
        }

        let update = self
            .reason
            .find_one_and_update(doc! {"_id": obj_id}, doc! {"$set" : update_doc})
            .await;
        match update {
            Ok(Some(ok)) => Ok(ok),
            Ok(None) => Err(ReasonErr::ReasonNotFound),
            Err(e) => Err(ReasonErr::CanNotUpdateReason {
                error: e.to_string(),
            }),
        }
    }
    pub async fn delete_reason_by_id(&self, id: String) -> ReasonResult<ReasonModel> {
        let id_obj = ObjectId::from_str(&id).unwrap();
        let delete = self.reason.find_one_and_delete(doc! {"_id": id_obj}).await;
        match delete {
            Ok(Some(reason)) => Ok(reason),
            Ok(None) => Err(ReasonErr::ReasonNotFound),
            Err(e) => Err(ReasonErr::CanNotDeleteReason {
                error: e.to_string(),
            }),
        }
    }
}
