use std::sync::Arc;

use crate::{
    error::class::reason_error::{ReasonErr, ReasonResult},
    models::{
        class::reasons_model::{ReasonModelGet, ReasonModelNew, ReasonModelUpdate},
        school::trading_model::TradingModelUpdateReasons,
    },
    AppState,
};

pub async fn create_reason_controller(
    query: Arc<AppState>,
    reason: ReasonModelNew,
) -> ReasonResult<ReasonModelGet> {
    let create = query.db.reason_db.create_reason(reason.clone()).await;
    match create {
        Ok(res) => {
            let id = res
                .inserted_id
                .as_object_id()
                .map(|oid| oid.to_hex())
                .ok_or(ReasonErr::InvalidId)
                .unwrap();

            if reason.trading.is_some() {
                let trading_model_update = TradingModelUpdateReasons {
                    tradings_id: reason.trading,
                    reason_id: id.clone(),
                };
                let updates_trading = query
                    .db
                    .trading_db
                    .update_trading_reasons(trading_model_update)
                    .await;
                if let Err(err) = updates_trading {
                    return Err(ReasonErr::CanMakeReasonBecauseOfTradingError {
                        error: err.to_string(),
                    });
                }
            };
            let get = query.db.reason_db.get_reason_by_id(id).await;
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

pub async fn get_reason_controller(
    query: Arc<AppState>,
    id: String,
) -> ReasonResult<ReasonModelGet> {
    let get = query.db.reason_db.get_reason_by_id(id.clone()).await;
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

pub async fn update_reason_controller(
    query: Arc<AppState>,
    id: String,
    reason: ReasonModelUpdate,
) -> ReasonResult<ReasonModelGet> {
    let trading_model_reason = TradingModelUpdateReasons {
        reason_id: id.clone(),
        tradings_id: reason.trading.clone(),
    };

    let update_trading = query
        .db
        .trading_db
        .update_trading_reasons(trading_model_reason)
        .await;

    if let Err(err) = update_trading {
        return Err(ReasonErr::CanMakeReasonBecauseOfTradingError {
            error: err.to_string(),
        });
    }

    let update = query
        .db
        .reason_db
        .update_reason_by_id(id.clone(), reason)
        .await;
    match update {
        Ok(res) => {
            let re_id = res.id.map_or("".to_string(), |obj_id| obj_id.to_string());
            let get = query.db.reason_db.get_reason_by_id(re_id).await;
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
                Err(e) => Err(e),
            }
        }
        Err(err) => Err(err),
    }
}
pub async fn delete_reason_by_id_controller(
    query: Arc<AppState>,
    id: String,
) -> ReasonResult<ReasonModelGet> {
    let delete = query.db.reason_db.delete_reason_by_id(id.clone()).await;
    match delete {
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
