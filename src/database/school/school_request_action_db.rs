use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};

use crate::{
    errors::{MyError, Result},
    models::school::school_request_model::{SchoolRequestModel, SchoolRequestModelNew},
};

#[derive(Debug, Clone)]
pub struct SchoolRequestActionDb {
    pub school_request: Collection<SchoolRequestModel>,
}

impl SchoolRequestActionDb {
    pub async fn create_school_request(
        &self,
        request: SchoolRequestModelNew,
    ) -> Result<InsertOneResult> {
        // check if the request is already in the database
        let index = IndexModel::builder()
            .keys(doc! {"email" : 1, "username" : 1})
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let one_index = self.school_request.create_index(index).await;
        if one_index.is_err() {
            println!("Error creating school request index: {:?}", one_index);
            return Err(MyError::SchoolRequestCanNotCreate);
        }

        let new_req = SchoolRequestModel::new(request);

        let create_request = self
            .school_request
            .insert_one(&new_req)
            .await
            .map_err(|_| MyError::SchoolRequestCanNotCreate);

        match create_request {
            Ok(request) => Ok(request),
            Err(e) => {
                println!("Error inserting school request: {:?}", e);
                Err(MyError::SchoolRequestCanNotCreate)
            }
        }
    }
    pub async fn get_school_request_by_id(&self, id: &str) -> Result<SchoolRequestModel> {
        let req_id = ObjectId::parse_str(id).map_err(|_| MyError::InvalidId)?;

        let req = self
            .school_request
            .find_one(doc! {"_id": req_id})
            .await
            .map_err(|_| MyError::CanNotFIndSchoolRequest);

        match req {
            Ok(Some(request)) => Ok(request),
            Ok(None) => Err(MyError::CanNotFIndSchoolRequest),
            Err(e) => {
                println!("Error getting school request: {:?}", e);
                Err(MyError::CanNotFIndSchoolRequest)
            }
        }
    }

    pub async fn get_school_request_by_email(&self, email: &str) -> Result<SchoolRequestModel> {
        let req = self
            .school_request
            .find_one(doc! {"email": email})
            .await
            .map_err(|_| MyError::CanNotFIndSchoolRequest);

        match req {
            Ok(Some(request)) => Ok(request),
            Ok(None) => Err(MyError::CanNotFIndSchoolRequest),
            Err(e) => {
                println!("Error getting school request: {:?}", e);
                Err(MyError::CanNotFIndSchoolRequest)
            }
        }
    }

    pub async fn get_school_request_by_username(
        &self,
        username: &str,
    ) -> Result<SchoolRequestModel> {
        let req = self
            .school_request
            .find_one(doc! {"username": username})
            .await
            .map_err(|_| MyError::CanNotFIndSchoolRequest);

        match req {
            Ok(Some(request)) => Ok(request),
            Ok(None) => Err(MyError::CanNotFIndSchoolRequest),
            Err(e) => {
                println!("Error getting school request: {:?}", e);
                Err(MyError::CanNotFIndSchoolRequest)
            }
        }
    }
}
