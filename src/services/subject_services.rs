use crate::{
    error::db_class_error::{DbClassError, DbClassResult},
    libs::{
        functions::{crud_fn::to_document, data_type_fn::convert_fields_to_string},
        schemas::subject_schema::SubjectSchema,
    },
    AppState,
};
use mongodb::{
    bson::{doc, Document},
    options::IndexOptions,
    IndexModel,
};
use std::sync::Arc;

pub async fn create_subject_servicer(
    state: Arc<AppState>,
    item: SubjectSchema,
) -> DbClassResult<Document> {
    let index = IndexModel::builder()
        .keys(doc! {
        "code" : 1,
        })
        .options(IndexOptions::builder().unique(true).build())
        .build();

    if (state.db.subject.collection.create_index(index).await).is_err() {
        return Err(DbClassError::OtherError {
            err: "Subject Code is ready exit, try other code".to_string(),
        });
    }

    match state
        .db
        .subjects
        .create_new(item, Some("subject".to_string()))
        .await
    {
        Err(e) => Err(e),
        Ok(doc) => Ok(convert_fields_to_string(to_document(&doc))),
    }
}
