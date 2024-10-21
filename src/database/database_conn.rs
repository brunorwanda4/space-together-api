use std::env;

use mongodb::{Client, Collection};

use super::{
    class::reasons_action_db::ReasonActionDb,
    countries_action_db::CountyActionDb,
    school::{
        school_request_action_db::SchoolRequestActionDb, term_action_db::TermActionDb,
        trading_action_db::TradingActionDb,
    },
    user_action_db::UserActionDb,
};
use crate::{
    errors::Result,
    models::{
        class::reasons_model::ReasonModel,
        country_model::CountryModel,
        images_models::ProfileImagesModel,
        school::{
            school_request_model::SchoolRequestModel, term_model::TermModel,
            trading_model::TradingModel,
        },
        user_model::UserModel,
    },
};

#[derive(Debug, Clone)]
pub struct DBConn {
    pub country_action_db: CountyActionDb,
    pub user_action_db: UserActionDb,
    pub school_request_db: SchoolRequestActionDb,
    pub term_db: TermActionDb,
    pub trading_db: TradingActionDb,
    pub reason_db: ReasonActionDb,
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

        // collections in the database of space-together-images
        let avatar_collection: Collection<ProfileImagesModel> =
            space_together_image_db.collection("avatars");

        // Corrections in the database of space-together
        let count_collection: Collection<CountryModel> = space_together_db.collection("countries");
        let country_action_db = CountyActionDb {
            country: count_collection,
        };

        let user_collection: Collection<UserModel> = space_together_db.collection("users");
        let user_action_db = UserActionDb {
            user: user_collection,
            avatar: avatar_collection,
        };

        let trading: Collection<TradingModel> = space_together_db.collection("tradings");
        let trading_db = TradingActionDb { trading };

        let term: Collection<TermModel> = space_together_db.collection("terms");
        let term_db = TermActionDb { term };

        let school_request: Collection<SchoolRequestModel> =
            space_together_db.collection("school_requests");
        let school_request_db = SchoolRequestActionDb { school_request };

        let reason: Collection<ReasonModel> = space_together_db.collection("reasons");
        let reason_db = ReasonActionDb { reason };

        println!("Database connected successfully ✅");

        Ok(Self {
            country_action_db,
            user_action_db,
            school_request_db,
            term_db,
            trading_db,
            reason_db,
        })
    }
}
