use std::sync::Arc;

use crate::{
    error::user::user_error::{UserError, UserResult},
    models::user_model::{UserModelGet, UserModelNew},
    AppState,
};

pub async fn create_user_controller(
    query: Arc<AppState>,
    user: UserModelNew,
) -> UserResult<UserModelGet> {
    if query
        .db
        .user_action_db
        .get_user_by_email(user.email.clone())
        .await
        .is_ok()
    {
        return Err(UserError::UserEmailIsReadyExit);
    }

    let get = query.db.user_action_db.create_user(user).await;

    match get {
        Ok(result) => {
            let id = result
                .inserted_id
                .as_object_id()
                .map(|oid| oid.to_hex())
                .ok_or(ClassError::InvalidId)
                .unwrap();

        }
        Err(err) => Err(err),
    }
}
