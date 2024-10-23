use mongodb::{
    bson::{doc, oid::ObjectId, Bson, DateTime},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};

use crate::{
    error::school::staff_error::StaffResult,
    models::school::staff_model::{StaffModel, StaffModelNew},
};

pub struct StaffActionDb {
    staff: Collection<StaffModel>,
}

impl StaffActionDb {
    pub async fn create_staff(&self, staff: StaffModelNew) -> StaffResult<InsertOneResult> {
        // let create = self.staff.insert_one()
        todo!()
    }
}
