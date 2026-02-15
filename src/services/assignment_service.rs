use chrono::Utc;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};

use crate::{
    domain::{
        assignment::{
            Assignment, AssignmentPartial, AssignmentStatus, AssignmentWithRelations, Submission,
            SubmissionPartial, SubmissionStatus, SubmissionWithRelations,
        },
        common_details::Paginated,
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::assignment_pipeline::{
        assignment_with_teacher_pipeline, submission_with_relations_pipeline,
    },
    repositories::base_repo::BaseRepository,
    services::cloudinary_service::CloudinaryService,
    utils::mongo_utils::{build_search_filter, extract_valid_fields},
};

pub struct AssignmentService {
    pub collection: Collection<Assignment>,
    pub submission_collection: Collection<Submission>,
    pub db: Database,
}

impl AssignmentService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Assignment>("assignments"),
            submission_collection: db.collection::<Submission>("submissions"),
            db: db.clone(),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let assignment_indexes = vec![
            IndexDef::single("school_id", false),
            IndexDef::single("class_id", false),
            IndexDef::single("subject_id", false),
            IndexDef::single("teacher_id", false),
            IndexDef::single("due_date", false),
            IndexDef::single("status", false),
            IndexDef::single("is_deleted", false),
            IndexDef::compound(vec![("school_id", 1), ("class_id", 1)], false),
            IndexDef::compound(vec![("school_id", 1), ("subject_id", 1)], false),
            IndexDef::compound(vec![("school_id", 1), ("teacher_id", 1)], false),
        ];

        let submission_indexes = vec![
            IndexDef::single("assignment_id", false),
            IndexDef::single("student_id", false),
            IndexDef::single("graded_by", false),
            IndexDef::single("submitted_at", false),
            IndexDef::single("status", false),
            IndexDef::single("is_deleted", false),
            IndexDef::compound(vec![("assignment_id", 1), ("student_id", 1)], true),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&assignment_indexes).await?;

        let submission_repo =
            BaseRepository::new(self.submission_collection.clone().clone_with_type::<Document>());
        submission_repo.ensure_indexes(&submission_indexes).await?;

        Ok(())
    }

    // =========================
    // ASSIGNMENT CRUD
    // =========================

    pub async fn create_assignment(
        &self,
        dto: Assignment,
    ) -> Result<Assignment, AppError> {
        self.ensure_indexes().await?;

        if dto.title.trim().is_empty() {
            return Err(AppError {
                message: "Assignment title is required".into(),
            });
        }

        if dto.max_score <= 0.0 {
            return Err(AppError {
                message: "Max score must be greater than 0".into(),
            });
        }

        // Validate teacher is assigned to subject
        if let (Some(teacher_id), Some(subject_id)) = (dto.teacher_id, dto.subject_id) {
            let subject_collection = self.db.collection::<Document>("class_subjects");
            let subject = subject_collection
                .find_one(doc! { "_id": subject_id })
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to validate subject: {}", e),
                })?;

            if let Some(subject_doc) = subject {
                if let Some(assigned_teacher_id) = subject_doc.get_object_id("teacher_id").ok() {
                    if assigned_teacher_id != teacher_id {
                        return Err(AppError {
                            message: "Teacher is not assigned to this subject".into(),
                        });
                    }
                }
            }
        }

        let mut assignment = dto;

        // Handle attachment upload
        if let Some(attachment_data) = assignment.attachment_url.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&attachment_data)
                .await
                .map_err(|e| AppError { message: e })?;
            assignment.attachment_id = Some(cloud_res.public_id);
            assignment.attachment_url = Some(cloud_res.secure_url);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create::<Assignment>(extract_valid_fields(assignment.to_document()?), None)
            .await
    }

    pub async fn find_one_assignment(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Assignment, AppError> {
        let mut filter = extra_match.unwrap_or_default();
        filter.insert("is_deleted", false);

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.find_one::<Assignment>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Assignment not found".into(),
            })
    }

    pub async fn get_all_assignments(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Assignment>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_doc = extra_match.unwrap_or_default();
        match_doc.insert("is_deleted", false);

        let searchable = [
            "title",
            "description",
            "_id",
            "school_id",
            "class_id",
            "subject_id",
            "teacher_id",
            "status",
        ];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Assignment>(filter, &searchable, limit, skip, Some(match_doc))
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn get_all_assignments_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<AssignmentWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_stage = extra_match.unwrap_or_default();
        match_stage.insert("is_deleted", false);

        if let Some(f) = filter {
            let search = build_search_filter(
                Some(f),
                &[
                    "title",
                    "description",
                    "_id",
                    "school_id",
                    "class_id",
                    "subject_id",
                    "teacher_id",
                    "status",
                ],
            );
            match_stage.extend(search);
        }

        let pipeline = assignment_with_teacher_pipeline(match_stage);
        repo.aggregate_with_paginate::<AssignmentWithRelations>(pipeline, limit, skip)
            .await
    }

    pub async fn find_one_assignment_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<AssignmentWithRelations, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();
        match_stage.insert("is_deleted", false);

        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.aggregate_one::<AssignmentWithRelations>(
            assignment_with_teacher_pipeline(match_stage),
            None,
        )
        .await?
        .ok_or(AppError {
            message: "Assignment not found".into(),
        })
    }

    pub async fn update_assignment(
        &self,
        id: &IdType,
        update: &AssignmentPartial,
    ) -> Result<Assignment, AppError> {
        let existing = self.find_one_assignment(Some(id), None).await?;

        if let Some(ref title) = update.title {
            if title.trim().is_empty() {
                return Err(AppError {
                    message: "Assignment title cannot be empty".into(),
                });
            }
        }

        if let Some(max_score) = update.max_score {
            if max_score <= 0.0 {
                return Err(AppError {
                    message: "Max score must be greater than 0".into(),
                });
            }
        }

        let mut update_data = update.clone();

        // Handle attachment update
        if let Some(new_attachment) = update.attachment_url.clone().flatten() {
            if Some(new_attachment.clone()) != existing.attachment_url {
                if let Some(old_attachment_id) = existing.attachment_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_attachment_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_attachment)
                    .await
                    .map_err(|e| AppError { message: e })?;

                update_data.attachment_id = Some(Some(cloud_res.public_id));
                update_data.attachment_url = Some(Some(cloud_res.secure_url));
            }
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<Assignment>(
            id,
            extract_valid_fields(Assignment::from_partial(update_data)?),
        )
        .await
    }

    pub async fn delete_assignment(&self, id: &IdType) -> Result<Assignment, AppError> {
        let assignment = self.find_one_assignment(Some(id), None).await?;

        // Soft delete
        let update = AssignmentPartial {
            is_deleted: Some(true),
            deleted_at: Some(Some(Utc::now())),
            ..Default::default()
        };

        self.update_assignment(id, &update).await?;

        Ok(assignment)
    }

    pub async fn count_assignments(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_doc = extra_match.unwrap_or_default();
        match_doc.insert("is_deleted", false);

        let searchable = [
            "title",
            "description",
            "school_id",
            "class_id",
            "subject_id",
            "teacher_id",
            "status",
        ];

        repo.count(filter, &searchable, Some(match_doc)).await
    }

    // =========================
    // SUBMISSION CRUD
    // =========================

    pub async fn validate_submission_deadline(
        &self,
        assignment: &Assignment,
    ) -> Result<bool, AppError> {
        let now = Utc::now();
        let is_late = now > assignment.due_date;

        if is_late && !assignment.allow_late_submission {
            return Err(AppError {
                message: "Submission deadline has passed and late submissions are not allowed"
                    .into(),
            });
        }

        Ok(is_late)
    }

    pub async fn create_submission(
        &self,
        dto: Submission,
    ) -> Result<Submission, AppError> {
        self.ensure_indexes().await?;

        // Validate assignment exists
        let assignment_id_type = dto
            .assignment_id
            .map(|id| IdType::ObjectId(id))
            .ok_or(AppError {
                message: "Assignment ID is required".into(),
            })?;

        let assignment = self
            .find_one_assignment(Some(&assignment_id_type), None)
            .await?;

        // Check if assignment is published
        if !matches!(assignment.status, AssignmentStatus::Published) {
            return Err(AppError {
                message: "Assignment is not published".into(),
            });
        }

        // Validate deadline
        let is_late = self.validate_submission_deadline(&assignment).await?;

        // Check for existing submission
        if let (Some(assignment_id), Some(student_id)) = (dto.assignment_id, dto.student_id) {
            let existing = self
                .submission_collection
                .find_one(
                    doc! {
                        "assignment_id": assignment_id,
                        "student_id": student_id,
                        "is_deleted": false
                    }
                )
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to check existing submission: {}", e),
                })?;

            if existing.is_some() {
                return Err(AppError {
                    message: "You have already submitted this assignment. Use update instead."
                        .into(),
                });
            }
        }

        let mut submission = dto;
        submission.is_late = is_late;

        // Handle file upload
        if let Some(file_data) = submission.file_url.clone() {
            let cloud_res = CloudinaryService::upload_to_cloudinary(&file_data)
                .await
                .map_err(|e| AppError { message: e })?;
            submission.file_id = Some(cloud_res.public_id);
            submission.file_url = Some(cloud_res.secure_url);
        }

        let repo =
            BaseRepository::new(self.submission_collection.clone().clone_with_type::<Document>());
        repo.create::<Submission>(extract_valid_fields(submission.to_document()?), None)
            .await
    }

    pub async fn find_one_submission(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Submission, AppError> {
        let mut filter = extra_match.unwrap_or_default();
        filter.insert("is_deleted", false);

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo =
            BaseRepository::new(self.submission_collection.clone().clone_with_type::<Document>());
        repo.find_one::<Submission>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Submission not found".into(),
            })
    }

    pub async fn get_all_submissions(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Submission>, AppError> {
        let repo =
            BaseRepository::new(self.submission_collection.clone().clone_with_type::<Document>());

        let mut match_doc = extra_match.unwrap_or_default();
        match_doc.insert("is_deleted", false);

        let searchable = [
            "_id",
            "assignment_id",
            "student_id",
            "graded_by",
            "status",
            "is_late",
        ];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Submission>(filter, &searchable, limit, skip, Some(match_doc))
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn get_all_submissions_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<SubmissionWithRelations>, AppError> {
        let repo =
            BaseRepository::new(self.submission_collection.clone().clone_with_type::<Document>());

        let mut match_stage = extra_match.unwrap_or_default();
        match_stage.insert("is_deleted", false);

        if let Some(f) = filter {
            let search = build_search_filter(
                Some(f),
                &[
                    "_id",
                    "assignment_id",
                    "student_id",
                    "graded_by",
                    "status",
                ],
            );
            match_stage.extend(search);
        }

        let pipeline = submission_with_relations_pipeline(match_stage);
        repo.aggregate_with_paginate::<SubmissionWithRelations>(pipeline, limit, skip)
            .await
    }

    pub async fn update_submission(
        &self,
        id: &IdType,
        update: &SubmissionPartial,
    ) -> Result<Submission, AppError> {
        let existing = self.find_one_submission(Some(id), None).await?;

        // If grading, validate score
        if let Some(score) = update.score.flatten() {
            if let Some(assignment_id) = existing.assignment_id {
                let assignment = self
                    .find_one_assignment(Some(&IdType::ObjectId(assignment_id)), None)
                    .await?;

                if score > assignment.max_score {
                    return Err(AppError {
                        message: format!(
                            "Score cannot exceed max score of {}",
                            assignment.max_score
                        ),
                    });
                }
            }
        }

        let mut update_data = update.clone();

        // Handle file update
        if let Some(new_file) = update.file_url.clone().flatten() {
            if Some(new_file.clone()) != existing.file_url {
                if let Some(old_file_id) = existing.file_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_file_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&new_file)
                    .await
                    .map_err(|e| AppError { message: e })?;

                update_data.file_id = Some(Some(cloud_res.public_id));
                update_data.file_url = Some(Some(cloud_res.secure_url));
            }
        }

        // Handle feedback file upload
        if let Some(feedback_file) = update.feedback_file_url.clone().flatten() {
            if Some(feedback_file.clone()) != existing.feedback_file_url {
                if let Some(old_feedback_file_id) = existing.feedback_file_id.clone() {
                    CloudinaryService::delete_from_cloudinary(&old_feedback_file_id)
                        .await
                        .ok();
                }

                let cloud_res = CloudinaryService::upload_to_cloudinary(&feedback_file)
                    .await
                    .map_err(|e| AppError { message: e })?;

                update_data.feedback_file_id = Some(Some(cloud_res.public_id));
                update_data.feedback_file_url = Some(Some(cloud_res.secure_url));
            }
        }

        let repo =
            BaseRepository::new(self.submission_collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<Submission>(
            id,
            extract_valid_fields(Submission::from_partial(update_data)?),
        )
        .await
    }

    pub async fn grade_submission(
        &self,
        submission_id: &IdType,
        score: f64,
        feedback: Option<String>,
        feedback_file: Option<String>,
        graded_by: mongodb::bson::oid::ObjectId,
    ) -> Result<Submission, AppError> {
        let submission = self.find_one_submission(Some(submission_id), None).await?;

        // Validate score against assignment max_score
        if let Some(assignment_id) = submission.assignment_id {
            let assignment = self
                .find_one_assignment(Some(&IdType::ObjectId(assignment_id)), None)
                .await?;

            if score > assignment.max_score {
                return Err(AppError {
                    message: format!(
                        "Score cannot exceed max score of {}",
                        assignment.max_score
                    ),
                });
            }
        }

        let mut update = SubmissionPartial {
            score: Some(Some(score)),
            feedback: Some(feedback),
            graded_by: Some(Some(graded_by)),
            graded_at: Some(Some(Utc::now())),
            status: Some(SubmissionStatus::Graded),
            ..Default::default()
        };

        if let Some(file) = feedback_file {
            update.feedback_file_url = Some(Some(file));
        }

        self.update_submission(submission_id, &update).await
    }

    pub async fn delete_submission(&self, id: &IdType) -> Result<Submission, AppError> {
        let submission = self.find_one_submission(Some(id), None).await?;

        // Soft delete
        let update = SubmissionPartial {
            is_deleted: Some(true),
            deleted_at: Some(Some(Utc::now())),
            ..Default::default()
        };

        self.update_submission(id, &update).await?;

        Ok(submission)
    }

    // =========================
    // PLACEHOLDER FOR FUTURE AI AUTO-GRADING
    // =========================
    #[allow(dead_code)]
    pub async fn trigger_auto_grading(&self, _submission_id: &IdType) -> Result<(), AppError> {
        // Placeholder for future AI integration
        // This would:
        // 1. Fetch submission and assignment
        // 2. Send to AI service for grading
        // 3. Update submission with auto_grade_score and ai_feedback
        Ok(())
    }
}
