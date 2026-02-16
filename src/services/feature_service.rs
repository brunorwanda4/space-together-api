use mongodb::{bson::doc, Collection, Database};

use crate::{errors::AppError, models::id_model::IdType};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchoolFeatures {
    #[serde(rename = "_id")]
    pub school_id: mongodb::bson::oid::ObjectId,
    pub features: std::collections::HashMap<String, bool>,
}

pub struct FeatureService {
    pub collection: Collection<SchoolFeatures>,
}

impl FeatureService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<SchoolFeatures>("school_features"),
        }
    }

    pub async fn is_feature_enabled(
        &self,
        school_id: &IdType,
        feature_name: &str,
    ) -> Result<bool, AppError> {
        let school_oid = IdType::to_object_id(school_id)?;

        let features = self
            .collection
            .find_one(doc! { "_id": school_oid })
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
            })?;

        if let Some(features) = features {
            Ok(*features.features.get(feature_name).unwrap_or(&true))
        } else {
            Ok(true)
        }
    }

    pub async fn toggle_feature(
        &self,
        school_id: &IdType,
        feature_name: &str,
        enabled: bool,
    ) -> Result<(), AppError> {
        let school_oid = IdType::to_object_id(school_id)?;

        self.collection
            .update_one(
                doc! { "_id": school_oid },
                doc! {
                    "$set": {
                        format!("features.{}", feature_name): enabled
                    }
                },
            )
            .upsert(true)
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
            })?;

        Ok(())
    }
}
