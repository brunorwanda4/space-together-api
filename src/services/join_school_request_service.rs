use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};

use chrono::{DateTime, Utc};

use crate::{
    domain::{
        common_details::Paginated,
        join_school_request::{
            BulkRespondRequest, JoinRequestQuery, JoinSchoolRequest,
            JoinSchoolRequestWithRelations, JoinStatus,
        },
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    repositories::base_repo::BaseRepository,
};

pub struct JoinSchoolRequestService {
    pub collection: Collection<JoinSchoolRequest>,
}

impl JoinSchoolRequestService {
    // ======================================================
    // INIT
    // ======================================================
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<JoinSchoolRequest>("join_school_requests"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("email", false),
            IndexDef::single("school_id", false),
            IndexDef::single("class_id", false),
            IndexDef::single("status", false),
            IndexDef::compound(vec![("email", 1), ("school_id", 1), ("status", 1)], false),
            IndexDef::single("expires_at", false),
            IndexDef::single("created_at", false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    // ======================================================
    // CREATE
    // ======================================================
    pub async fn create(&self, request: JoinSchoolRequest) -> Result<JoinSchoolRequest, AppError> {
        self.ensure_indexes().await?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let doc = mongodb::bson::to_document(&request).map_err(|e| AppError {
            message: format!("Serialize join request failed: {}", e),
        })?;

        repo.create::<JoinSchoolRequest>(doc, None).await
    }

    // ======================================================
    // FIND
    // ======================================================
    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<JoinSchoolRequest>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<JoinSchoolRequest>(doc! { "_id": IdType::to_object_id(id)? }, None)
            .await
    }

    // ======================================================
    // STATUS UPDATES
    // ======================================================
    pub async fn accept_request(
        &self,
        id: &IdType,
        invited_user_id: ObjectId,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        self.update_status(
            id,
            JoinStatus::Accepted,
            Some(invited_user_id),
            responded_by,
        )
        .await
    }

    pub async fn reject_request(
        &self,
        id: &IdType,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        self.update_status(id, JoinStatus::Rejected, None, responded_by)
            .await
    }

    pub async fn cancel_request(
        &self,
        id: &IdType,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        self.update_status(id, JoinStatus::Cancelled, None, responded_by)
            .await
    }

    async fn update_status(
        &self,
        id: &IdType,
        status: JoinStatus,
        invited_user_id: Option<ObjectId>,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let mut update = doc! {
            "status": bson::to_bson(&status).map_err(|e| AppError {
                message: format!("Failed to serialize status: {}", e),
            })?,
        };

        if let Some(uid) = invited_user_id {
            update.insert("invited_user_id", uid);
        }

        if let Some(by) = responded_by {
            update.insert("responded_by", by);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.update_one_and_fetch::<JoinSchoolRequest>(id, update)
            .await
    }

    // ======================================================
    // EXPIRATION
    // ======================================================
    pub async fn update_expiration(
        &self,
        id: &IdType,
        expires_at: DateTime<Utc>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.update_one_and_fetch::<JoinSchoolRequest>(
            id,
            doc! {
                "expires_at": bson::to_bson(&expires_at).unwrap(),
            },
        )
        .await
    }

    pub async fn expire_old_requests(&self) -> Result<u64, AppError> {
        let now = Utc::now();
        let result = self
            .collection
            .update_many(
                doc! {
                    "expires_at": { "$lte": bson::to_bson(&now).unwrap() },
                    "status": bson::to_bson(&JoinStatus::Pending).map_err(|e| AppError {
                        message: format!("Failed to serialize status: {}", e),
                    })?
                },
                doc! {
                    "$set": {
                        "status": bson::to_bson(&JoinStatus::Expired).map_err(|e| AppError {
                            message: format!("Failed to serialize status: {}", e),
                        })?,
                        "updated_at": bson::to_bson(&now).unwrap()
                    }
                },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to expire old join requests: {}", e),
            })?;

        Ok(result.modified_count)
    }

    pub async fn cleanup_expired_requests(&self, older_than_days: i64) -> Result<u64, AppError> {
        let cutoff_date = Utc::now() - chrono::Duration::days(older_than_days);
        let result = self
            .collection
            .delete_many(doc! {
                "status": bson::to_bson(&JoinStatus::Expired).map_err(|e| AppError {
                    message: format!("Failed to serialize status: {}", e),
                })?,
                "updated_at": { "$lte": bson::to_bson(&cutoff_date).unwrap() }
            })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to cleanup expired join requests: {}", e),
            })?;

        Ok(result.deleted_count)
    }

    // ======================================================
    // RELATIONS (PIPELINE)
    // ======================================================
    pub async fn query_with_relations(
        &self,
        query: &JoinRequestQuery,
    ) -> Result<Vec<JoinSchoolRequestWithRelations>, AppError> {
        let mut match_stage = Document::new();

        if let Some(status) = &query.status {
            match_stage.insert("status", status);
        }

        if let Some(school_id) = &query.school_id {
            match_stage.insert(
                "school_id",
                IdType::to_object_id(&IdType::String(school_id.clone()))?,
            );
        }

        let pipeline = vec![
            doc! { "$match": match_stage },
            doc! {
                "$lookup": {
                    "from": "schools",
                    "localField": "school_id",
                    "foreignField": "_id",
                    "as": "school"
                }
            },
            doc! { "$unwind": { "path": "$school", "preserveNullAndEmptyArrays": true } },
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.aggregate::<JoinSchoolRequestWithRelations>(pipeline)
            .await
    }
}
