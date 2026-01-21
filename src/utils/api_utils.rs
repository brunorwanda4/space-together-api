use crate::{models::api_request_model::RequestQuery, utils::object_id::parse_object_id_value};
use actix_web::HttpResponse;
use mongodb::bson::{doc, Bson, Document};

pub fn build_extra_match(query: &RequestQuery) -> Result<Option<Document>, HttpResponse> {
    if query.field.is_empty() {
        return Ok(None);
    }

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
    Ok(Some(doc! { "$and": conditions }))
}
