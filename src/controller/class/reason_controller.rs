use std::sync::Arc;

use mongodb::bson::{oid::ObjectId, Bson};

use crate::{
    error::class::reason_error::{ReasonErr, ReasonResult},
    models::class::reasons_model::{ReasonModelGet, ReasonModelNew},
    AppState,
};

pub async fn create_reason_controller(
    query: Arc<AppState>,
    reason: ReasonModelNew,
) -> ReasonResult<ReasonModelGet> {
    let create = query.db.reason_db.create_reason(reason).await;
    match create {
        Ok(res) => {
            let id = if let Bson::ObjectId(oid) = res.inserted_id {
                Ok(oid.to_hex())
            } else {
                Err(ReasonErr::InvalidId)
            };
            let get = query.db.reason_db.get_reason_by_id(id.unwrap()).await;
            match get {
                Ok(result) => {
                    let reason_get = ReasonModelGet {
                        id: result
                            .id
                            .map_or("".to_string(), |obj_id| obj_id.to_string()),
                        name: result.name,
                        code: result.code,
                        reason_content: result.reason_content,
                        hours: result.hours,
                        is_active: result.is_active,
                        is_public: result.is_public,
                        follow_up_required: result.follow_up_required,
                        reason_type: result.reason_type,
                        severity: result.severity,
                        school: result.school.map(|id| id.to_string()),
                        classes: result
                            .classes
                            .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
                        updated_by: result
                            .updated_by
                            .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
                        teachers_id: result
                            .teachers_id
                            .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
                        trading: result
                            .trading
                            .map(|ids| ids.iter().map(|id| id.to_string()).collect()),
                        created_at: result.created_at.to_rfc3339_string(),
                        updated_at: result.updated_at.map(|dt| dt.to_rfc3339_string()),
                    };
                    Ok(reason_get)
                }
                Err(err) => Err(err),
            }
        }
        Err(e) => Err(e),
    }
}
