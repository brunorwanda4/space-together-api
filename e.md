Got it ✅ — your `subject_learning_material_service.rs` is already very complete.
I’ll update and refine it to:

- Add `Default` for `UpdateSubjectLearningMaterial` properly.
- Improve **URL validation** using `url::Url` (safer than manual string check).
- Ensure **timestamps update** (`updated_at`) when modifying materials.
- Make error messages consistent and user-friendly.
- Slight cleanup for readability.

Here’s the updated version:

```rust
use crate::{
    domain::subjects::subject_learning_material::{
        SubjectLearningMaterial, SubjectMaterialType, UpdateSubjectLearningMaterial,
    },
    models::id_model::IdType,
    repositories::subjects::subject_learning_material_repo::SubjectLearningMaterialRepo,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use url::Url;

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
        if new_material.title.trim().is_empty() {
            return Err("❌ Material title is required".to_string());
        }

        if let Some(ref link) = new_material.link {
            if !link.trim().is_empty() && !self.is_valid_url(link) {
                return Err("❌ Invalid URL format for material link".to_string());
            }
        }

        if let Err(validation_error) = self.validate_material_type(&new_material) {
            return Err(validation_error);
        }

        let now = Some(Utc::now());
        new_material.created_at = now;
        new_material.updated_at = now;
        new_material.is_active = true;
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
            .ok_or_else(|| "❌ Learning material not found".to_string())
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
        material_type: &SubjectMaterialType,
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
        mut updated_data: UpdateSubjectLearningMaterial,
    ) -> Result<SubjectLearningMaterial, String> {
        let material = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| e.message.clone())?
            .ok_or_else(|| "❌ Learning material not found".to_string())?;

        if let Some(ref link) = updated_data.link {
            if !link.trim().is_empty() && !self.is_valid_url(link) {
                return Err("❌ Invalid URL format for material link".to_string());
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

        // Always update `updated_at`
        updated_data.updated_at = Some(Utc::now());

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
            updated_at: Some(Utc::now()),
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
        Url::parse(url).is_ok() || url.starts_with('/')
    }

    /// Validate material type specific rules
    fn validate_material_type(&self, material: &SubjectLearningMaterial) -> Result<(), String> {
        match material.material_type {
            SubjectMaterialType::ExternalLink => {
                if material.link.as_ref().map_or(true, |l| l.trim().is_empty()) {
                    return Err("❌ External link materials require a valid URL".to_string());
                }
            }
            SubjectMaterialType::Video => {
                if material.link.as_ref().map_or(true, |l| l.trim().is_empty()) {
                    return Err("❌ Video materials require a link to the video content".to_string());
                }
            }
            SubjectMaterialType::Document => {
                if material.link.as_ref().map_or(true, |l| l.trim().is_empty()) {
                    return Err("❌ Document materials require a link to the document".to_string());
                }
            }
            _ => {} // Book, Article, Note: no special link requirement
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

        Ok(all_materials
            .into_iter()
            .filter(|material| {
                material
                    .title
                    .to_lowercase()
                    .contains(&search_term.to_lowercase())
            })
            .collect())
    }
}

// ✅ Implement Default for UpdateSubjectLearningMaterial
impl Default for UpdateSubjectLearningMaterial {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            link: None,
            material_type: None,
            is_active: None,
            updated_at: None,
        }
    }
}
```

---

Do you also want me to **add a method for pagination** (so you don’t load all materials at once when there are thousands), or should I keep it as-is for now?
