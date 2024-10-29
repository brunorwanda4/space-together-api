use std::str::FromStr;

use mongodb::{
    bson::{doc, oid::ObjectId, Bson, DateTime},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};

use crate::{
    error::school::staff_error::{StaffError, StaffResult},
    models::school::staff_model::{StaffModel, StaffModelNew},
};

#[derive(Debug, Clone)]
pub struct StaffActionDb {
    pub staff: Collection<StaffModel>,
}

impl StaffActionDb {
    pub async fn create_staff(&self, staff: StaffModelNew) -> StaffResult<InsertOneResult> {
        let index = IndexModel::builder()
            .keys(doc! {
                "user_id" : 1,
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        let one_index = self.staff.create_index(index).await;
        if one_index.is_err() {
            return Err(StaffError::StaffIsReadyExit);
        }

        let new = StaffModel::new(staff);
        let create = self.staff.insert_one(&new).await;
        match create {
            Ok(res) => Ok(res),
            Err(err) => Err(StaffError::CanNotCreateStaff {
                error: err.to_string(),
            }),
        }
    }

    pub async fn get_staff_by_id(&self, id: String) -> StaffResult<StaffModel> {
        let obj_id = ObjectId::from_str(&id).expect("Can not convert staff id to a string");
        let get = self.staff.find_one(doc! {"_id" : obj_id}).await;

        match get {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(StaffError::StaffNotFound),
            Err(err) => Err(StaffError::CanNotFindStaffById {
                error: err.to_string(),
            }),
        }
    }
    pub async fn get_staff_user_id(&self, user_id: String) -> StaffResult<StaffModel> {
        let obj_id = ObjectId::from_str(&user_id).expect("Can not convert staff id to a string");
        let get = self.staff.find_one(doc! {"user_id" : obj_id}).await;

        match get {
            Ok(Some(res)) => Ok(res),
            Ok(None) => Err(StaffError::UserIdNotFound),
            Err(err) => Err(StaffError::CanNotFindStaffByUserId {
                error: err.to_string(),
            }),
        }
    }
}
