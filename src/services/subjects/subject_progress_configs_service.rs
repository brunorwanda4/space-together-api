use crate::{
    domain::subjects::subject_progress_tracking_config::{
        SubjectProgressTrackingConfig, UpdateSubjectProgressTrackingConfig,
    },
    models::id_model::IdType,
    repositories::subjects::subject_progress_configs_repo::SubjectProgressConfigsRepo,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct SubjectProgressConfigsService<'a> {
    repo: &'a SubjectProgressConfigsRepo,
}

impl<'a> SubjectProgressConfigsService<'a> {
    pub fn new(repo: &'a SubjectProgressConfigsRepo) -> Self {
        Self { repo }
    }

    /// Get all progress configs
    pub async fn get_all_configs(&self) -> Result<Vec<SubjectProgressTrackingConfig>, String> {
        self.repo.get_all_configs().await.map_err(|e| e.message)
    }

    /// Create a new progress config
    pub async fn create_config(
        &self,
        mut new_config: SubjectProgressTrackingConfig,
    ) -> Result<SubjectProgressTrackingConfig, String> {
        // ✅ Check if config already exists for this main_subject_id
        if let Some(main_subject_id) = &new_config.main_subject_id {
            let id_type = IdType::from_object_id(main_subject_id.clone());
            if let Ok(Some(_)) = self.repo.find_by_main_subject_id(&id_type).await {
                return Err("Progress config already exists for this subject".to_string());
            }
        }

        // Validate thresholds
        if let Err(threshold_error) = self.validate_thresholds(&new_config.thresholds) {
            return Err(threshold_error);
        }

        let now = Some(Utc::now());
        new_config.created_at = now;
        new_config.updated_at = now;

        // Ensure Mongo generates id
        new_config.id = Some(ObjectId::new());

        let inserted_config = self
            .repo
            .insert_config(&new_config)
            .await
            .map_err(|e| e.message)?;
        Ok(inserted_config)
    }

    /// Get progress config by ID
    pub async fn get_config_by_id(
        &self,
        id: &IdType,
    ) -> Result<SubjectProgressTrackingConfig, String> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Progress config not found".to_string())
    }

    /// Get progress config by main_subject_id
    pub async fn get_config_by_main_subject_id(
        &self,
        main_subject_id: &IdType,
    ) -> Result<SubjectProgressTrackingConfig, String> {
        self.repo
            .find_by_main_subject_id(main_subject_id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Progress config not found for this subject".to_string())
    }

    /// Update a progress config by id
    pub async fn update_config(
        &self,
        id: &IdType,
        updated_data: UpdateSubjectProgressTrackingConfig,
    ) -> Result<SubjectProgressTrackingConfig, String> {
        // Fetch existing config
        let config = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Progress config not found".to_string())?;

        // Validate thresholds if they're being updated
        if let Some(ref upd_thresholds) = updated_data.thresholds {
            // Merge update thresholds with existing config thresholds to get effective values for validation
            let effective_thresholds = crate::domain::subjects::subject_progress_tracking_config::SubjectProgressThresholds {
                satisfactory: upd_thresholds.satisfactory.unwrap_or(config.thresholds.satisfactory),
                needs_improvement: upd_thresholds.needs_improvement.unwrap_or(config.thresholds.needs_improvement),
                at_risk: upd_thresholds.at_risk.unwrap_or(config.thresholds.at_risk),
            };
            self.validate_thresholds(&effective_thresholds)?
        }

        // Prevent changing main_subject_id to one that already has a config
        // (Note: main_subject_id is not in UpdateSubjectProgressTrackingConfig by design)

        let updated_config = self
            .repo
            .update_config(id, updated_data)
            .await
            .map_err(|e| e.message)?;
        Ok(updated_config)
    }

    /// Delete a progress config by id
    pub async fn delete_config(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_config(id).await.map_err(|e| e.message)
    }

    /// Bulk get configs by main_subject_ids
    pub async fn get_configs_by_main_subject_ids(
        &self,
        main_subject_ids: &[ObjectId],
    ) -> Result<Vec<SubjectProgressTrackingConfig>, String> {
        self.repo
            .find_by_main_subject_ids(main_subject_ids)
            .await
            .map_err(|e| e.message)
    }

    /// Validate threshold values
    fn validate_thresholds(
        &self,
        thresholds: &crate::domain::subjects::subject_progress_tracking_config::SubjectProgressThresholds,
    ) -> Result<(), String> {
        // Ensure thresholds are in valid range (0.0 - 100.0)
        if thresholds.satisfactory < 0.0 || thresholds.satisfactory > 100.0 {
            return Err("Satisfactory threshold must be between 0.0 and 100.0".to_string());
        }
        if thresholds.needs_improvement < 0.0 || thresholds.needs_improvement > 100.0 {
            return Err("Needs improvement threshold must be between 0.0 and 100.0".to_string());
        }
        if thresholds.at_risk < 0.0 || thresholds.at_risk > 100.0 {
            return Err("At risk threshold must be between 0.0 and 100.0".to_string());
        }

        // Ensure thresholds are in correct order: satisfactory > needs_improvement > at_risk
        if thresholds.satisfactory <= thresholds.needs_improvement {
            return Err(
                "Satisfactory threshold must be greater than needs improvement threshold"
                    .to_string(),
            );
        }
        if thresholds.needs_improvement <= thresholds.at_risk {
            return Err(
                "Needs improvement threshold must be greater than at risk threshold".to_string(),
            );
        }

        Ok(())
    }

    /// Get or create default config for a subject
    pub async fn get_or_create_default_config(
        &self,
        main_subject_id: &IdType,
        created_by: Option<ObjectId>,
    ) -> Result<SubjectProgressTrackingConfig, String> {
        // Try to get existing config
        match self.get_config_by_main_subject_id(main_subject_id).await {
            Ok(config) => Ok(config),
            Err(_) => {
                // Create default config if not exists
                let default_config = SubjectProgressTrackingConfig {
                    id: None,
                    main_subject_id: Some(ObjectId::parse_str(main_subject_id.as_string()).map_err(|e| e.to_string())?),
                    track_attendance: true,
                    track_assignments: true,
                    track_topic_coverage: true,
                    track_skill_acquisition: false,
                    thresholds: crate::domain::subjects::subject_progress_tracking_config::SubjectProgressThresholds {
                        satisfactory: 80.0,
                        needs_improvement: 60.0,
                        at_risk: 40.0,
                    },
                    created_by,
                    created_at: None,
                    updated_at: None,
                };

                self.create_config(default_config).await
            }
        }
    }
}
