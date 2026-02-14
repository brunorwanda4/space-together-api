use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};

use crate::{
    domain::{
        common_details::Paginated,
        score::{Score, ScoreAuditLog, ScorePartial},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::IndexDef,
    },
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct ScoreService {
    pub collection: Collection<Score>,
    pub audit_collection: Collection<ScoreAuditLog>,
}

impl ScoreService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Score>("scores"),
            audit_collection: db.collection::<ScoreAuditLog>("score_audit_logs"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::compound(
                vec![("school_id", 1), ("student_id", 1), ("exam_id", 1)],
                false,
            ),
            IndexDef::compound(
                vec![("school_id", 1), ("class_subject_id", 1), ("exam_id", 1)],
                false,
            ),
            IndexDef::compound(
                vec![
                    ("student_id", 1),
                    ("class_subject_id", 1),
                    ("exam_id", 1),
                    ("assessment_category_id", 1),
                ],
                true, // unique
            ),
            IndexDef::single("is_deleted", false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    pub async fn create(&self, mut score: Score) -> Result<Score, AppError> {
        self.ensure_indexes().await?;

        // Calculate percentage
        score.percentage = if score.max_score > 0.0 {
            (score.score / score.max_score) * 100.0
        } else {
            0.0
        };

        // Validate score <= max_score
        if score.score > score.max_score {
            return Err(AppError {
                message: "Score cannot exceed maximum score".to_string(),
            });
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let mut doc = extract_valid_fields(mongodb::bson::to_document(&score).map_err(|e| AppError {
            message: format!("Failed to serialize score: {}", e),
        })?);
        doc.insert("is_deleted", false);

        repo.create::<Score>(doc, None).await.map_err(|e| {
            if e.message.contains("duplicate key") {
                AppError {
                    message: "Score already exists for this student, subject, exam, and assessment category".to_string(),
                }
            } else {
                e
            }
        })
    }

    pub async fn create_many(&self, scores: Vec<Score>) -> Result<Vec<Score>, AppError> {
        let mut created_scores = Vec::new();

        for score in scores {
            match self.create(score).await {
                Ok(s) => created_scores.push(s),
                Err(e) => {
                    eprintln!("Failed to create score: {}", e.message);
                }
            }
        }

        Ok(created_scores)
    }

    pub async fn find_one(&self, id: &IdType) -> Result<Score, AppError> {
        let filter = doc! {
            "_id": IdType::to_object_id(id)?,
            "is_deleted": false
        };

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.find_one::<Score>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Score not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Score>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = [
            "_id",
            "school_id",
            "student_id",
            "class_subject_id",
            "exam_id",
            "assessment_category_id",
        ];

        let mut match_filter = extra_match.unwrap_or_default();
        match_filter.insert("is_deleted", false);

        let (data, total, total_pages, current_page) = repo
            .get_all::<Score>(filter, &searchable, limit, skip, Some(match_filter))
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
        update: &ScorePartial,
        changed_by: &ObjectId,
        change_reason: Option<String>,
    ) -> Result<Score, AppError> {
        let existing = self.find_one(id).await?;

        let mut update_doc = Document::new();
        let mut new_score_value = existing.score;

        if let Some(score) = update.score {
            if let Some(max_score) = update.max_score.or(Some(existing.max_score)) {
                if score > max_score {
                    return Err(AppError {
                        message: "Score cannot exceed maximum score".to_string(),
                    });
                }
                let percentage = if max_score > 0.0 {
                    (score / max_score) * 100.0
                } else {
                    0.0
                };
                update_doc.insert("score", score);
                update_doc.insert("percentage", percentage);
                new_score_value = score;
            }
        }

        if let Some(max_score) = update.max_score {
            update_doc.insert("max_score", max_score);
            let score_val = update.score.unwrap_or(existing.score);
            let percentage = if max_score > 0.0 {
                (score_val / max_score) * 100.0
            } else {
                0.0
            };
            update_doc.insert("percentage", percentage);
        }

        if let Some(remarks) = &update.remarks {
            update_doc.insert("remarks", remarks);
        }

        // Create audit log if score changed
        if new_score_value != existing.score {
            let audit_log = ScoreAuditLog {
                id: None,
                score_id: existing.id,
                old_score: existing.score,
                new_score: new_score_value,
                changed_by: Some(*changed_by),
                change_reason,
                changed_at: Some(chrono::Utc::now()),
            };
            self.audit_collection.insert_one(audit_log).await?;
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<Score>(id, update_doc).await
    }

    pub async fn delete(&self, id: &IdType) -> Result<Score, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let update_doc = doc! { "is_deleted": true };

        repo.update_one_and_fetch::<Score>(id, update_doc).await
    }

    pub async fn get_student_exam_scores(
        &self,
        student_id: &ObjectId,
        exam_id: &ObjectId,
    ) -> Result<Vec<Score>, AppError> {
        let filter = doc! {
            "student_id": student_id,
            "exam_id": exam_id,
            "is_deleted": false
        };

        let mut cursor = self.collection.find(filter).await?;
        let mut scores = Vec::new();

        while cursor.advance().await? {
            scores.push(cursor.deserialize_current()?);
        }

        Ok(scores)
    }

    pub async fn get_audit_logs(&self, score_id: &ObjectId) -> Result<Vec<ScoreAuditLog>, AppError> {
        let filter = doc! { "score_id": score_id };
        let mut cursor = self
            .audit_collection
            .find(filter)
            .sort(doc! { "changed_at": -1 })
            .await?;

        let mut logs = Vec::new();
        while cursor.advance().await? {
            logs.push(cursor.deserialize_current()?);
        }

        Ok(logs)
    }
}
