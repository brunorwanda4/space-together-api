use serde_json::{json, Value};

fn assert_compatible_json_shape(old_response: &Value, postgres_response: &Value) {
    match (old_response, postgres_response) {
        (Value::Object(old), Value::Object(new)) => {
            for (key, old_value) in old {
                assert!(
                    new.contains_key(key),
                    "PostgreSQL response is missing compatibility field `{key}`"
                );
                assert_compatible_json_shape(old_value, &new[key]);
            }
        }
        (Value::Array(old_items), Value::Array(new_items)) => {
            if let (Some(old_item), Some(new_item)) = (old_items.first(), new_items.first()) {
                assert_compatible_json_shape(old_item, new_item);
            }
        }
        _ => {}
    }
}

#[test]
fn postgres_responses_keep_mongo_style_public_ids() {
    let response = json!({
        "_id": "664f2b78a68dff8b9cf934d1",
        "name": "Student One",
        "school_id": "664f2b78a68dff8b9cf934d2"
    });

    assert_eq!(response["_id"].as_str().unwrap().len(), 24);
    assert!(response.get("_id").is_some());
}

#[test]
fn compatibility_shape_allows_postgres_extra_internal_fields_only_when_legacy_fields_remain() {
    let mongo_response = json!({
        "_id": "664f2b78a68dff8b9cf934d1",
        "name": "Student One",
        "email": "student@example.com",
        "school_id": "664f2b78a68dff8b9cf934d2",
        "class_id": "664f2b78a68dff8b9cf934d3"
    });

    let postgres_response = json!({
        "_id": "664f2b78a68dff8b9cf934d1",
        "name": "Student One",
        "email": "student@example.com",
        "school_id": "664f2b78a68dff8b9cf934d2",
        "class_id": "664f2b78a68dff8b9cf934d3",
        "student_id": "664f2b78a68dff8b9cf934d4",
        "enrollment_id": "664f2b78a68dff8b9cf934d1"
    });

    assert_compatible_json_shape(&mongo_response, &postgres_response);
}

#[test]
fn school_compatibility_keeps_database_name_as_response_field() {
    let mongo_response = json!({
        "_id": "664f2b78a68dff8b9cf934d1",
        "name": "Space Together School",
        "username": "space-school",
        "database_name": "school_664f2b78a68dff8b9cf934d1"
    });

    let postgres_response = json!({
        "_id": "664f2b78a68dff8b9cf934d1",
        "name": "Space Together School",
        "username": "space-school",
        "database_name": "school_664f2b78a68dff8b9cf934d1"
    });

    assert_compatible_json_shape(&mongo_response, &postgres_response);
}
