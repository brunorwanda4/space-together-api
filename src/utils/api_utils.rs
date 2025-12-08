use actix_web::HttpResponse;
use mongodb::bson::{doc, Bson, Document};

use crate::utils::object_id::parse_object_id_value;

pub fn build_extra_match(
    field: &Option<String>,
    value: &Option<String>,
) -> Result<Option<Document>, HttpResponse> {
    if let (Some(field), Some(value)) = (field, value) {
        let mut bson_value: Bson = value.clone().into();

        // If ends with "_id" convert to ObjectId
        if field.ends_with("_id") {
            match parse_object_id_value(value) {
                Ok(object_id) => bson_value = object_id.into(),
                Err(e) => return Err(HttpResponse::BadRequest().json(e)),
            }
        }

        let extra = doc! { field.clone(): bson_value };
        Ok(Some(extra))
    } else {
        Ok(None)
    }
}
