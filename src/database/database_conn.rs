use std::env;

use mongodb::{Client, Collection};

use super::{
    countries_action_db::CountyActionDb,
    school::{school_request_action_db::SchoolRequestActionDb, team_action_db::TeamActionDb},
    user_action_db::UserActionDb,
};
use crate::{
    errors::Result,
    models::{
        country_model::CountryModel,
        images_models::ProfileImagesModel,
        school::{school_request_model::SchoolRequestModel, team_model::TeamModel},
        user_model::UserModel,
    },
};

#[derive(Debug, Clone)]
pub struct DBConn {
    pub country_action_db: CountyActionDb,
    pub user_action_db: UserActionDb,
    pub school_request_db: SchoolRequestActionDb,
    pub team_db: TeamActionDb,
}

impl DBConn {
    pub async fn init() -> Result<Self> {
        let uri: String = match env::var("MONGODB_URI") {
            Ok(val) => val.to_string(),
            Err(_) => "mongodb://localhost:27017/".to_string(),
        };
        let client = Client::with_uri_str(&uri)
            .await
            .expect("Can not connect to Database");

        // connect to databases
        let space_together_db = client.database("space_together");
        let space_together_image_db = client.database("space_together_images");

        // Corrections in the database of space-together
        let count_collection: Collection<CountryModel> = space_together_db.collection("countries");
        let user_collection: Collection<UserModel> = space_together_db.collection("users");
        let school_request: Collection<SchoolRequestModel> =
            space_together_db.collection("school_requests");
        let team: Collection<TeamModel> = space_together_db.collection("teams");

        // collections in the database of space-together-images
        let avatar_collection: Collection<ProfileImagesModel> =
            space_together_image_db.collection("avatars");

        // initialize the database actions
        let country_action_db = CountyActionDb {
            country: count_collection,
        };
        let user_action_db = UserActionDb {
            user: user_collection,
            avatar: avatar_collection,
        };
        let team_db = TeamActionDb { team };
        let school_request_db = SchoolRequestActionDb { school_request };

        println!("Database connected successfully ✅");

        Ok(Self {
            country_action_db,
            user_action_db,
            school_request_db,
            team_db,
        })
    }
}
