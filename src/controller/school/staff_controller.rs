use std::sync::Arc;

use crate::{
    error::school::staff_error::{StaffError, StaffResult},
    models::school::staff_model::{StaffModelGet, StaffModelNew, StaffSchoolModelNew},
    AppState,
};

pub async fn create_staff_controller(
    staff: StaffModelNew,
    query: Arc<AppState>,
) -> StaffResult<StaffModelGet> {
    let find_user = query
        .db
        .staff_db
        .get_staff_user_id(staff.user_id.clone())
        .await;
    if find_user.is_ok() {
        return Err(StaffError::UserIdIsReadyExit);
    }
    let create = query.db.staff_db.create_staff(staff).await;
    match create {
        Ok(res) => {
            let id = res
                .inserted_id
                .as_object_id()
                .map(|oid| oid.to_hex())
                .ok_or(StaffError::InvalidId)
                .unwrap();
            let get = query.db.staff_db.get_staff_by_id(id).await;

            match get {
                Ok(result) => {
                    let staff_model = StaffModelGet {
                        id: result
                            .id
                            .map_or("".to_string(), |obj_id| obj_id.to_string()),

                        user_id: result
                            .user_id
                            .map_or("".to_string(), |obj_id| obj_id.to_string()),
                        school: None,
                        school_active: Some(
                            result
                                .school_active
                                .map_or("".to_string(), |id| id.to_string()),
                        ),
                        created_at: result.created_at.to_rfc3339_string(),
                        updated_at: result.updated_at.map(|dt| dt.to_rfc3339_string()),
                    };
                    Ok(staff_model)
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

pub async fn get_staff_by_id_controller(
    query: Arc<AppState>,
    id: String,
) -> StaffResult<StaffModelGet> {
    let get = query.db.staff_db.get_staff_by_id(id).await;
    match get {
        Ok(result) => {
            let staff_model = StaffModelGet {
                id: result
                    .id
                    .map_or("".to_string(), |obj_id| obj_id.to_string()),

                user_id: result
                    .user_id
                    .map_or("".to_string(), |obj_id| obj_id.to_string()),
                school: None,
                school_active: Some(
                    result
                        .school_active
                        .map_or("".to_string(), |id| id.to_string()),
                ),
                created_at: result.created_at.to_rfc3339_string(),
                updated_at: result.updated_at.map(|dt| dt.to_rfc3339_string()),
            };
            Ok(staff_model)
        }
        Err(err) => Err(err),
    }
}
