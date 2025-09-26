use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubjectCompetencyBlock {
    pub knowledge: Vec<String>,
    pub skills: Vec<String>,
    pub attitudes: Vec<String>,
}
