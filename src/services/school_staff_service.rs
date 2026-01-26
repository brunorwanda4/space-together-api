use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};

use crate::{
    config::state::AppState,
    domain::{
        common_details::Paginated,
        join_school_request::{CreateJoinSchoolRequest, JoinRole},
        school_staff::{SchoolStaff, SchoolStaffPartial},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    repositories::base_repo::BaseRepository,
    services::{
        cloudinary_service::CloudinaryService,
        join_school_request_service::JoinSchoolRequestService,
    },
    utils::{email::is_valid_email, mongo_utils::extract_valid_fields, names::is_valid_name},
};

pub struct SchoolStaffService {
    pub collection: Collection<SchoolStaff>,
}

impl SchoolStaffService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<SchoolStaff>("school_staff"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("email", true),
            IndexDef::single_with_partial(
                "user_id",
                true,
                doc! { "user_id": { "$type": "objectId" } },
                Some("user_id_objectid_unique"),
            ),
            IndexDef::single("school_id", false),
            IndexDef::single("creator_id", false),
            IndexDef::single("type", false),
            IndexDef::single("is_active", false),
            IndexDef::compound(vec![("school_id", 1), ("type", 1)], false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    pub async fn create(
        &self,
        dto: SchoolStaff,
        state: Option<&AppState>,
    ) -> Result<SchoolStaff, AppError> {
        self.ensure_indexes().await?;

        if let Err(e) = is_valid_name(&dto.name) {
            return Err(AppError { message: e });
        }

        if let Err(e) = is_valid_email(&dto.email) {
            return Err(AppError { message: e });
        }

        if let Ok(existing) = self
            .find_one(None, Some(doc! { "email": &dto.email }))
            .await
        {
            return Err(AppError {
                message: format!("Email already exists: {}", existing.email),
            });
        }

        let mut partial = dto;
        if let Some(image_data) = partial.image.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&image_data)
                .await
                .map_err(|e| AppError { message: e })?;
            partial.image_id = Some(cloud_res.public_id);
            partial.image = Some(cloud_res.secure_url);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let school_staff = repo
            .create::<SchoolStaff>(extract_valid_fields(partial.to_document()?), None)
            .await?;

        if let Some(school_id) = school_staff.school_id {
            if let Some(sent_by) = school_staff.creator_id {
                if let Some(app_state) = state {
                    let create_request = CreateJoinSchoolRequest {
                        school_id: school_id.to_string(),
                        class_id: None,
                        message: Some("Join School request".to_string()),
                        r#type: "SchoolStaff".to_string(),
                        role: JoinRole::Staff,
                        email: school_staff.email.clone(),
                        sent_by: sent_by.to_hex(),
                    };

                    let join_request_service =
                        JoinSchoolRequestService::new(&app_state.db.main_db());

                    let _join_request = join_request_service
                        .create(create_request, sent_by, app_state)
                        .await
                        .ok();
                }
            }
        }

        Ok(school_staff)
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<SchoolStaff, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<SchoolStaff>(filter, None)
            .await?
            .ok_or(AppError {
                message: "School staff not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<SchoolStaff>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "email", "tags", "type", "_id"];

        let (data, total, total_pages, current_page) = repo
            .get_all::<SchoolStaff>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn update(
        &self,
        id: &IdType,
        update: &SchoolStaffPartial,
    ) -> Result<SchoolStaff, AppError> {
        if let Some(ref name) = update.name {
            if let Err(e) = is_valid_name(name) {
                return Err(AppError { message: e });
            }
        }

        if let Some(ref email) = update.email {
            if let Err(e) = is_valid_email(email) {
                return Err(AppError { message: e });
            }
        }

        let existing = self.find_one(Some(id), None).await?;

        if let Some(ref email) = update.email {
            if existing.email != *email {
                if let Ok(staff) = self.find_one(None, Some(doc! { "email": email })).await {
                    return Err(AppError {
                        message: format!("Email already exists: {}", staff.email),
                    });
                }
            }
        }

        let mut update_data = update.clone();

        if let Some(new_image_data) = update.image.clone().flatten() {
            if Some(new_image_data.clone()) != existing.image {
                if let Some(old_image_id) = existing.image_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_image_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_image_data)
                    .await
                    .map_err(|e| AppError { message: e })?;

                update_data.image_id = Some(Some(cloud_res.public_id));
                update_data.image = Some(Some(cloud_res.secure_url));
            }
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.update_one_and_fetch::<SchoolStaff>(
            id,
            extract_valid_fields(SchoolStaff::from_partial(update_data)?),
        )
        .await
    }

    pub async fn delete(&self, id: &IdType) -> Result<SchoolStaff, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let staff = self.find_one(Some(id), None).await?;

        repo.delete_one(id).await?;

        Ok(staff)
    }

    pub async fn count_staff(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "email",
            "tags",
            "type",
            "school_id",
            "creator_id",
            "is_active",
        ];

        repo.count(filter, &searchable, extra_match).await
    }
}
