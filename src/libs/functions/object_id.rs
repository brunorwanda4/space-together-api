use std::str::FromStr;

use mongodb::{bson::oid::ObjectId, results::InsertOneResult};

use crate::models::request_error_model::ReqErrModel;

// pub fn change_insertoneresult_into_string(id: InsertOneResult) -> String {
//     id.inserted_id
//         .as_object_id()
//         .map(|oid| oid.to_hex())
//         .unwrap()
// }make function in rust which get value which is unknown can be array string object or  other this but it check if the value is objectID (mongodb id) it change tah value into string,

pub fn change_insertoneresult_into_object_id(id: InsertOneResult) -> ObjectId {
    id.inserted_id.as_object_id().unwrap()
}
// pub fn change_object_id_into_string(id: ObjectId) -> String {
//     id.to_string()
// }

pub fn change_string_into_object_id(id: String) -> core::result::Result<ObjectId, ReqErrModel> {
    match ObjectId::from_str(&id) {
        Err(_) => Err(ReqErrModel::id(id)),
        Ok(res) => Ok(res),
    }
}
