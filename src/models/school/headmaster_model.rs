use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

use crate::models::social_media_model::SocialMediaModel;

#[derive(Deserialize, Debug)]
pub struct HeadMasterContact {
    pub email: String,
    pub phone: String,
    pub social_account: Option<Vec<SocialMediaModel>>,
}

#[derive(Debug, Deserialize)]
pub struct HeadmasterModel {
    #[serde(rename = "_id", skip_serializing_if = "Optional::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: Option<ObjectId>,
    pub contacts: HeadMasterContact,
}
