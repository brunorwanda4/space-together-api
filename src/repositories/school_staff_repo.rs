use crate::domain::school_staff::{
    BulkIdsRequest, BulkTagsRequest, BulkUpdateActiveStatusRequest, SchoolStaff, SchoolStaffType,
    SchoolStaffWithRelations, UpdateSchoolStaff,
};
use crate::errors::AppError;
use crate::helpers::aggregate_helpers::{aggregate_many, aggregate_single};
use crate::models::id_model::IdType;
use crate::pipeline::school_staff_pipeline::school_staff_with_relations_pipeline;
use crate::utils::object_id::parse_object_id;

use chrono::Utc;
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

pub struct SchoolStaffRepo {
    pub collection: Collection<SchoolStaff>,
}

impl SchoolStaffRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<SchoolStaff>("school_staff"),
        }
    }

    pub async fn get_all_with_relations(&self) -> Result<Vec<SchoolStaffWithRelations>, AppError> {
        aggregate_many(&self.collection.clone().clone_with_type::<Document>(), {
            let mut pipeline = school_staff_with_relations_pipeline(doc! {});
            pipeline.insert(0, doc! { "$sort": { "updated_at": -1 } });
            pipeline
        })
        .await
    }

    pub async fn find_by_id_with_relations(
        &self,
        id: &IdType,
    ) -> Result<Option<SchoolStaffWithRelations>, AppError> {
        let obj_id = parse_object_id(id)?;
        aggregate_single(
            &self.collection.clone().clone_with_type::<Document>(),
            school_staff_with_relations_pipeline(doc! { "_id": obj_id }),
        )
        .await
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<SchoolStaff>, AppError> {
        let obj_id = parse_object_id(id)?;
        self.collection
            .find_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find school staff by id: {}", e),
            })
    }

    pub async fn find_by_user_id(&self, user_id: &IdType) -> Result<Option<SchoolStaff>, AppError> {
        let obj_id = parse_object_id(user_id)?;
        self.collection
            .find_one(doc! { "user_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find school staff by user_id: {}", e),
            })
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<SchoolStaff>, AppError> {
        self.collection
            .find_one(doc! { "email": email })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find school staff by email: {}", e),
            })
    }

    pub async fn find_by_school_id(
        &self,
        school_id: &IdType,
    ) -> Result<Vec<SchoolStaff>, AppError> {
        let obj_id = parse_object_id(school_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "school_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find school staff by school_id: {}", e),
            })?;

        let mut staff_members = Vec::new();
        while let Some(staff) = cursor.next().await {
            staff_members.push(staff.map_err(|e| AppError {
                message: format!("Failed to process school staff: {}", e),
            })?);
        }
        Ok(staff_members)
    }

    pub async fn find_by_creator_id(
        &self,
        creator_id: &IdType,
    ) -> Result<Vec<SchoolStaff>, AppError> {
        let obj_id = parse_object_id(creator_id)?;
        let mut cursor = self
            .collection
            .find(doc! { "creator_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find school staff by creator_id: {}", e),
            })?;

        let mut staff_members = Vec::new();
        while let Some(staff) = cursor.next().await {
            staff_members.push(staff.map_err(|e| AppError {
                message: format!("Failed to process school staff: {}", e),
            })?);
        }
        Ok(staff_members)
    }

    pub async fn find_by_type(
        &self,
        staff_type: SchoolStaffType,
    ) -> Result<Vec<SchoolStaff>, AppError> {
        let mut cursor = self
            .collection
            .find(
                doc! { "type": bson::to_bson(&staff_type).map_err(|e| AppError {
                    message: format!("Failed to serialize staff type: {}", e),
                })? },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find school staff by type: {}", e),
            })?;

        let mut staff_members = Vec::new();
        while let Some(staff) = cursor.next().await {
            staff_members.push(staff.map_err(|e| AppError {
                message: format!("Failed to process school staff: {}", e),
            })?);
        }
        Ok(staff_members)
    }

    pub async fn find_by_school_and_type(
        &self,
        school_id: &IdType,
        staff_type: SchoolStaffType,
    ) -> Result<Vec<SchoolStaff>, AppError> {
        let obj_id = parse_object_id(school_id)?;
        let mut cursor = self
            .collection
            .find(doc! {
                "school_id": obj_id,
                "type": bson::to_bson(&staff_type).map_err(|e| AppError {
                    message: format!("Failed to serialize staff type: {}", e),
                })?
            })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find school staff by school_id and type: {}", e),
            })?;

        let mut staff_members = Vec::new();
        while let Some(staff) = cursor.next().await {
            staff_members.push(staff.map_err(|e| AppError {
                message: format!("Failed to process school staff: {}", e),
            })?);
        }
        Ok(staff_members)
    }

    pub async fn insert_school_staff(&self, staff: &SchoolStaff) -> Result<SchoolStaff, AppError> {
        self.ensure_indexes().await?;

        let mut to_insert = staff.clone();
        to_insert.id = None;
        to_insert.created_at = Utc::now();
        to_insert.updated_at = Utc::now();

        let res = self
            .collection
            .insert_one(&to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert school staff: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to extract inserted school staff id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "School staff not found after insert".to_string(),
            })
    }

    async fn ensure_indexes(&self) -> Result<(), AppError> {
        let email_index = IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let user_id_index = IndexModel::builder()
            .keys(doc! { "user_id": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        let school_index = IndexModel::builder()
            .keys(doc! { "school_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let creator_index = IndexModel::builder()
            .keys(doc! { "creator_id": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let type_index = IndexModel::builder()
            .keys(doc! { "type": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let is_active_index = IndexModel::builder()
            .keys(doc! { "is_active": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        let school_type_index = IndexModel::builder()
            .keys(doc! { "school_id": 1, "type": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        // ✅ One Director per school
        let one_director_per_school_index = IndexModel::builder()
            .keys(doc! { "school_id": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .partial_filter_expression(doc! { "type": "Director" })
                    .name(Some("unique_director_per_school".to_string()))
                    .build(),
            )
            .build();

        for index in [
            email_index,
            user_id_index,
            school_index,
            creator_index,
            type_index,
            is_active_index,
            school_type_index,
            one_director_per_school_index,
        ] {
            self.collection
                .create_index(index)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to create index: {}", e),
                })?;
        }

        Ok(())
    }

    pub async fn get_all_school_staff(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
    ) -> Result<Vec<SchoolStaff>, AppError> {
        let mut pipeline = vec![];

        // Add search/filter functionality
        if let Some(f) = filter {
            let regex = doc! {
                "$regex": f,
                "$options": "i"  // case insensitive
            };
            pipeline.push(doc! {
                "$match": {
                    "$or": [
                        { "name": &regex },
                        { "email": &regex },
                        { "tags": &regex },
                    ]
                }
            });
        }

        // Add sorting by updated_at (most recent first)
        pipeline.push(doc! { "$sort": { "updated_at": -1 } });

        // Add pagination
        if let Some(s) = skip {
            pipeline.push(doc! { "$skip": s });
        }

        if let Some(l) = limit {
            pipeline.push(doc! { "$limit": l });
        } else {
            // Default limit if none provided
            pipeline.push(doc! { "$limit": 50 });
        }

        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch school staff: {}", e),
            })?;

        let mut staff_members = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate school staff: {}", e),
        })? {
            let staff: SchoolStaff =
                mongodb::bson::from_document(result).map_err(|e| AppError {
                    message: format!("Failed to deserialize school staff: {}", e),
                })?;
            staff_members.push(staff);
        }

        Ok(staff_members)
    }

    pub async fn get_active_school_staff(&self) -> Result<Vec<SchoolStaff>, AppError> {
        let mut cursor = self
            .collection
            .find(doc! { "is_active": true })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find active school staff: {}", e),
            })?;

        let mut staff_members = Vec::new();
        while let Some(staff) = cursor.next().await {
            staff_members.push(staff.map_err(|e| AppError {
                message: format!("Failed to process school staff: {}", e),
            })?);
        }
        Ok(staff_members)
    }

    pub async fn update_school_staff(
        &self,
        id: &IdType,
        update: &UpdateSchoolStaff,
    ) -> Result<SchoolStaff, AppError> {
        let obj_id = parse_object_id(id)?;

        // Create update document manually to handle Option fields
        let mut update_doc = Document::new();

        if let Some(name) = &update.name {
            update_doc.insert("name", name);
        }
        if let Some(email) = &update.email {
            update_doc.insert("email", email);
        }
        if let Some(staff_type) = &update.r#type {
            update_doc.insert(
                "type",
                bson::to_bson(staff_type).map_err(|e| AppError {
                    message: format!("Failed to serialize staff type: {}", e),
                })?,
            );
        }
        if let Some(is_active) = update.is_active {
            update_doc.insert("is_active", is_active);
        }
        if let Some(tags) = &update.tags {
            update_doc.insert("tags", tags);
        }

        update_doc.insert("updated_at", bson::to_bson(&Utc::now()).unwrap());

        let update_doc = doc! { "$set": update_doc };

        self.collection
            .update_one(doc! { "_id": obj_id }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update school staff: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "School staff not found after update".to_string(),
        })
    }

    pub async fn delete_school_staff(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = parse_object_id(id)?;
        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete school staff: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No school staff deleted; it may not exist".to_string(),
            });
        }
        Ok(())
    }

    pub async fn count_by_school_id(
        &self,
        school_id: &IdType,
        staff_type: Option<SchoolStaffType>,
        is_active: Option<bool>,
    ) -> Result<u64, AppError> {
        let obj_id = parse_object_id(school_id)?;

        // Base filter
        let mut filter = doc! { "school_id": obj_id };

        // Optional filters
        if let Some(t) = staff_type {
            filter.insert("type", t.to_string());
        }

        if let Some(active) = is_active {
            filter.insert("is_active", active);
        }

        self.collection
            .count_documents(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count school staff by school_id: {}", e),
            })
    }

    pub async fn count_by_creator_id(&self, creator_id: &IdType) -> Result<u64, AppError> {
        let obj_id = parse_object_id(creator_id)?;
        self.collection
            .count_documents(doc! { "creator_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count school staff by creator_id: {}", e),
            })
    }

    pub async fn count_by_type(&self, staff_type: SchoolStaffType) -> Result<u64, AppError> {
        self.collection
            .count_documents(
                doc! { "type": bson::to_bson(&staff_type).map_err(|e| AppError {
                    message: format!("Failed to serialize staff type: {}", e),
                })? },
            )
            .await
            .map_err(|e| AppError {
                message: format!("Failed to count school staff by type: {}", e),
            })
    }

    // Bulk operations
    pub async fn create_many_school_staff(
        &self,
        staff_members: Vec<SchoolStaff>,
    ) -> Result<Vec<SchoolStaff>, AppError> {
        self.ensure_indexes().await?;

        let mut staff_to_insert = Vec::with_capacity(staff_members.len());
        let now = Utc::now();

        for mut staff in staff_members {
            staff.id = None;
            staff.created_at = now;
            staff.updated_at = now;
            staff_to_insert.push(staff);
        }

        let result = self
            .collection
            .insert_many(&staff_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert multiple school staff: {}", e),
            })?;

        let inserted_ids: Vec<ObjectId> = result
            .inserted_ids
            .values()
            .filter_map(|bson| bson.as_object_id())
            .collect();

        if inserted_ids.len() != staff_to_insert.len() {
            return Err(AppError {
                message: "Failed to get all inserted school staff IDs".to_string(),
            });
        }

        let mut inserted_staff = Vec::with_capacity(inserted_ids.len());
        for id in inserted_ids {
            let staff = self.find_by_id(&IdType::from_object_id(id)).await?;
            if let Some(staff) = staff {
                inserted_staff.push(staff);
            }
        }

        Ok(inserted_staff)
    }

    pub async fn create_many_school_staff_with_validation(
        &self,
        staff_members: Vec<SchoolStaff>,
    ) -> Result<Vec<SchoolStaff>, AppError> {
        self.ensure_indexes().await?;

        let mut emails = std::collections::HashSet::new();
        let mut user_ids = std::collections::HashSet::new();

        for staff in &staff_members {
            if !emails.insert(&staff.email) {
                return Err(AppError {
                    message: format!("Duplicate email found: {}", staff.email),
                });
            }

            if let Some(user_id) = &staff.user_id {
                if !user_ids.insert(user_id) {
                    return Err(AppError {
                        message: format!("Duplicate user_id found: {}", user_id),
                    });
                }
            }
        }

        for staff in &staff_members {
            if let Some(existing) = self.find_by_email(&staff.email).await? {
                return Err(AppError {
                    message: format!("Email already exists: {}", existing.email),
                });
            }

            if let Some(user_id) = &staff.user_id {
                if let Some(_existing) = self
                    .find_by_user_id(&IdType::from_object_id(*user_id))
                    .await?
                {
                    return Err(AppError {
                        message: format!("User ID already exists: {}", user_id),
                    });
                }
            }
        }

        self.create_many_school_staff(staff_members).await
    }

    pub async fn update_many_school_staff(
        &self,
        updates: Vec<(IdType, UpdateSchoolStaff)>,
    ) -> Result<Vec<SchoolStaff>, AppError> {
        let mut updated_staff = Vec::with_capacity(updates.len());

        for (id, update) in updates {
            match self.update_school_staff(&id, &update).await {
                Ok(staff) => updated_staff.push(staff),
                Err(e) => {
                    eprintln!("Failed to update school staff {:?}: {}", id, e.message);
                }
            }
        }

        if updated_staff.is_empty() {
            return Err(AppError {
                message: "No school staff were successfully updated".to_string(),
            });
        }

        Ok(updated_staff)
    }

    pub async fn bulk_update_active_status(
        &self,
        request: &BulkUpdateActiveStatusRequest,
    ) -> Result<Vec<SchoolStaff>, AppError> {
        let ids: Result<Vec<ObjectId>, AppError> = request
            .ids
            .iter()
            .map(|id| parse_object_id(&IdType::String(id.clone())))
            .collect();

        let object_ids = ids?;

        let update_doc = doc! {
            "$set": {
                "is_active": request.is_active,
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_many(doc! { "_id": { "$in": &object_ids } }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to bulk update active status: {}", e),
            })?;

        // Return updated staff members
        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated school staff: {}", e),
            })?;

        let mut updated_staff = Vec::new();
        while let Some(staff) = cursor.next().await {
            updated_staff.push(staff.map_err(|e| AppError {
                message: format!("Failed to process school staff: {}", e),
            })?);
        }

        Ok(updated_staff)
    }

    pub async fn bulk_add_tags(
        &self,
        request: &BulkTagsRequest,
    ) -> Result<Vec<SchoolStaff>, AppError> {
        let ids: Result<Vec<ObjectId>, AppError> = request
            .ids
            .iter()
            .map(|id| parse_object_id(&IdType::String(id.clone())))
            .collect();

        let object_ids = ids?;

        let update_doc = doc! {
            "$addToSet": {
                "tags": { "$each": &request.tags }
            },
            "$set": {
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_many(doc! { "_id": { "$in": &object_ids } }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to bulk add tags: {}", e),
            })?;

        // Return updated staff members
        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated school staff: {}", e),
            })?;

        let mut updated_staff = Vec::new();
        while let Some(staff) = cursor.next().await {
            updated_staff.push(staff.map_err(|e| AppError {
                message: format!("Failed to process school staff: {}", e),
            })?);
        }

        Ok(updated_staff)
    }

    pub async fn bulk_remove_tags(
        &self,
        request: &BulkTagsRequest,
    ) -> Result<Vec<SchoolStaff>, AppError> {
        let ids: Result<Vec<ObjectId>, AppError> = request
            .ids
            .iter()
            .map(|id| parse_object_id(&IdType::String(id.clone())))
            .collect();

        let object_ids = ids?;

        let update_doc = doc! {
            "$pullAll": {
                "tags": &request.tags
            },
            "$set": {
                "updated_at": bson::to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_many(doc! { "_id": { "$in": &object_ids } }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to bulk remove tags: {}", e),
            })?;

        // Return updated staff members
        let mut cursor = self
            .collection
            .find(doc! { "_id": { "$in": &object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch updated school staff: {}", e),
            })?;

        let mut updated_staff = Vec::new();
        while let Some(staff) = cursor.next().await {
            updated_staff.push(staff.map_err(|e| AppError {
                message: format!("Failed to process school staff: {}", e),
            })?);
        }

        Ok(updated_staff)
    }

    pub async fn delete_many_school_staff(
        &self,
        request: &BulkIdsRequest,
    ) -> Result<u64, AppError> {
        let ids: Result<Vec<ObjectId>, AppError> = request
            .ids
            .iter()
            .map(|id| parse_object_id(&IdType::String(id.clone())))
            .collect();

        let object_ids = ids?;

        let result = self
            .collection
            .delete_many(doc! { "_id": { "$in": object_ids } })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete multiple school staff: {}", e),
            })?;

        Ok(result.deleted_count)
    }
}
