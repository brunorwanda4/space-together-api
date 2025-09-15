use crate::domain::sector::{Sector, UpdateSector};
use crate::errors::AppError;
use crate::models::id_model::IdType;

use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

pub struct SectorRepo {
    pub collection: Collection<Sector>,
}

impl SectorRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Sector>("sectors"),
        }
    }

    pub async fn find_by_id(&self, id: &IdType) -> Result<Option<Sector>, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let filter = doc! { "_id": obj_id };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find sector by id: {}", e),
            })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<Sector>, AppError> {
        let filter = doc! { "username": username };
        self.collection
            .find_one(filter)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to find sector by username: {}", e),
            })
    }

    pub async fn insert_sector(&self, sector: &Sector) -> Result<Sector, AppError> {
        // unique index on name + country maybe?
        // Unique index for username
        let username_index = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        // Non-unique index for name + country (for fast search/filtering)
        let name_country_index = IndexModel::builder()
            .keys(doc! { "name": 1, "country": 1 })
            .options(IndexOptions::builder().unique(false).build())
            .build();

        self.collection
            .create_index(username_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create username index: {}", e),
            })?;

        self.collection
            .create_index(name_country_index)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to create name+country index: {}", e),
            })?;

        let mut sector_to_insert = sector.clone();
        sector_to_insert.id = None;
        sector_to_insert.created_at = Some(Utc::now());
        sector_to_insert.updated_at = Some(Utc::now());

        let res = self
            .collection
            .insert_one(&sector_to_insert)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert sector: {}", e),
            })?;

        let inserted_id: ObjectId = res
            .inserted_id
            .as_object_id()
            .ok_or_else(|| AppError {
                message: "Failed to get inserted sector id".to_string(),
            })?
            .to_owned();

        self.find_by_id(&IdType::from_object_id(inserted_id))
            .await?
            .ok_or(AppError {
                message: "Sector not found after insert".to_string(),
            })
    }

    pub async fn get_all_sectors(&self) -> Result<Vec<Sector>, AppError> {
        let pipeline = vec![doc! { "$sort": { "updated_at": -1 } }];

        let mut cursor = self
            .collection
            .aggregate(pipeline)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch sectors: {}", e),
            })?;

        let mut sectors = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| AppError {
            message: format!("Failed to iterate sectors: {}", e),
        })? {
            let sector: Sector = bson::from_document(result).map_err(|e| AppError {
                message: format!("Failed to deserialize sector: {}", e),
            })?;
            sectors.push(sector);
        }

        Ok(sectors)
    }

    pub async fn update_sector(
        &self,
        id: &IdType,
        update: &UpdateSector,
    ) -> Result<Sector, AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let mut update_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Failed to convert update sector to document: {}", e),
        })?;

        // 🔥 Remove all `null` fields so they don’t overwrite existing values
        update_doc = update_doc
            .into_iter()
            .filter(|(_, v)| !matches!(v, bson::Bson::Null))
            .collect();

        update_doc.insert("updated_at", bson::to_bson(&Utc::now()).unwrap());
        update_doc.remove("_id"); // prevent overwriting id

        let update_doc = doc! { "$set": update_doc };

        self.collection
            .update_one(doc! { "_id": obj_id }, update_doc)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update sector: {}", e),
            })?;

        self.find_by_id(id).await?.ok_or(AppError {
            message: "Sector not found after update".to_string(),
        })
    }

    pub async fn delete_sector(&self, id: &IdType) -> Result<(), AppError> {
        let obj_id = ObjectId::parse_str(id.as_string()).map_err(|e| AppError {
            message: format!("Failed to parse id: {}", e),
        })?;

        let result = self
            .collection
            .delete_one(doc! { "_id": obj_id })
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete sector: {}", e),
            })?;

        if result.deleted_count == 0 {
            return Err(AppError {
                message: "No sector deleted; sector may not exist".to_string(),
            });
        }

        Ok(())
    }
}
