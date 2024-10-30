use std::str::FromStr;

use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, DateTime};
use mongodb::results::InsertOneResult;
use mongodb::Collection;

use crate::error::term_error::{TermError, TermResult};
use crate::models::school::term_model::{TermModel, TermModelNew, TermModelUpdate};

#[derive(Debug, Clone)]
pub struct TermActionDb {
    pub term: Collection<TermModel>,
}

impl TermActionDb {
    pub async fn create_term(&self, term: TermModelNew) -> TermResult<InsertOneResult> {
        let new = TermModel::new(term);
        let new_term = self.term.insert_one(new).await;

        match new_term {
            Ok(result) => Ok(result),
            Err(_) => Err(TermError::CanNotCreateTerm),
        }
    }
    pub async fn get_term_by_id(&self, id: String) -> TermResult<TermModel> {
        let obj_id = ObjectId::from_str(&id)
            .map_err(|_| TermError::TermInvalidId)
            .expect("Can not invalid id");
        let get = self.term.find_one(doc! {"_id" : obj_id}).await;

        match get {
            Ok(Some(term)) => Ok(term),
            Ok(None) => Err(TermError::TermNotFound),
            Err(err) => Err(TermError::CanNotGetTerm {
                error: err.to_string(),
            }),
        }
    }
    pub async fn update_term_by_id(
        &self,
        id: String,
        term: TermModelUpdate,
    ) -> TermResult<TermModel> {
        let obj_id = ObjectId::from_str(&id)
            .map_err(|_| TermError::TermInvalidId)
            .expect("can not change term id for update");
        let mut update_doc = doc! {};
        let mut is_updated = false;
        if let Some(name) = term.name {
            update_doc.insert("name", name);
            is_updated = true;
        }
        if let Some(description) = term.description {
            update_doc.insert("description", description);
            is_updated = true;
        }
        if let Some(school) = term.school {
            update_doc.insert("school", school);
            is_updated = true;
        }
        if let Some(term_type) = term.term_type {
            update_doc.insert("term_type", term_type.to_string());
            is_updated = true;
        }
        if let Some(start_on) = term.start_on {
            update_doc.insert("start_on", start_on);
            is_updated = true;
        }
        if let Some(end_on) = term.end_on {
            update_doc.insert("end_on", end_on);
            is_updated = true;
        }
        if let Some(status) = term.status {
            update_doc.insert("status", status);
            is_updated = true;
        }

        if update_doc.is_empty() {
            return Err(TermError::NoFieldsToUpdate);
        }
        if is_updated {
            update_doc.insert("updated_at", DateTime::now());
        }
        let update = self
            .term
            .find_one_and_update(doc! {"_id" : obj_id}, doc! {"$set" : update_doc})
            .await;
        match update {
            Ok(Some(ok)) => Ok(ok),
            Ok(None) => Err(TermError::TermNotFound),
            Err(_) => Err(TermError::CanNotUpdateTerm),
        }
    }
}
