use crate::{
    domain::{
        join_school_request::{JoinSchoolRequest, JoinSchoolRequestWithRelations, JoinStatus},
        school::School,
        user::User,
    },
    errors::AppError,
    helpers::object_id_helpers::parse_object_id,
    models::id_model::IdType,
    pipeline::join_school_request_pipeline::join_school_request_with_relations_pipeline,
};
use std::time::Duration as StdDuration;

use chrono::{DateTime, Utc};
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

pub struct JoinSchoolRequestRepo {
    pub collection: Collection<JoinSchoolRequest>,
}

impl JoinSchoolRequestRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<JoinSchoolRequest>("join_school_requests"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let email_index = IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(IndexOptions::builder().build())
            .build();

        let school_id_index = IndexModel::builder()
            .keys(doc! { "school_id": 1 })
            .options(IndexOptions::builder().build())
            .build();

        let invited_user_id_index = IndexModel::builder()
            .keys(doc! { "invited_user_id": 1 })
            .options(IndexOptions::builder().build())
            .build();

        let status_index = IndexModel::builder()
            .keys(doc! { "status": 1 })
            .options(IndexOptions::builder().build())
            .build();

        let role_index = IndexModel::builder()
            .keys(doc! { "role": 1 })
            .options(IndexOptions::builder().build())
            .build();

        let sent_by_index = IndexModel::builder()
            .keys(doc! { "sent_by": 1 })
            .options(IndexOptions::builder().build())
            .build();

        let expires_at_index = IndexModel::builder()
            .keys(doc! { "expires_at": 1 })
            .options(
                IndexOptions::builder()
                    .expire_after(Some(StdDuration::from_secs(0)))
                    .build(),
            )
            .build();

        let created_at_index = IndexModel::builder()
            .keys(doc! { "created_at": -1 })
            .options(IndexOptions::builder().build())
            .build();

        // Compound indexes for common query patterns
        let school_status_index = IndexModel::builder()
            .keys(doc! { "school_id": 1, "status": 1 })
            .options(IndexOptions::builder().build())
            .build();

        let email_status_index = IndexModel::builder()
            .keys(doc! { "email": 1, "status": 1 })
            .options(IndexOptions::builder().build())
            .build();

        let user_school_index = IndexModel::builder()
            .keys(doc! { "invited_user_id": 1, "school_id": 1 })
            .options(IndexOptions::builder().build())
            .build();

        let indexes = vec![
            email_index,
            school_id_index,
            invited_user_id_index,
            status_index,
            role_index,
            sent_by_index,
            expires_at_index,
            created_at_index,
            school_status_index,
            email_status_index,
            user_school_index,
        ];

        for index in indexes {
            self.collection
                .create_index(index)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to create index: {}", e),
                })?;
        }

        Ok(())
    }

    // Basic CRUD operations
    pub async fn create(
        &self,
        mut request: JoinSchoolRequest,
    ) -> Result<JoinSchoolRequest, AppError> {
        self.ensure_indexes().await?;

        let now = Utc::now();
        request.id = None;
        request.created_at = now;
        request.updated_at = now;

        // Set default expiration if not provided
        if request.expires_at.is_none() {
            request.expires_at = Some(now + chrono::Duration::days(7));
        }

        let result = self
            .collection
            .insert_one(&request)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create join request: {}", e),
            })?;

        let inserted_id = result.inserted_id.as_object_id().ok_or_else(|| AppError {
            message: "Failed to get inserted ID".to_string(),
        })?;

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Join request not found after creation".to_string(),
            })
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<JoinSchoolRequest>, AppError> {
        let obj_id = parse_object_id(id).map_err(|e| AppError { message: e })?;
        self.collection
            .find_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find join request by id: {}", e),
            })
    }

    pub async fn delete(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = parse_object_id(id).map_err(|e| AppError { message: e })?;
        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete join request: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "Join request not found for deletion".to_string(),
            });
        }

        Ok(())
    }

    // Find operations
    pub async fn find_by_email(&self, email: &str) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let mut cursor = self
            .collection
            .find(doc! { "email": email })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find join requests by email: {}", e),
            })?;

        let mut requests = Vec::new();
        while let Some(request) = cursor.next().await {
            requests.push(request.map_err(|e| AppError {
                message: format!("Failed to process join request: {}", e),
            })?);
        }
        Ok(requests)
    }

    // Get all join requests by email with relations
    pub async fn find_with_relations_by_email(
        &self,
        email: &str,
    ) -> Result<Vec<JoinSchoolRequestWithRelations>, AppError> {
        // Build aggregation pipeline with relations and match filter
        let pipeline = join_school_request_with_relations_pipeline(doc! { "email": email });

        let mut cursor = self
            .collection
            .clone_with_type::<Document>()
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to aggregate join requests with relations: {}", e),
            })?;

        let mut results = Vec::new();

        while let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to process aggregated document: {}", e),
        })? {
            // Deserialize base request
            let request: JoinSchoolRequest =
                mongodb::bson::from_document(doc.clone()).map_err(|e| AppError {
                    message: format!("Failed to deserialize join request: {}", e),
                })?;

            // Extract school, invited_user, and sender relationships
            let school: Option<School> = doc
                .get_array("school")
                .ok()
                .and_then(|arr| arr.first())
                .and_then(|bson| mongodb::bson::from_bson(bson.clone()).ok());

            let invited_user: Option<User> = doc
                .get_array("invited_user")
                .ok()
                .and_then(|arr| arr.first())
                .and_then(|bson| mongodb::bson::from_bson(bson.clone()).ok());

            let sender: Option<User> = doc
                .get_array("sender")
                .ok()
                .and_then(|arr| arr.first())
                .and_then(|bson| mongodb::bson::from_bson(bson.clone()).ok());

            results.push(JoinSchoolRequestWithRelations {
                request,
                school,
                invited_user,
                sender,
            });
        }

        Ok(results)
    }

    pub async fn find_by_school_id(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let obj_id = parse_object_id(school_id).map_err(|e| AppError { message: e })?;
        let mut cursor = self
            .collection
            .find(doc! { "school_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find join requests by school_id: {}", e),
            })?;

        let mut requests = Vec::new();
        while let Some(request) = cursor.next().await {
            requests.push(request.map_err(|e| AppError {
                message: format!("Failed to process join request: {}", e),
            })?);
        }
        Ok(requests)
    }

    pub async fn find_by_invited_user_id(
        &self,
        user_id: &IdType,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let obj_id = parse_object_id(user_id).map_err(|e| AppError { message: e })?;
        let mut cursor = self
            .collection
            .find(doc! { "invited_user_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find join requests by invited_user_id: {}", e),
            })?;

        let mut requests = Vec::new();
        while let Some(request) = cursor.next().await {
            requests.push(request.map_err(|e| AppError {
                message: format!("Failed to process join request: {}", e),
            })?);
        }
        Ok(requests)
    }

    pub async fn find_by_status(
        &self,
        status: JoinStatus,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let mut cursor = self
            .collection
            .find(
                doc! { "status": bson::to_bson(&status).map_err(|e| AppError {
                    message: format!("Failed to serialize status: {}", e),
                })? },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find join requests by status: {}", e),
            })?;

        let mut requests = Vec::new();
        while let Some(request) = cursor.next().await {
            requests.push(request.map_err(|e| AppError {
                message: format!("Failed to process join request: {}", e),
            })?);
        }
        Ok(requests)
    }

    // Complex find operations
    pub async fn find_by_school_and_status(
        &self,
        school_id: &IdType,
        status: JoinStatus,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let school_obj_id = parse_object_id(school_id).map_err(|e| AppError { message: e })?;
        let mut cursor = self
            .collection
            .find(doc! {
                "school_id": school_obj_id,
                "status": bson::to_bson(&status).map_err(|e| AppError {
                    message: format!("Failed to serialize status: {}", e),
                })?
            })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find join requests by school and status: {}", e),
            })?;

        let mut requests = Vec::new();
        while let Some(request) = cursor.next().await {
            requests.push(request.map_err(|e| AppError {
                message: format!("Failed to process join request: {}", e),
            })?);
        }
        Ok(requests)
    }

    pub async fn find_by_email_and_status(
        &self,
        email: &str,
        status: JoinStatus,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let mut cursor = self
            .collection
            .find(doc! {
                "email": email,
                "status": bson::to_bson(&status).map_err(|e| AppError {
                    message: format!("Failed to serialize status: {}", e),
                })?
            })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find join requests by email and status: {}", e),
            })?;

        let mut requests = Vec::new();
        while let Some(request) = cursor.next().await {
            requests.push(request.map_err(|e| AppError {
                message: format!("Failed to process join request: {}", e),
            })?);
        }
        Ok(requests)
    }

    pub async fn find_pending_by_email(
        &self,
        email: &str,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        self.find_by_email_and_status(email, JoinStatus::Pending)
            .await
    }

    pub async fn find_pending_by_email_and_school(
        &self,
        email: &str,
        school_id: &IdType,
    ) -> Result<Option<JoinSchoolRequest>, AppError> {
        let school_obj_id = parse_object_id(school_id).map_err(|e| AppError { message: e })?;
        self.collection
            .find_one(doc! {
                "email": email,
                "school_id": school_obj_id,
                "status": bson::to_bson(&JoinStatus::Pending).map_err(|e| AppError {
                    message: format!("Failed to serialize status: {}", e),
                })?
            })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find pending request by email and school: {}", e),
            })
    }

    // Status management operations
    pub async fn update_status(
        &self,
        id: &IdType,
        status: JoinStatus,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let obj_id = parse_object_id(id).map_err(|e| AppError { message: e })?;
        let now = Utc::now();

        let mut update_doc = doc! {
            "status": bson::to_bson(&status).map_err(|e| AppError {
                message: format!("Failed to serialize status: {}", e),
            })?,
            "updated_at": bson::to_bson(&now).unwrap(),
        };

        if let Some(responded_by_id) = responded_by {
            update_doc.insert("responded_by", responded_by_id);
        }

        // Only set responded_at for status changes that represent a response
        if matches!(
            status,
            JoinStatus::Accepted | JoinStatus::Rejected | JoinStatus::Cancelled
        ) {
            update_doc.insert("responded_at", bson::to_bson(&now).unwrap());
        }

        self.collection
            .update_one(doc! { "_id": obj_id }, doc! { "$set": update_doc })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update join request status: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Join request not found after status update".to_string(),
        })
    }

    pub async fn accept_request(
        &self,
        id: &IdType,
        invited_user_id: ObjectId,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let obj_id = parse_object_id(id).map_err(|e| AppError { message: e })?;
        let now = Utc::now();

        let mut update_doc = doc! {
            "status": bson::to_bson(&JoinStatus::Accepted).map_err(|e| AppError {
                message: format!("Failed to serialize status: {}", e),
            })?,
            "invited_user_id": invited_user_id,
            "responded_at": bson::to_bson(&now).unwrap(),
            "updated_at": bson::to_bson(&now).unwrap(),
        };

        if let Some(responded_by_id) = responded_by {
            update_doc.insert("responded_by", responded_by_id);
        }

        self.collection
            .update_one(doc! { "_id": obj_id }, doc! { "$set": update_doc })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to accept join request: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Join request not found after acceptance".to_string(),
        })
    }

    pub async fn reject_request(
        &self,
        id: &IdType,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        self.update_status(id, JoinStatus::Rejected, responded_by)
            .await
    }

    pub async fn cancel_request(
        &self,
        id: &IdType,
        responded_by: Option<ObjectId>,
    ) -> Result<JoinSchoolRequest, AppError> {
        self.update_status(id, JoinStatus::Cancelled, responded_by)
            .await
    }

    // Bulk operations
    pub async fn create_many(
        &self,
        requests: Vec<JoinSchoolRequest>,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        self.ensure_indexes().await?;

        let now = Utc::now();
        let mut requests_to_insert = Vec::with_capacity(requests.len());

        for mut request in requests {
            request.id = None;
            request.created_at = now;
            request.updated_at = now;

            if request.expires_at.is_none() {
                request.expires_at = Some(now + chrono::Duration::days(7));
            }

            requests_to_insert.push(request);
        }

        let result = self
            .collection
            .insert_many(&requests_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert multiple join requests: {}", e),
            })?;

        let inserted_ids: Vec<ObjectId> = result
            .inserted_ids
            .values()
            .filter_map(|bson| bson.as_object_id())
            .collect();

        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": inserted_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch inserted join requests: {}", e),
            })?;

        let mut inserted_requests = Vec::new();
        while let Some(request) = cursor.next().await {
            inserted_requests.push(request.map_err(|e| AppError {
                message: format!("Failed to process join request: {}", e),
            })?);
        }

        Ok(inserted_requests)
    }

    pub async fn bulk_update_status(
        &self,
        ids: Vec<ObjectId>,
        status: JoinStatus,
        responded_by: Option<ObjectId>,
    ) -> Result<u64, AppError> {
        let now = Utc::now();
        let mut update_doc = doc! {
            "status": bson::to_bson(&status).map_err(|e| AppError {
                message: format!("Failed to serialize status: {}", e),
            })?,
            "updated_at": bson::to_bson(&now).unwrap(),
        };

        if let Some(responded_by_id) = responded_by {
            update_doc.insert("responded_by", responded_by_id);
        }

        if matches!(
            status,
            JoinStatus::Accepted | JoinStatus::Rejected | JoinStatus::Cancelled
        ) {
            update_doc.insert("responded_at", bson::to_bson(&now).unwrap());
        }

        let result = self
            .collection
            .update_many(doc! { "_id": { "$in": ids } }, doc! { "$set": update_doc })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to bulk update join request status: {}", e),
            })?;

        Ok(result.modified_count)
    }

    // Query with pagination and filtering
    pub async fn query_requests(
        &self,
        query: crate::domain::join_school_request::JoinRequestQuery,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        let mut pipeline = vec![];

        let mut match_stage = Document::new();

        if let Some(email) = query.email {
            match_stage.insert("email", doc! { "$regex": email, "$options": "i" });
        }

        if let Some(school_id) = query.school_id {
            let school_obj_id =
                parse_object_id(&IdType::String(school_id)).map_err(|e| AppError { message: e })?;
            match_stage.insert("school_id", school_obj_id);
        }

        if let Some(status) = query.status {
            match_stage.insert(
                "status",
                bson::to_bson(&status).map_err(|e| AppError {
                    message: format!("Failed to serialize status: {}", e),
                })?,
            );
        }

        if let Some(role) = query.role {
            match_stage.insert(
                "role",
                bson::to_bson(&role).map_err(|e| AppError {
                    message: format!("Failed to serialize role: {}", e),
                })?,
            );
        }

        if let Some(older_than_days) = query.older_than_days {
            let cutoff_date = Utc::now() - chrono::Duration::days(older_than_days);
            match_stage.insert(
                "created_at",
                doc! { "$lte": bson::to_bson(&cutoff_date).unwrap() },
            );
        }

        // ✅ Apply $match stage
        if !match_stage.is_empty() {
            pipeline.push(doc! { "$match": match_stage });
        }

        // ✅ Sort newest first
        pipeline.push(doc! { "$sort": { "created_at": -1 } });

        // ✅ Skip and limit
        if let Some(skip) = query.skip {
            pipeline.push(doc! { "$skip": skip });
        }

        if let Some(limit) = query.limit {
            pipeline.push(doc! { "$limit": limit });
        }

        // ✅ Use aggregate instead of find
        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to query join requests: {}", e),
            })?;

        let mut requests = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate join requests: {}", e),
        })? {
            let request: JoinSchoolRequest =
                mongodb::bson::from_document(doc).map_err(|e| AppError {
                    message: format!("Failed to deserialize join request: {}", e),
                })?;
            requests.push(request);
        }

        Ok(requests)
    }

    // Add the missing methods that the controller expects
    pub async fn bulk_create(
        &self,
        requests: Vec<JoinSchoolRequest>,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        self.create_many(requests).await
    }

    pub async fn bulk_respond(
        &self,
        request: &crate::domain::join_school_request::BulkRespondRequest,
    ) -> Result<Vec<JoinSchoolRequest>, AppError> {
        // Convert all request_ids (String) to ObjectIds
        let ids: Result<Vec<ObjectId>, AppError> = request
            .request_ids
            .iter()
            .map(|id| {
                parse_object_id(&IdType::String(id.clone())).map_err(|e| AppError { message: e })
            })
            .collect();

        let object_ids = ids?;

        // Update all requests
        self.bulk_update_status(object_ids.clone(), request.status.clone(), None)
            .await?;

        // Fetch updated requests
        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated join requests: {}", e),
            })?;

        let mut updated_requests = Vec::new();
        while let Some(request) = cursor.next().await {
            updated_requests.push(request.map_err(|e| AppError {
                message: format!("Failed to process join request: {}", e),
            })?);
        }

        Ok(updated_requests)
    }

    pub async fn query_with_relations(
        &self,
        query: &crate::domain::join_school_request::JoinRequestQuery,
    ) -> Result<Vec<crate::domain::join_school_request::JoinSchoolRequestWithRelations>, AppError>
    {
        let requests = self.query_requests(query.clone()).await?;

        // For now, return without relations. You can implement the lookup logic later.
        let result = requests
            .into_iter()
            .map(
                |request| crate::domain::join_school_request::JoinSchoolRequestWithRelations {
                    request,
                    school: None,
                    invited_user: None,
                    sender: None,
                },
            )
            .collect();

        Ok(result)
    }

    pub async fn count_by_status(&self, status: JoinStatus) -> Result<u64, AppError> {
        self.collection
            .count_documents(
                doc! { "status": bson::to_bson(&status).map_err(|e| AppError {
                    message: format!("Failed to serialize status: {}", e),
                })? },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count join requests by status: {}", e),
            })
    }

    pub async fn count_by_school_and_status(
        &self,
        school_id: &IdType,
        status: JoinStatus,
    ) -> Result<u64, AppError> {
        let school_obj_id = parse_object_id(school_id).map_err(|e| AppError { message: e })?;
        self.collection
            .count_documents(doc! {
                "school_id": school_obj_id,
                "status": bson::to_bson(&status).map_err(|e| AppError {
                    message: format!("Failed to serialize status: {}", e),
                })?
            })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count join requests by school and status: {}", e),
            })
    }

    pub async fn count_pending_by_school(&self, school_id: &IdType) -> Result<u64, AppError> {
        self.count_by_school_and_status(school_id, JoinStatus::Pending)
            .await
    }

    // Expiration and cleanup operations
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

    // Check for duplicate/conflicting requests
    pub async fn has_pending_request(
        &self,
        email: &str,
        school_id: &IdType,
    ) -> Result<bool, AppError> {
        let school_obj_id = parse_object_id(school_id).map_err(|e| AppError { message: e })?;
        let count = self
            .collection
            .count_documents(doc! {
                "email": email,
                "school_id": school_obj_id,
                "status": bson::to_bson(&JoinStatus::Pending).map_err(|e| AppError {
                    message: format!("Failed to serialize status: {}", e),
                })?
            })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to check for pending request: {}", e),
            })?;

        Ok(count > 0)
    }

    // Update expiration date
    pub async fn update_expiration(
        &self,
        id: &IdType,
        expires_at: DateTime<Utc>,
    ) -> Result<JoinSchoolRequest, AppError> {
        let obj_id = parse_object_id(id).map_err(|e| AppError { message: e })?;
        let update_doc = doc! {
            "expires_at": bson::to_bson(&expires_at).unwrap(),
            "updated_at": bson::to_bson(&Utc::now()).unwrap(),
        };

        self.collection
            .update_one(doc! { "_id": obj_id }, doc! { "$set": update_doc })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update expiration: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Join request not found after expiration update".to_string(),
        })
    }

    // Get requests with relations
    pub async fn find_with_relations_by_id(
        &self,
        id: &IdType,
    ) -> Result<Option<crate::domain::join_school_request::JoinSchoolRequestWithRelations>, AppError>
    {
        let obj_id = parse_object_id(id).map_err(|e| AppError { message: e })?;
        let pipeline = join_school_request_with_relations_pipeline(doc! {"_id": obj_id});

        let mut cursor = self
            .collection
            .clone_with_type::<Document>()
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to aggregate join request with relations: {}", e),
            })?;

        if let Some(doc) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to process aggregated result: {}", e),
        })? {
            let request: JoinSchoolRequest =
                mongodb::bson::from_document(doc.clone()).map_err(|e| AppError {
                    message: format!("Failed to deserialize join request: {}", e),
                })?;

            let school: Option<School> = doc
                .get_array("school")
                .ok()
                .and_then(|arr| arr.first())
                .and_then(|bson| mongodb::bson::from_bson(bson.clone()).ok());

            let invited_user: Option<User> = doc
                .get_array("invited_user")
                .ok()
                .and_then(|arr| arr.first())
                .and_then(|bson| mongodb::bson::from_bson(bson.clone()).ok());

            let sender: Option<User> = doc
                .get_array("sender")
                .ok()
                .and_then(|arr| arr.first())
                .and_then(|bson| mongodb::bson::from_bson(bson.clone()).ok());

            let result = crate::domain::join_school_request::JoinSchoolRequestWithRelations {
                request,
                school,
                invited_user,
                sender,
            };

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}
