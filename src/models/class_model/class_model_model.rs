use std::str::FromStr;

use mongodb::bson::{self, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassModel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub username: Option<String>,
    pub class_teacher_id: Option<ObjectId>,
    pub trade_id: Option<ObjectId>,
    pub sector_id: Option<ObjectId>,
    pub code: Option<String>,
    pub class_type_id: Option<ObjectId>,
    pub class_room_id: Option<ObjectId>,
    pub is_public: Option<bool>,
    pub description: Option<String>,
    pub symbol_id: Option<ObjectId>,
    pub created_on: DateTime,
    pub updated_on: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassModelNew {
    pub name: String,
    pub username: Option<String>,
    pub class_teacher: Option<String>,
    pub trade: Option<String>,
    pub sector: Option<String>,
    pub code: Option<String>,
    pub class_type: Option<String>,
    pub class_room: Option<String>,
    pub is_public: Option<bool>,
    pub description: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassModelGet {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
    pub class_teacher: Option<String>,
    pub class_room: Option<String>,
    pub trade: Option<String>,
    pub sector: Option<String>,
    pub code: Option<String>,
    pub class_type: Option<String>,
    pub is_public: Option<bool>,
    pub description: Option<String>,
    pub created_on: String,
    pub updated_on: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClassModelPut {
    pub name: Option<String>,
    pub username: Option<String>,
    pub class_teacher: Option<String>,
    pub trade: Option<String>,
    pub sector: Option<String>,
    pub code: Option<String>,
    pub class_type: Option<String>,
    pub class_room: Option<String>,
    pub is_public: Option<bool>,
    pub description: Option<String>,
    pub symbol: Option<String>,
}

impl ClassModel {
    pub fn new(class_model_new: ClassModelNew) -> Self {
        ClassModel {
            id: None,
            name: class_model_new.name,
            username: class_model_new.username,
            class_teacher_id: class_model_new
                .class_teacher
                .and_then(|id| ObjectId::from_str(&id).ok()),
            class_room_id: class_model_new
                .class_room
                .and_then(|id| ObjectId::from_str(&id).ok()),
            trade_id: class_model_new
                .trade
                .and_then(|id| ObjectId::from_str(&id).ok()),
            sector_id: class_model_new
                .sector
                .and_then(|id| ObjectId::from_str(&id).ok()),
            code: class_model_new.code,
            class_type_id: class_model_new
                .class_type
                .and_then(|id| ObjectId::from_str(&id).ok()),
            is_public: class_model_new.is_public,
            symbol_id: class_model_new
                .symbol
                .map(|id| ObjectId::from_str(&id).unwrap()),
            description: class_model_new.description,
            created_on: DateTime::now(),
            updated_on: None,
        }
    }

    pub fn format(class: Self) -> ClassModelGet {
        ClassModelGet {
            id: class.id.map_or("".to_string(), |id| id.to_string()),
            name: class.name.clone(),
            username: class.username.clone(),
            description: class.description.clone(),
            class_teacher: class.class_teacher_id.map(|id| id.to_string()),
            class_room: class.class_room_id.map(|id| id.to_string()),
            trade: class.trade_id.map(|id| id.to_string()),
            sector: class.sector_id.map(|id| id.to_string()),
            class_type: class.class_type_id.map(|id| id.to_string()),
            is_public: class.is_public,
            code: class.code.clone(),
            symbol: class.symbol_id.map(|id| id.to_string()),
            created_on: class
                .created_on
                .try_to_rfc3339_string()
                .unwrap_or("".to_string()),
            updated_on: Some(class.updated_on.map_or("".to_string(), |date| {
                date.try_to_rfc3339_string().unwrap_or("".to_string())
            })),
        }
    }

    pub fn put(class_model_put: ClassModelPut) -> Document {
        let mut doc = Document::new();
        let mut is_updated = false;

        let mut insert_if_some = |key: &str, value: Option<bson::Bson>| {
            if let Some(v) = value {
                doc.insert(key, v);
                is_updated = true;
            }
        };

        insert_if_some("name", class_model_put.name.map(bson::Bson::String));
        insert_if_some("username", class_model_put.username.map(bson::Bson::String));
        insert_if_some(
            "class_teacher_id",
            class_model_put
                .class_teacher
                .and_then(|id| ObjectId::from_str(&id).ok())
                .map(bson::Bson::ObjectId),
        );
        insert_if_some(
            "class_room_id",
            class_model_put
                .class_room
                .and_then(|id| ObjectId::from_str(&id).ok())
                .map(bson::Bson::ObjectId),
        );
        insert_if_some(
            "trade_id",
            class_model_put
                .trade
                .and_then(|id| ObjectId::from_str(&id).ok())
                .map(bson::Bson::ObjectId),
        );
        insert_if_some(
            "sector_id",
            class_model_put
                .sector
                .and_then(|id| ObjectId::from_str(&id).ok())
                .map(bson::Bson::ObjectId),
        );
        insert_if_some("code", class_model_put.code.map(bson::Bson::String));
        insert_if_some(
            "class_type_id",
            class_model_put
                .class_type
                .and_then(|id| ObjectId::from_str(&id).ok())
                .map(bson::Bson::ObjectId),
        );
        insert_if_some(
            "is_public",
            class_model_put.is_public.map(bson::Bson::Boolean),
        );
        insert_if_some(
            "description",
            class_model_put.description.map(bson::Bson::String),
        );

        insert_if_some(
            "symbol_id",
            class_model_put
                .symbol
                .and_then(|id| ObjectId::from_str(&id).ok())
                .map(bson::Bson::ObjectId),
        );

        if is_updated {
            doc.insert("updated_on", bson::Bson::DateTime(DateTime::now()));
        }

        doc
    }
}
