use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum StaffRole {
    Teacher,
    Headmaster,
    AssistantHeadmaster,
    Principal,
    DeputyPrincipal,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum StaffRoleType {
    Role(StaffRole),
    StringRole(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StaffSchoolModel {
    pub school_id: ObjectId,
    pub role: StaffRoleType,
    pub stated_job: DateTime,
    pub name: String,
    pub is_active: bool,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StaffSchoolModelNew {
    pub school_id: String,
    pub role: StaffRoleType,
    pub stated_job: String,
    pub name: String,
    pub is_active: bool,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StaffModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: Option<ObjectId>,
    pub school: Option<Vec<StaffSchoolModel>>,
    pub school_active: Option<String>,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StaffModelNew {
    pub user_id: String,
    pub school: StaffSchoolModelNew,
}

impl StaffModel {
    fn new(staff: StaffModelNew) -> Self {
        todo!()
    }
}
