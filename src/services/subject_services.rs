use std::sync::Arc;

use crate::{
    error::db_class_error::DbClassResult, libs::schemas::subject_schema::SubjectSchema, AppState,
};

pub async fn create_subject_servicer(
    state: Arc<AppState>,
    item: SubjectSchema,
) -> DbClassResult<SubjectSchema> {
    state
        .db
        .subjects
        .create_new(item, Some("subject".to_string()))
        .await
}
