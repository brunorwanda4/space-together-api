use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{helpers::object_id_helpers, make_partial};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BackupType {
    #[default]
    #[serde(rename = "AUTOMATED")]
    Automated,
    #[serde(rename = "MANUAL")]
    Manual,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BackupStatus {
    #[default]
    #[serde(rename = "IN_PROGRESS")]
    InProgress,
    #[serde(rename = "COMPLETED")]
    Completed,
    #[serde(rename = "FAILED")]
    Failed,
}

make_partial! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct SchoolBackup {
        #[serde(
            rename = "_id",
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub id: Option<ObjectId>,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub school_id: Option<ObjectId>,

        pub backup_name: String,

        #[serde(default)]
        pub backup_type: BackupType,

        pub file_path: String,

        #[serde(default)]
        pub size_bytes: i64,

        #[serde(default)]
        pub status: BackupStatus,

        #[serde(
            serialize_with = "object_id_helpers::serialize",
            deserialize_with = "object_id_helpers::deserialize",
            skip_serializing_if = "Option::is_none",
            default
        )]
        pub created_by: Option<ObjectId>,

        #[serde(default = "Utc::now")]
        pub created_at: DateTime<Utc>,

        pub completed_at: Option<DateTime<Utc>>,

        pub error_message: Option<String>,
    } => SchoolBackupPartial
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchoolBackupWithRelations {
    #[serde(flatten)]
    pub backup: SchoolBackup,

    pub school: Option<crate::domain::school::School>,
    pub creator: Option<crate::domain::user::User>,
}
