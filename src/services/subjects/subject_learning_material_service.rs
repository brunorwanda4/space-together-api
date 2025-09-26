use crate::{
    domain::subjects::subject_learning_material::{
        SubjectLearningMaterial, UpdateSubjectLearningMaterial,
    },
    models::id_model::IdType,
    repositories::subjects::subject_learning_material_repo::SubjectLearningMaterialRepo,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;

pub struct SubjectLearningMaterialService<'a> {
    repo: &'a SubjectLearningMaterialRepo,
}

impl<'a> SubjectLearningMaterialService<'a> {
    pub fn new(repo: &'a SubjectLearningMaterialRepo) -> Self {
        Self { repo }
    }

    /// Get all learning materials
    pub async fn get_all_materials(&self) -> Result<Vec<SubjectLearningMaterial>, String> {
        self.repo.get_all_materials().await.map_err(|e| e.message)
    }

    /// Create a new learning material
    pub async fn create_material(
        &self,
        mut new_material: SubjectLearningMaterial,
    ) -> Result<SubjectLearningMaterial, String> {
        // ✅ Validate required fields
        if new_material.title.trim().is_empty() {
            return Err("Material title is required".to_string());
        }

        // ✅ Validate link if provided
        if let Some(ref link) = new_material.link {
            if !link.trim().is_empty() && !self.is_valid_url(link) {
                return Err("Invalid URL format for material link".to_string());
            }
        }

        // ✅ Validate material type specific rules
        if let Err(validation_error) = self.validate_material_type(&new_material) {
            return Err(validation_error);
        }

        let now = Some(Utc::now());
        new_material.created_at = now;
        new_material.updated_at = now;

        // Ensure material is active by default
        new_material.is_active = true;

        // Ensure Mongo generates id
        new_material.id = Some(ObjectId::new());

        let inserted_material = self
            .repo
            .insert_material(&new_material)
            .await
            .map_err(|e| e.message)?;
        Ok(inserted_material)
    }

