use crate::{
    domain::subjects::subject_progress_tracking_config::{
        DefaultSubjectProgressThresholds, SubjectProgressTrackingConfig,
        SubjectProgressTrackingConfigType, UpdateSubjectProgressTrackingConfig,
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
        // ✅ Check if config already exists for this reference_id
        if let Some(reference_id) = &new_config.reference_id {
            let id_type = IdType::from_object_id(*reference_id);
            if let Ok(Some(_)) = self.repo.find_by_reference_id(&id_type).await {
                return Err("Progress config already exists for this reference".to_string());
            }
        }

        // Validate thresholds
        self.validate_thresholds(&new_config.thresholds)?;

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

    /// Get progress config by reference_id
    pub async fn get_config_by_reference_id(
        &self,
        reference_id: &IdType,
    ) -> Result<SubjectProgressTrackingConfig, String> {
        self.repo
            .find_by_reference_id(reference_id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Progress config not found for this reference".to_string())
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
            let effective_thresholds = crate::domain::subjects::subject_progress_tracking_config::SubjectProgressThresholds {
                satisfactory: upd_thresholds.satisfactory.unwrap_or(config.thresholds.satisfactory),
                needs_improvement: upd_thresholds.needs_improvement.unwrap_or(config.thresholds.needs_improvement),
                at_risk: upd_thresholds.at_risk.unwrap_or(config.thresholds.at_risk),
            };
            self.validate_thresholds(&effective_thresholds)?
        }

        // Prevent changing reference_id (not allowed in Update struct)

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

    /// Bulk get configs by reference_ids
    pub async fn get_configs_by_reference_ids(
        &self,
        reference_ids: &[ObjectId],
    ) -> Result<Vec<SubjectProgressTrackingConfig>, String> {
        self.repo
            .find_by_reference_ids(reference_ids)
            .await
            .map_err(|e| e.message)
    }

    /// Validate threshold values
    fn validate_thresholds(
        &self,
        thresholds: &crate::domain::subjects::subject_progress_tracking_config::SubjectProgressThresholds,
    ) -> Result<(), String> {
        if thresholds.satisfactory < 0.0 || thresholds.satisfactory > 100.0 {
            return Err("Satisfactory threshold must be between 0.0 and 100.0".to_string());
        }
        if thresholds.needs_improvement < 0.0 || thresholds.needs_improvement > 100.0 {
            return Err("Needs improvement threshold must be between 0.0 and 100.0".to_string());
        }
        if thresholds.at_risk < 0.0 || thresholds.at_risk > 100.0 {
            return Err("At risk threshold must be between 0.0 and 100.0".to_string());
        }

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

    /// Get or create default config for a reference
    pub async fn get_or_create_default_config(
        &self,
        default: DefaultSubjectProgressThresholds,
    ) -> Result<SubjectProgressTrackingConfig, String> {
        // Try to get existing config
        match self.get_config_by_reference_id(&default.reference_id).await {
            Ok(config) => Ok(config),
            Err(_) => {
                // Create default config if not exists
                let default_config = SubjectProgressTrackingConfig {
                    id: None,
                    reference_id: Some(ObjectId::parse_str(default.reference_id.as_string()).map_err(|e| e.to_string())?),
                    track_attendance: true,
                    track_assignments: true,
                    track_topic_coverage: true,
                    track_skill_acquisition: false,
                    thresholds: crate::domain::subjects::subject_progress_tracking_config::SubjectProgressThresholds {
                        satisfactory: 80.0,
                        needs_improvement: 60.0,
                        at_risk: 40.0,
                    },
                    role: SubjectProgressTrackingConfigType::MainSubject,
                    created_by: default.created_by,
                    created_at: None,
                    updated_at: None,
                };

                self.create_config(default_config).await
            }
        }
    }
}
