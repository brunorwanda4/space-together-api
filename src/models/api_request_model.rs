use serde::{Deserialize, Deserializer};

#[derive(Debug)] // Removed Deserialize derive, we will do it manually
pub struct RequestQuery {
    pub filter: Option<String>,
    pub limit: Option<i64>,
    pub skip: Option<i64>,
    pub field: Vec<String>,
    pub value: Vec<String>,
    pub class_id: Option<String>,
    pub education_year_id: Option<String>,
    pub school_id: Option<String>,
    pub term_id: Option<String>,
    pub exam_id: Option<String>,
    pub gpa_threshold: Option<f64>,
    pub by_ids: Vec<String>,
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
            class_id: None,
            education_year_id: None,
            school_id: None,
            term_id: None,
            exam_id: None,
            gpa_threshold: None,
            by_ids: Vec::new(),
        };

        // 2. Manually assign the values from the list
        for (key, val) in pairs {
            match key.as_str() {
                "filter" => query.filter = Some(val),
                "limit" => query.limit = val.parse().ok(),
                "skip" => query.skip = val.parse().ok(),
                "field" => query.field.push(val),
                "value" => query.value.push(val),
                "class_id" => query.class_id = Some(val),
                "education_year_id" => query.education_year_id = Some(val),
                "school_id" => query.school_id = Some(val),
                "term_id" => query.term_id = Some(val),
                "exam_id" => query.exam_id = Some(val),
                "gpa_threshold" => query.gpa_threshold = val.parse().ok(),
                "by_ids" => query.by_ids.push(val),
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
