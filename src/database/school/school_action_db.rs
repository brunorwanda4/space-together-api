use mongodb::{results::InsertOneResult, Collection};

use crate::{error::school::school_error::SchoolResult, models::school::school_model::SchoolModel};

pub struct SchoolAction {
    pub school_action: Collection<SchoolModel>,
}

impl SchoolAction {
    // Implement methods for CRUD operations on SchoolModel
    pub async fn create_school(&self) -> SchoolResult<InsertOneResult> {
        todo!()
    }
}
