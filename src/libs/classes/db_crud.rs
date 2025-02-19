use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime, Document},
    Collection,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    libs::functions::object_id::change_insertoneresult_into_object_id,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct GetManyByField {
    pub field: String,
    pub value: ObjectId,
}

#[derive(Debug)]
pub struct MongoCrud<T>
where
    T: Send + Sync,
{
    pub(crate) collection: Collection<T>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountModel {
    pub _id: Option<ObjectId>,
    pub created_at: Option<DateTime>,
}

impl<T> MongoCrud<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Unpin + Send + Sync + 'static,
{
    pub async fn create(&self, document: T, collection: Option<String>) -> DbClassResult<ObjectId> {
        let insert_result = self.collection.insert_one(document).await;
        match insert_result {
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                action: "create".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
            Ok(id) => Ok(change_insertoneresult_into_object_id(id)),
        }
    }

    pub async fn get_one_by_id(
        &self,
        id: ObjectId,
        collection: Option<String>,
    ) -> DbClassResult<T> {
        let filter = doc! { "_id": id };
        let item = self.collection.find_one(filter).await;

        match item {
            Ok(Some(i)) => Ok(i),
            Ok(None) => Err(DbClassError::CanNotDoAction {
                error: "Item not found".to_string(),
                action: "get one".to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                how_fix_it: "Change Id".to_string(),
            }),
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                action: "get one".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
        }
    }

    pub async fn get_one_by_field(
        &self,
        field: GetManyByField,
        collection: Option<String>,
    ) -> DbClassResult<T> {
        let doc = doc! {field.field: field.value};
        let item = self.collection.find_one(doc).await;

        match item {
            Ok(Some(i)) => Ok(i),
            Ok(None) => Err(DbClassError::CanNotDoAction {
                error: "Item not found".to_string(),
                action: "get one".to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                how_fix_it: "Change Id".to_string(),
            }),
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                action: "get one".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
        }
    }

    pub async fn create_new(
        &self,
        mut document: T,
        collection: Option<String>,
    ) -> DbClassResult<T> {
        let created_at = DateTime::now();
        if let Some(account) =
            (&mut document as &mut dyn std::any::Any).downcast_mut::<AccountModel>()
        {
            account.created_at = Some(created_at);
        }

        let insert_result = self.collection.insert_one(&document).await;
        match insert_result {
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                action: "create".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
            Ok(inserted_id) => {
                self.get_one_by_id(
                    change_insertoneresult_into_object_id(inserted_id),
                    collection.clone(),
                )
                .await
            }
        }
    }

    pub async fn get_many(
        &self,
        field: Option<GetManyByField>,
        collection: Option<String>,
    ) -> DbClassResult<Vec<T>> {
        let mut filter = doc! {};

        if let Some(i) = field {
            filter = doc! {i.field: i.value};
        }

        let cursor_result = self.collection.find(filter).await;

        match cursor_result {
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                action: "get many".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
            Ok(r) => {
                let items = r.try_collect().await;
                match items {
                    Ok(data) => Ok(data),
                    Err(err) => Err(DbClassError::CanNotDoAction {
                        error: err.to_string(),
                        collection: collection.unwrap_or_else(|| "unknown".to_string()),
                        action: "convert many into array".to_string(),
                        how_fix_it: "try again later".to_string(),
                    }),
                }
            }
        }
    }

    pub async fn update(
        &self,
        id: ObjectId,
        updated_doc: Document,
        collection: Option<String>,
    ) -> DbClassResult<T> {
        let filter = doc! { "_id": id };
        let update_result = self
            .collection
            .find_one_and_update(filter, doc! {"$set" : updated_doc})
            .await;
        match update_result {
            Ok(Some(i)) => Ok(i),
            Ok(None) => Err(DbClassError::CanNotDoAction {
                error: "Item not found".to_string(),
                action: "update one".to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                how_fix_it: "Change Id".to_string(),
            }),
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                action: "update".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
        }
    }

    pub async fn delete(&self, id: ObjectId, collection: Option<String>) -> DbClassResult<T> {
        let filter = doc! { "_id": id };
        let delete_result = self.collection.find_one_and_delete(filter).await;

        match delete_result {
            Ok(Some(i)) => Ok(i),
            Ok(None) => Err(DbClassError::CanNotDoAction {
                error: "Item not found".to_string(),
                action: "delete one".to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                how_fix_it: "Change Id".to_string(),
            }),
            Err(e) => Err(DbClassError::CanNotDoAction {
                error: e.to_string(),
                collection: collection.unwrap_or_else(|| "unknown".to_string()),
                action: "delete one".to_string(),
                how_fix_it: "try again later".to_string(),
            }),
        }
    }
}
