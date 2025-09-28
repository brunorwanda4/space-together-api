use crate::{
    domain::subjects::subject_learning_material::{
        SubjectLearningMaterial, SubjectLearningMaterialRole, SubjectMaterialType,
        UpdateSubjectLearningMaterial,
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

    /// Create a new learning material
    pub async fn create_material(
        &self,
        mut new_material: SubjectLearningMaterial,
    ) -> Result<SubjectLearningMaterial, String> {
        if new_material.title.trim().is_empty() {
            return Err("Material title is required".to_string());
        }

        if let Some(ref link) = new_material.link {
            if !link.trim().is_empty() && !self.is_valid_url(link) {
                return Err("Invalid URL format for material link".to_string());
            }
        }

        self.validate_material_type(&new_material)?;

        let now = Some(Utc::now());
        new_material.created_at = now;
        new_material.updated_at = now;
        new_material.is_active = true;
        new_material.id = Some(ObjectId::new());

        self.repo
            .insert_material(&new_material)
            .await
            .map_err(|e| e.message)
    }

    /// Get learning material by ID
    pub async fn get_material_by_id(&self, id: &IdType) -> Result<SubjectLearningMaterial, String> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Learning material not found".to_string())
    }

    /// Get materials by reference_id
    pub async fn get_by_reference_id(
        &self,
        reference_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, String> {
        self.repo
            .find_by_reference_id(reference_id)
            .await
            .map_err(|e| e.message)
    }

    /// Get materials by role + reference_id
    pub async fn get_by_role_and_reference(
        &self,
        role: &SubjectLearningMaterialRole,
        reference_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, String> {
        self.repo
            .find_by_role_and_reference(role, reference_id)
            .await
            .map_err(|e| e.message)
    }

    /// Get active materials by role + reference_id
    pub async fn get_active(
        &self,
        role: &SubjectLearningMaterialRole,
        reference_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, String> {
        self.repo
            .find_active(role, reference_id)
            .await
            .map_err(|e| e.message)
    }

    /// Get materials by type + role + reference_id
    pub async fn get_by_type_and_reference(
        &self,
        material_type: &SubjectMaterialType,
        role: &SubjectLearningMaterialRole,
        reference_id: &IdType,
    ) -> Result<Vec<SubjectLearningMaterial>, String> {
        self.repo
            .find_by_type_and_reference(material_type, role, reference_id)
            .await
            .map_err(|e| e.message)
    }

    /// Update a learning material by id
    pub async fn update_material(
        &self,
        id: &IdType,
        updated_data: UpdateSubjectLearningMaterial,
    ) -> Result<SubjectLearningMaterial, String> {
        let material = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "Learning material not found".to_string())?;

        if let Some(ref link) = updated_data.link {
            if !link.trim().is_empty() && !self.is_valid_url(link) {
                return Err("Invalid URL format for material link".to_string());
            }
        }

        if let Some(ref material_type) = updated_data.material_type {
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

        self.repo
            .update_material(id, updated_data)
            .await
            .map_err(|e| e.message)
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
            title: None,
            link: None,
            material_type: None,
            description: None,
            role: None,
            reference_id: None,
        };

        self.update_material(id, update_data).await
    }

    /// Validate URL format (basic)
    fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://") || url.starts_with("/")
    }

    /// Validate material type specific rules
    fn validate_material_type(&self, material: &SubjectLearningMaterial) -> Result<(), String> {
        match material.material_type {
            SubjectMaterialType::ExternalLink => {
                if material.link.as_ref().map_or(true, |l| l.trim().is_empty()) {
                    return Err("External link materials require a valid URL".to_string());
                }
            }
            SubjectMaterialType::Video => {
                if material.link.as_ref().map_or(true, |l| l.trim().is_empty()) {
                    return Err("Video materials require a link to the video content".to_string());
                }
            }
            SubjectMaterialType::Document => {
                if material.link.as_ref().map_or(true, |l| l.trim().is_empty()) {
                    return Err("Document materials require a link to the document".to_string());
                }
            }
            _ => {}
        }

        Ok(())
    }
}
