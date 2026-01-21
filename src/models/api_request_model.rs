use serde::{Deserialize, Deserializer};

#[derive(Debug)] // Removed Deserialize derive, we will do it manually
pub struct RequestQuery {
    pub filter: Option<String>,
    pub limit: Option<i64>,
    pub skip: Option<i64>,
    pub field: Vec<String>,
    pub value: Vec<String>,
}

impl<'de> Deserialize<'de> for RequestQuery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 1. Tell the parser to give us a raw list of pairs (e.g., [("field", "id"), ("field", "target")])
        // This bypasses the "duplicate field" error because it's not a Map anymore.
        let pairs = Vec::<(String, String)>::deserialize(deserializer)?;

        let mut query = RequestQuery {
            filter: None,
            limit: None,
            skip: None,
            field: Vec::new(),
            value: Vec::new(),
        };

        // 2. Manually assign the values from the list
        for (key, val) in pairs {
            match key.as_str() {
                "filter" => query.filter = Some(val),
                "limit" => query.limit = val.parse().ok(),
                "skip" => query.skip = val.parse().ok(),
                "field" => query.field.push(val),
                "value" => query.value.push(val),
                _ => {} // Ignore unknown fields
            }
        }

        Ok(query)
    }
}

#[derive(Deserialize)]
pub struct GetByIdsBody {
    pub ids: Vec<String>,
}
