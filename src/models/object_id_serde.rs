use mongodb::bson::oid::ObjectId;
use serde::{self, Deserialize, Deserializer, Serializer};
use serde_json::Value;

pub fn serialize<S>(oid: &Option<ObjectId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match oid {
        Some(oid) => serializer.serialize_str(&oid.to_hex()),
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
        _ => Ok(None),
    }
}
