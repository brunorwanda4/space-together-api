use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug , Clone ,Serialize , Deserialize)]
pub struct ProfileImageModel {
    pub src : String,
    pub created_at: Option<DateTime>,
}

#[derive(Debug , Serialize , Deserialize , Clone)]
pub struct ProfileImagesModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id : Option<ObjectId>,
    pub user_id : Option<ObjectId> ,
    pub images : Option<Vec<ProfileImageModel>>,
}



impl ProfileImagesModel {
    pub fn new(src : String , user_id : Option<String>) -> Self {
        let now = DateTime::now().into();
        
       let image = ProfileImageModel {
        src ,
        created_at : Some(DateTime::from_system_time(now))
       };
       
       ProfileImagesModel {
        id : None,
        user_id : Some(ObjectId::parse_str(user_id.expect("user id not found to create new profile image")).expect("Invalid user on ProfileImageModel")),
        images : Some(vec![image])
       }
    }

}
