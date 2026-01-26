use crate::{
    config::state::AppState,
    errors::AppError,
    services::{comment_service, like_service::LikeService},
};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};

pub async fn delete_target_handler(db: &Database, target_id: &ObjectId) -> Result<(), AppError> {
    let like_service = LikeService::new(db);
    let comment_service = comment_service::CommentService::new(db);

    like_service
        .delete_many(doc! {"target_id": target_id})
        .await?;
    comment_service
        .delete_many(doc! {"target_id": target_id})
        .await?;
    Ok(())
}
