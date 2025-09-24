use mongodb::bson::{self, oid::ObjectId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

pub fn serialize<S>(oid: &Option<ObjectId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match oid {
        // When serializing to JSON → string
        Some(oid) if serializer.is_human_readable() => serializer.serialize_str(&oid.to_hex()),

        // When serializing to BSON (Mongo) → native ObjectId
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
        // API input: plain string "650f3d..."
        Value::String(s) => Ok(Some(
            ObjectId::parse_str(&s).map_err(serde::de::Error::custom)?,
        )),

        // Mongo extended JSON {"$oid": "..."}
        Value::Object(mut map) => {
            if let Some(Value::String(s)) = map.remove("$oid") {
                Ok(Some(
                    ObjectId::parse_str(&s).map_err(serde::de::Error::custom)?,
                ))
            } else {
                Ok(None)
            }
        }

        // null or missing
        Value::Null => Ok(None),

        // If serde gives us directly an ObjectId (from BSON)
        other => {
            // Try ObjectId deserialization fallback
            bson::from_bson(bson::to_bson(&other).unwrap())
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
    }
}
