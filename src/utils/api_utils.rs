use crate::{models::api_request_model::RequestQuery, utils::object_id::parse_object_id_value};
use actix_web::HttpResponse;
use mongodb::bson::{doc, Bson, Document};

pub fn build_extra_match(query: &RequestQuery) -> Result<Option<Document>, HttpResponse> {
    let mut match_doc = Document::new();

    // Handle by_ids parameter for querying multiple IDs
    if !query.by_ids.is_empty() {
        let object_ids: Vec<mongodb::bson::oid::ObjectId> = query
            .by_ids
            .iter()
            .filter_map(|id| parse_object_id_value(id).ok())
            .collect();

        if object_ids.is_empty() {
            return Err(HttpResponse::BadRequest().json(serde_json::json!({
                "message": "No valid IDs provided in by_ids parameter"
            })));
        }

        match_doc.insert("_id", doc! { "$in": object_ids });
    }

    // Handle field/value pairs
    if !query.field.is_empty() {
        if query.field.len() != query.value.len() {
            return Err(HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Number of fields must match number of values"
            })));
        }

        let mut conditions = Vec::new();

        for (field, value) in query.field.iter().zip(query.value.iter()) {
            let mut bson_value: Bson = value.clone().into();

            // Handle ObjectId conversion
            if field.ends_with("_id") || field.ends_with("_ids") {
                match parse_object_id_value(value) {
                    Ok(object_id) => bson_value = object_id.into(),
                    Err(e) => return Err(HttpResponse::BadRequest().json(e)),
                }
            }

            // Push each condition as its own document: { "field_name": "value" }
            conditions.push(doc! { field: bson_value });
        }

        if !conditions.is_empty() {
            if match_doc.is_empty() {
                match_doc = doc! { "$and": conditions };
            } else {
                // Merge with existing conditions
                let mut all_conditions = vec![match_doc.clone()];
                all_conditions.extend(conditions);
                match_doc = doc! { "$and": all_conditions };
            }
        }
    }

    if match_doc.is_empty() {
        Ok(None)
    } else {
        Ok(Some(match_doc))
    }
}
