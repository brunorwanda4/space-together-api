use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use futures::TryStreamExt;

use crate::{
    config::state::AppState,
    domain::{
        announcement::AnnouncementWithRelations,
        common_details::Paginated,
        parent::{
            AttendanceSummary, ChildSummary, FinanceSummary, Parent, ParentDashboard,
            ParentPartial, ParentStatus, ParentWithRelations, StudentResults, SubjectGrade,
        },
        student_term_result::StudentTermResult,
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::parent_pipeline::{
        attendance_summary_pipeline, finance_summary_pipeline, parent_pipeline,
        student_results_pipeline,
    },
    repositories::base_repo::BaseRepository,
    services::{
        announcement_service::AnnouncementService, cloudinary_service::CloudinaryService,
    },
    utils::{
        email::is_valid_email,
        mongo_utils::{build_search_filter, extract_valid_fields},
        names::is_valid_name,
    },
};

pub struct ParentService {
    pub collection: Collection<Parent>,
}

impl ParentService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Parent>("parents"),
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
            IndexDef::single("student_ids", false),
            IndexDef::single("status", false),
            IndexDef::single("is_active", false),
            IndexDef::compound(vec![("school_id", 1), ("status", 1)], false),
        ];

        let repo = BaseRepository::new(
            self.collection
                .clone()
                .clone_with_type::<mongodb::bson::Document>(),
        );

        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    // =========================
    // CREATE
    // =========================
    pub async fn create(&self, dto: Parent) -> Result<Parent, AppError> {
        self.ensure_indexes().await?;

        if let Err(e) = is_valid_name(&dto.name) {
            return Err(AppError { message: e });
        }

        if let Err(e) = is_valid_email(&dto.email) {
            return Err(AppError { message: e });
        }

        if let Ok(parent) = self.find_one(None, Some(doc! {"email": &dto.email})).await {
            return Err(AppError {
                message: format!("Email already exists: {}", parent.email),
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

        if matches!(partial.status, ParentStatus::Active) {
            partial.status = ParentStatus::Active;
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let parent = repo
            .create::<Parent>(extract_valid_fields(partial.to_document()?), None)
            .await?;

        Ok(parent)
    }

    // =========================
    // FIND ONE
    // =========================
    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Parent, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<Parent>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Parent not found".into(),
            })
    }

    // =========================
    // GET ALL (PLAIN)
    // =========================
    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Parent>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "email",
            "_id",
            "user_id",
            "school_id",
            "phone",
            "gender",
            "relationship",
            "status",
        ];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Parent>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    // =========================
    // UPDATE
    // =========================
    pub async fn update(&self, id: &IdType, update: &ParentPartial) -> Result<Parent, AppError> {
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

        let existing_parent = self.find_one(Some(id), None).await?;

        if let Some(ref email) = update.email {
            if existing_parent.email != *email {
                if let Ok(parent) = self.find_one(None, Some(doc! { "email": email })).await {
                    return Err(AppError {
                        message: format!("Email already exists: {}", parent.email),
                    });
                }
            }
        }

        let mut update_data = update.clone();

        if let Some(new_image_data) = update.image.clone().flatten() {
            if Some(new_image_data.clone()) != existing_parent.image {
                if let Some(old_image_id) = existing_parent.image_id.clone() {
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
        repo.update_one_and_fetch::<Parent>(
            id,
            extract_valid_fields(Parent::from_partial(update_data)?),
        )
        .await
    }

    // =========================
    // DELETE
    // =========================
    pub async fn delete(&self, id: &IdType) -> Result<Parent, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let parent = self.find_one(Some(id), None).await?;

        if let Some(ref image_id) = parent.image_id {
            CloudinaryService::delete_from_cloudinary(image_id)
                .await
                .ok();
        }

        repo.delete_one(id).await?;

        Ok(parent)
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<ParentWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(f) = filter {
            let search = build_search_filter(
                Some(f),
                &[
                    "name",
                    "email",
                    "phone",
                    "gender",
                    "status",
                    "_id",
                    "user_id",
                    "school_id",
                    "relationship",
                ],
            );

            match_stage.extend(search);
        }

        let pipeline = parent_pipeline(match_stage);

        repo.aggregate_with_paginate::<ParentWithRelations>(pipeline, limit, skip)
            .await
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<ParentWithRelations, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.aggregate_one::<ParentWithRelations>(parent_pipeline(match_stage), None)
            .await?
            .ok_or(AppError {
                message: "Parent not found".into(),
            })
    }

    // =========================
    // COUNT
    // =========================
    pub async fn count_parents(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "name",
            "email",
            "phone",
            "gender",
            "school_id",
            "relationship",
            "status",
        ];

        repo.count(filter, &searchable, extra_match).await
    }

    // =========================
    // PARENT-SPECIFIC METHODS
    // =========================

    /// Validate that a parent has access to a specific student
    pub async fn validate_parent_student_access(
        &self,
        parent_id: &IdType,
        student_id: &IdType,
    ) -> Result<bool, AppError> {
        let parent = self.find_one(Some(parent_id), None).await?;

        let student_oid = IdType::to_object_id(student_id)?;

        if let Some(student_ids) = parent.student_ids {
            Ok(student_ids.contains(&student_oid))
        } else {
            Ok(false)
        }
    }

    /// Get parent dashboard with aggregated data
    pub async fn get_dashboard(
        &self,
        parent_id: &IdType,
        school_id: &str,
        state: &AppState,
    ) -> Result<ParentDashboard, AppError> {
        let parent = self.find_one(Some(parent_id), None).await?;

        let student_ids = parent.student_ids.clone().unwrap_or_default();
        let total_children = student_ids.len() as i64;

        // Get latest announcements
        let db = state.db.get_db(&state.db.school_db_name_from_id(school_id));
        let announcement_service = AnnouncementService::new(&db);

        let announcements_result = announcement_service
            .get_all_with_relations(None, Some(5), None, Some(doc! {}))
            .await
            .unwrap_or_else(|_| Paginated {
                data: vec![],
                total: 0,
                total_pages: 0,
                current_page: 1,
            });

        // Get children summary
        let mut children_summary = Vec::new();
        for student_id in student_ids {
            if let Ok(summary) = self
                .get_child_summary(&student_id.to_hex(), school_id, state)
                .await
            {
                children_summary.push(summary);
            }
        }

        Ok(ParentDashboard {
            total_children,
            latest_announcements: announcements_result.data,
            children_summary,
        })
    }

    /// Get summary for a single child
    async fn get_child_summary(
        &self,
        student_id: &str,
        school_id: &str,
        state: &AppState,
    ) -> Result<ChildSummary, AppError> {
        let db = state.db.get_db(&state.db.school_db_name_from_id(school_id));
        let students_collection = db.collection::<crate::domain::student::Student>("students");

        let student_oid = mongodb::bson::oid::ObjectId::parse_str(student_id)
            .map_err(|_| AppError {
                message: "Invalid student ID".into(),
            })?;

        let student = students_collection
            .find_one(doc! { "_id": student_oid })
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
            })?
            .ok_or(AppError {
                message: "Student not found".into(),
            })?;

        // Get class name
        let class_name = if let Some(class_id) = student.class_id {
            let classes_collection = db.collection::<crate::domain::class::Class>("classes");
            classes_collection
                .find_one(doc! { "_id": class_id })
                .await
                .ok()
                .flatten()
                .and_then(|c| Some(c.name))
        } else {
            None
        };

        // Calculate attendance percentage (placeholder)
        let attendance_percentage = 85.0;

        // Get current term GPA (placeholder)
        let current_term_gpa = 3.5;

        // Get outstanding fees (placeholder)
        let outstanding_fees = 0.0;

        Ok(ChildSummary {
            student_id: Some(student_oid),
            student_name: student.name,
            class_name,
            attendance_percentage,
            current_term_gpa,
            outstanding_fees,
        })
    }

    /// Get attendance summary for a student
    pub async fn get_attendance_summary(
        &self,
        student_id: &str,
        school_id: &str,
        state: &AppState,
    ) -> Result<AttendanceSummary, AppError> {
        let db = state.db.get_db(&state.db.school_db_name_from_id(school_id));
        let repo = BaseRepository::new(db.collection::<Document>("attendance"));

        let pipeline = attendance_summary_pipeline(student_id, school_id);

        let result: Vec<Document> = repo
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Aggregation error: {}", e),
            })?
            .try_collect::<Vec<Document>>()
            .await
            .map_err(|e| AppError {
                message: format!("Collection error: {}", e),
            })?;

        if result.is_empty() {
            return Ok(AttendanceSummary {
                present_count: 0,
                absent_count: 0,
                late_count: 0,
                excused_count: 0,
                total_days: 0,
                attendance_percentage: 0.0,
                recent_records: vec![],
            });
        }

        // Parse aggregation result
        let mut present_count = 0i64;
        let mut absent_count = 0i64;
        let mut late_count = 0i64;
        let mut excused_count = 0i64;
        let recent_records = vec![];

        if let Some(summary) = result[0].get_array("summary").ok() {
            for item in summary {
                if let Some(doc) = item.as_document() {
                    let status = doc.get_str("_id").unwrap_or("");
                    let count = doc.get_i64("count").unwrap_or(0);

                    match status {
                        "Present" => present_count = count,
                        "Absent" => absent_count = count,
                        "Late" => late_count = count,
                        "Excused" => excused_count = count,
                        _ => {}
                    }
                }
            }
        }

        let total_days = present_count + absent_count + late_count + excused_count;
        let attendance_percentage = if total_days > 0 {
            (present_count as f64 / total_days as f64) * 100.0
        } else {
            0.0
        };

        Ok(AttendanceSummary {
            present_count,
            absent_count,
            late_count,
            excused_count,
            total_days,
            attendance_percentage,
            recent_records,
        })
    }

    /// Get student results
    pub async fn get_student_results(
        &self,
        student_id: &str,
        school_id: &str,
        education_year_id: Option<&str>,
        term_id: Option<&str>,
        state: &AppState,
    ) -> Result<StudentResults, AppError> {
        let db = state.db.get_db(&state.db.school_db_name_from_id(school_id));
        let repo = BaseRepository::new(db.collection::<Document>("student_term_results"));

        let pipeline = student_results_pipeline(student_id, school_id, education_year_id, term_id);

        let result: Option<StudentTermResult> = repo
            .aggregate_one::<StudentTermResult>(pipeline, None)
            .await?;

        if let Some(term_result) = result {
            let subject_results: Vec<SubjectGrade> = term_result
                .subject_results
                .iter()
                .map(|sr| SubjectGrade {
                    subject_name: sr.subject_name.clone(),
                    score: sr.weighted_score,
                    max_score: 100.0,
                    percentage: sr.percentage,
                    grade: sr.grade.clone(),
                })
                .collect();

            Ok(StudentResults {
                term_gpa: term_result.gpa,
                rank: term_result.rank_in_class,
                total_students: term_result.total_students,
                grade: term_result.grade,
                subject_results,
                teacher_remarks: None,
            })
        } else {
            Err(AppError {
                message: "No results found for this student".into(),
            })
        }
    }

    /// Get finance summary for a student
    pub async fn get_finance_summary(
        &self,
        student_id: &str,
        school_id: &str,
        state: &AppState,
    ) -> Result<FinanceSummary, AppError> {
        let db = state.db.get_db(&state.db.school_db_name_from_id(school_id));
        let repo = BaseRepository::new(db.collection::<Document>("enrollments"));

        let pipeline = finance_summary_pipeline(student_id, school_id);

        let _result: Vec<Document> = repo
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Aggregation error: {}", e),
            })?
            .try_collect::<Vec<Document>>()
            .await
            .map_err(|e| AppError {
                message: format!("Collection error: {}", e),
            })?;

        // Placeholder implementation
        Ok(FinanceSummary {
            total_fee_required: 1000.0,
            amount_paid: 600.0,
            outstanding_balance: 400.0,
            payment_history: vec![],
            installments: vec![],
        })
    }

    /// Get announcements accessible to parent
    pub async fn get_parent_announcements(
        &self,
        parent_id: &IdType,
        school_id: &str,
        limit: Option<i64>,
        skip: Option<i64>,
        state: &AppState,
    ) -> Result<Paginated<AnnouncementWithRelations>, AppError> {
        let parent = self.find_one(Some(parent_id), None).await?;

        let db = state.db.get_db(&state.db.school_db_name_from_id(school_id));
        let announcement_service = AnnouncementService::new(&db);

        // Get class IDs of parent's children
        let mut class_ids = Vec::new();
        if let Some(student_ids) = parent.student_ids {
            let students_collection =
                db.collection::<crate::domain::student::Student>("students");

            for student_id in student_ids {
                if let Ok(Some(student)) = students_collection
                    .find_one(doc! { "_id": student_id })
                    .await
                {
                    if let Some(class_id) = student.class_id {
                        class_ids.push(class_id);
                    }
                }
            }
        }

        // Get announcements for school-wide or specific classes
        let filter = if class_ids.is_empty() {
            doc! {
                "$or": [
                    { "classes_ids": { "$exists": false } },
                    { "classes_ids": { "$size": 0 } }
                ]
            }
        } else {
            doc! {
                "$or": [
                    { "classes_ids": { "$exists": false } },
                    { "classes_ids": { "$size": 0 } },
                    { "classes_ids": { "$in": class_ids } }
                ]
            }
        };

        announcement_service
            .get_all_with_relations(None, limit, skip, Some(filter))
            .await
    }

    /// Check if a user is a parent of a specific student
    pub async fn is_parent_of(
        &self,
        parent_id: &IdType,
        student_id: &IdType,
    ) -> Result<bool, AppError> {
        self.validate_parent_student_access(parent_id, student_id)
            .await
    }
}