    /// Get learning material by ID
    pub async fn get_material_by_id(&self, id: &IdType) -> Result<SubjectLearningMaterial, String> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Learning material not found".to_string())
    }

    /// Get learning materials by subject_id
    pub async fn get_materials_by_subject_id(
        &self,
        subject_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, String> {
        self.repo
            .find_by_subject_id(subject_id)
            .await
            .map_err(|e| e.message)
    }

    /// Get active learning materials by subject_id
    pub async fn get_active_materials_by_subject_id(
        &self,
        subject_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, String> {
        self.repo
            .find_active_by_subject_id(subject_id)
            .await
            .map_err(|e| e.message)
    }

    /// Get learning materials by type and subject_id
    pub async fn get_materials_by_type_and_subject(
        &self,
        material_type: &crate::domain::subjects::subject_learning_material::SubjectMaterialType,
        subject_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, String> {
        self.repo
            .find_by_type_and_subject(material_type, subject_id)
            .await
            .map_err(|e| e.message)
    }

    /// Update a learning material by id
    pub async fn update_material(
        &self,
        id: &IdType,
        updated_data: UpdateSubjectLearningMaterial,
    ) -> Result<SubjectLearningMaterial, String> {
        // Fetch existing material
        let material = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Learning material not found".to_string())?;

        // ✅ Validate link if being updated
        if let Some(ref link) = updated_data.link {
            if !link.trim().is_empty() && !self.is_valid_url(link) {
                return Err("Invalid URL format for material link".to_string());
            }
        }

        // ✅ Validate material type specific rules if type is being updated
        if let Some(ref material_type) = updated_data.material_type {
            // Clone the existing material first to avoid partially moving fields out of `material`
            let existing = material.clone();

            let validation_material = SubjectLearningMaterial {
                material_type: material_type.clone(),
                title: updated_data
                    .title
                    .clone()
                    .unwrap_or_else(|| existing.title.clone()),
                link: updated_data.link.clone().or_else(|| existing.link.clone()),
                ..existing
            };

            self.validate_material_type(&validation_material)?
        }

        let updated_material = self
            .repo
            .update_material(id, updated_data)
            .await
            .map_err(|e| e.message)?;
        Ok(updated_material)
    }

    /// Delete a learning material by id
    pub async fn delete_material(&self, id: &IdType) -> Result<(), String> {
        self.repo.delete_material(id).await.map_err(|e| e.message)
    }

    /// Toggle material active status
    pub async fn toggle_material_status(
        &self,
        id: &IdType,
        is_active: bool,
    ) -> Result<SubjectLearningMaterial, String> {
        let update_data = UpdateSubjectLearningMaterial {
            is_active: Some(is_active),
            ..Default::default()
        };

        self.update_material(id, update_data).await
    }

    /// Bulk get materials by subject_ids
    pub async fn get_materials_by_subject_ids(
        &self,
        subject_ids: &[ObjectId],
    ) -> Result<Vec<SubjectLearningMaterial>, String> {
        self.repo
            .find_by_subject_ids(subject_ids)
            .await
            .map_err(|e| e.message)
    }

    /// Validate URL format
    fn is_valid_url(&self, url: &str) -> bool {
        // Basic URL validation - you can enhance this with a proper URL parser if needed
        url.starts_with("http://") || url.starts_with("https://") || url.starts_with("/")
    }

    /// Validate material type specific rules
    fn validate_material_type(&self, material: &SubjectLearningMaterial) -> Result<(), String> {
        match material.material_type {
            crate::domain::subjects::subject_learning_material::SubjectMaterialType::ExternalLink => {
                if material.link.is_none() || material.link.as_ref().unwrap().trim().is_empty() {
                    return Err("External link materials require a valid URL".to_string());
                }
            }
            crate::domain::subjects::subject_learning_material::SubjectMaterialType::Video => {
                if material.link.is_none() || material.link.as_ref().unwrap().trim().is_empty() {
                    return Err("Video materials require a link to the video content".to_string());
                }
            }
            crate::domain::subjects::subject_learning_material::SubjectMaterialType::Document => {
                if material.link.is_none() || material.link.as_ref().unwrap().trim().is_empty() {
                    return Err("Document materials require a link to the document".to_string());
                }
            }
            _ => {
                // Book, Article, Note types don't require links
            }
        }

        Ok(())
    }

    /// Get materials count by subject_id
    pub async fn get_materials_count_by_subject_id(
        &self,
        subject_id: &IdType,
    ) -> Result<usize, String> {
        let materials = self.get_materials_by_subject_id(subject_id).await?;
        Ok(materials.len())
    }

    /// Get active materials count by subject_id
    pub async fn get_active_materials_count_by_subject_id(
        &self,
        subject_id: &IdType,
    ) -> Result<usize, String> {
        let materials = self.get_active_materials_by_subject_id(subject_id).await?;
        Ok(materials.len())
    }

    /// Search materials by title (case-insensitive)
    pub async fn search_materials_by_title(
        &self,
        subject_id: &IdType,
        search_term: &str,
    ) -> Result<Vec<SubjectLearningMaterial>, String> {
        let all_materials = self.get_materials_by_subject_id(subject_id).await?;

        let filtered_materials: Vec<SubjectLearningMaterial> = all_materials
            .into_iter()
            .filter(|material| {
                material
                    .title
                    .to_lowercase()
                    .contains(&search_term.to_lowercase())
            })
            .collect();

        Ok(filtered_materials)
    }
}

// Implement Default for UpdateSubjectLearningMaterial to make toggle_material_status easier
impl Default for UpdateSubjectLearningMaterial {
    fn default() -> Self {
        Self {
            material_type: None,
            title: None,
            link: None,
            description: None,
            role: None,
            is_active: None,
        }
    }
}
