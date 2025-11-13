use mongodb::bson::{self, oid::ObjectId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::models::id_model::IdType;

// ----------------------------
// Option<ObjectId>
// ----------------------------
pub fn serialize<S>(oid: &Option<ObjectId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match oid {
        Some(oid) if serializer.is_human_readable() => serializer.serialize_str(&oid.to_hex()),
        Some(oid) => oid.serialize(serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ObjectId>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;

    match v {
        Value::String(s) => Ok(Some(
            ObjectId::parse_str(&s).map_err(serde::de::Error::custom)?,
        )),
        Value::Object(mut map) => {
            if let Some(Value::String(s)) = map.remove("$oid") {
                Ok(Some(
                    ObjectId::parse_str(&s).map_err(serde::de::Error::custom)?,
                ))
            } else {
                Ok(None)
            }
        }
        Value::Null => Ok(None),
        other => bson::from_bson(bson::to_bson(&other).unwrap())
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

// ----------------------------
// Vec<ObjectId>
// ----------------------------
pub fn serialize_vec<S>(oids: &Vec<ObjectId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if serializer.is_human_readable() {
        let hex_vec: Vec<String> = oids.iter().map(|oid| oid.to_hex()).collect();
        hex_vec.serialize(serializer)
    } else {
        oids.serialize(serializer)
    }
}

pub fn deserialize_vec<'de, D>(deserializer: D) -> Result<Vec<ObjectId>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;

    match v {
        Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                match item {
                    Value::String(s) => {
                        result.push(ObjectId::parse_str(&s).map_err(serde::de::Error::custom)?)
                    }
                    Value::Object(mut map) => {
                        if let Some(Value::String(s)) = map.remove("$oid") {
                            result.push(ObjectId::parse_str(&s).map_err(serde::de::Error::custom)?);
                        }
                    }
                    _ => {}
                }
            }
            Ok(result)
        }
        Value::Null => Ok(vec![]),
        other => bson::from_bson(bson::to_bson(&other).unwrap()).map_err(serde::de::Error::custom),
    }
}

// ----------------------------
// Option<Vec<ObjectId>>
// ----------------------------
pub fn serialize_opt_vec<S>(oids: &Option<Vec<ObjectId>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match oids {
        Some(vec) => serialize_vec(vec, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_opt_vec<'de, D>(deserializer: D) -> Result<Option<Vec<ObjectId>>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;

    match v {
        Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                match item {
                    Value::String(s) => {
                        result.push(ObjectId::parse_str(&s).map_err(serde::de::Error::custom)?)
                    }
                    Value::Object(mut map) => {
                        if let Some(Value::String(s)) = map.remove("$oid") {
                            result.push(ObjectId::parse_str(&s).map_err(serde::de::Error::custom)?);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Some(result))
        }
        Value::Null => Ok(None),
        other => {
            let parsed: Vec<ObjectId> = bson::from_bson(bson::to_bson(&other).unwrap())
                .map_err(serde::de::Error::custom)?;
            Ok(Some(parsed))
        }
    }
}

// ----------------------------
// ObjectId (non-option)
// ----------------------------
pub fn serialize_oid<S>(oid: &ObjectId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if serializer.is_human_readable() {
        serializer.serialize_str(&oid.to_hex())
    } else {
        oid.serialize(serializer)
    }
}

pub fn deserialize_oid<'de, D>(deserializer: D) -> Result<ObjectId, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;

    match v {
        // When data comes as string
        Value::String(s) => ObjectId::parse_str(&s).map_err(serde::de::Error::custom),

        // When data comes as { "$oid": "..." }
        Value::Object(mut map) => {
            if let Some(Value::String(s)) = map.remove("$oid") {
                ObjectId::parse_str(&s).map_err(serde::de::Error::custom)
            } else {
                Err(serde::de::Error::custom("Missing $oid field"))
            }
        }

        // Handle BSON format
        other => bson::from_bson(bson::to_bson(&other).unwrap()).map_err(serde::de::Error::custom),
    }
}

// ----------------------------
// Vec<ObjectId> (non-option)
// ----------------------------
// pub fn serialize_vec_oid<S>(oids: &Vec<ObjectId>, serializer: S) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     if serializer.is_human_readable() {
//         // Convert each ObjectId to hex string
//         let hex_vec: Vec<String> = oids.iter().map(|oid| oid.to_hex()).collect();
//         hex_vec.serialize(serializer)
//     } else {
//         oids.serialize(serializer)
//     }
// }

// pub fn deserialize_vec_oid<'de, D>(deserializer: D) -> Result<Vec<ObjectId>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let v = Value::deserialize(deserializer)?;

//     match v {
//         // JSON array of IDs
//         Value::Array(arr) => {
//             let mut result = Vec::new();
//             for item in arr {
//                 match item {
//                     // Plain string: "6567e9d6..."
//                     Value::String(s) => {
//                         result.push(ObjectId::parse_str(&s).map_err(serde::de::Error::custom)?);
//                     }
//                     // BSON-style: { "$oid": "..." }
//                     Value::Object(mut map) => {
//                         if let Some(Value::String(s)) = map.remove("$oid") {
//                             result.push(ObjectId::parse_str(&s).map_err(serde::de::Error::custom)?);
//                         }
//                     }
//                     _ => {
//                         return Err(serde::de::Error::custom("Invalid ObjectId array element"));
//                     }
//                 }
//             }
//             Ok(result)
//         }
//         Value::Null => Ok(vec![]),
//         // Fallback for BSON representation
//         other => bson::from_bson(bson::to_bson(&other).unwrap()).map_err(serde::de::Error::custom),
//     }
// }

pub fn parse_object_id(id: &IdType) -> Result<ObjectId, String> {
    match id {
        IdType::String(id_str) => {
            ObjectId::parse_str(id_str).map_err(|e| format!("Invalid object ID: {}", e))
        }
        IdType::ObjectId(oid) => Ok(*oid),
    }
}
