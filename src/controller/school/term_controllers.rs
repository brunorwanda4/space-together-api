use std::sync::Arc;

use mongodb::bson::Bson;

use crate::{
    error::term_error::{TermError, TermResult},
    models::school::term_model::{TermModelGet, TermModelNew, TermModelUpdate},
    AppState,
};

pub async fn create_term_controller(
    term: TermModelNew,
    school: Option<String>,
    db: Arc<AppState>,
) -> TermResult<TermModelGet> {
    let new = db
        .db
        .term_db
        .create_term(term)
        .await
        .map_err(|_| TermError::CanNotCreateTerm);

    match new {
        Ok(result) => {
            let id = if let Bson::ObjectId(oid) = result.inserted_id {
                Ok(oid.to_hex())
            } else {
                Err(TermError::CanNotCreateTerm)
            };
            let get = db.db.term_db.get_term_by_id(id.unwrap()).await;
            match get {
                Ok(result) => {
                    let get_term_model = TermModelGet {
                        id: result
                            .id
                            .map_or("".to_string(), |obj_id| obj_id.to_string()),
                        name: result.name,
                        school: Some(
                            result
                                .school
                                .map_or("".to_string(), |obj_id| obj_id.to_string()),
                        ),
                        description: result.description,
                        username: result.username,
                        end_on: result.end_on.to_rfc3339_string(),
                        status: result.status,
                        start_on: result.start_on.to_rfc3339_string(),
                        term_type: result.term_type,
                        created_at: result.created_at.to_rfc3339_string(),
                        updated_at: result.updated_at.map(|dt| dt.to_rfc3339_string()),
                    };
                    Ok(get_term_model)
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}
pub async fn get_term_controller(query: Arc<AppState>, id: String) -> TermResult<TermModelGet> {
    let get = query.db.term_db.get_term_by_id(id).await;
    match get {
        Ok(result) => {
            let get_term_model = TermModelGet {
                id: result
                    .id
                    .map_or("".to_string(), |obj_id| obj_id.to_string()),
                name: result.name,
                school: Some(
                    result
                        .school
                        .map_or("".to_string(), |obj_id| obj_id.to_string()),
                ),
                description: result.description,
                username: result.username,
                status: result.status,
                end_on: result.end_on.to_rfc3339_string(),
                start_on: result.start_on.to_rfc3339_string(),
                term_type: result.term_type,
                created_at: result.created_at.to_rfc3339_string(),
                updated_at: result.updated_at.map(|dt| dt.to_rfc3339_string()),
            };
            Ok(get_term_model)
        }
        Err(err) => Err(err),
    }
}

pub async fn update_term_controller(
    query: Arc<AppState>,
    term: TermModelUpdate,
    id: String,
) -> TermResult<TermModelGet> {
    let update = query.db.term_db.update_term_by_id(id, term).await;
    match update {
        Ok(result) => {
            let term_id = result
                .id
                .map_or("".to_string(), |obj_id| obj_id.to_string());
            let get_term = query.db.term_db.get_term_by_id(term_id).await?;

            let get_term_model = TermModelGet {
                id: get_term
                    .id
                    .map_or("".to_string(), |obj_id| obj_id.to_string()),
                name: get_term.name,
                school: Some(
                    get_term
                        .school
                        .map_or("".to_string(), |obj_id| obj_id.to_string()),
                ),
                description: get_term.description,
                username: get_term.username,
                status: get_term.status,
                end_on: get_term.end_on.to_rfc3339_string(),
                start_on: get_term.start_on.to_rfc3339_string(),
                term_type: get_term.term_type,
                created_at: get_term.created_at.to_rfc3339_string(),
                updated_at: get_term.updated_at.map(|dt| dt.to_rfc3339_string()),
            };
            Ok(get_term_model)
        }
        Err(err) => Err(err),
    }
}
