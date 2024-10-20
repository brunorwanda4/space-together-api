use crate::{
    error::school::trading_error::{TradingErr, TradingResult},
    models::school::trading_model::{TradingModel, TradingModelNew, TradingModelUpdate},
};
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, DateTime},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};
use std::{result, str::FromStr};

#[derive(Debug, Clone)]
pub struct TradingActionDb {
    pub trading: Collection<TradingModel>,
}

impl TradingActionDb {
    // create a new trading
    pub async fn create_trading(&self, trading: TradingModelNew) -> TradingResult<InsertOneResult> {
        let index = IndexModel::builder()
            .keys(doc! {
                "username" : 1,
                "code" : 1
            })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        let one_index = self.trading.create_index(index).await;
        if one_index.is_err() {
            return Err(TradingErr::CanNotCreateTradingIndex);
        }

        let new = TradingModel::new(trading);
        let create = self.trading.insert_one(new).await;

        match create {
            Ok(result) => Ok(result),
            Err(_) => Err(TradingErr::CanNotCreateTrading),
        }
    }

    // get trading
    pub async fn get_trading_by_id(&self, id: String) -> TradingResult<TradingModel> {
        let obj_id = ObjectId::from_str(&id)
            .map_err(|_| TradingErr::CanChangeTradingIdIntoObjectId)
            .expect("Can not get trading object id");
        let get = self.trading.find_one(doc! {"_id" : obj_id}).await;

        match get {
            Ok(Some(trading)) => Ok(trading),
            Ok(None) => Err(TradingErr::NotFoundTrading),
            Err(_) => Err(TradingErr::CanNotGetTrading),
        }
    }
    // get all trading
    pub async fn get_all_tradings(&self) -> TradingResult<Vec<TradingModel>> {
        let mut cursor = self
            .trading
            .find(doc! {})
            .await
            .map_err(|_| TradingErr::CanNotGetAllTradings)?;

        let mut tradings: Vec<TradingModel> = Vec::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(trading) => tradings.push(trading),
                Err(_) => return Err(TradingErr::CanNotGetAllTradings),
            }
        }

        Ok(tradings)
    }

    // get a trading by username
    pub async fn get_trading_by_username(&self, username: String) -> TradingResult<TradingModel> {
        let get = self.trading.find_one(doc! {"username": username}).await;
        match get {
            Ok(Some(trading)) => Ok(trading),
            Ok(None) => Err(TradingErr::NotFoundTrading),
            Err(_) => Err(TradingErr::CanNotGetTrading),
        }
    }

    // get a trading by code
    pub async fn get_trading_by_code(&self, code: String) -> TradingResult<TradingModel> {
        let get = self.trading.find_one(doc! {"code": code}).await;
        match get {
            Ok(Some(trading)) => Ok(trading),
            Ok(None) => Err(TradingErr::NotFoundTrading),
            Err(_) => Err(TradingErr::CanNotGetTrading),
        }
    }

    // update a trading
    pub async fn update_trading(
        &self,
        id: String,
        trading: TradingModelUpdate,
    ) -> TradingResult<String> {
        let obj_id =
            ObjectId::from_str(&id).map_err(|_| TradingErr::CanChangeTradingIdIntoObjectId)?;

        let mut update_doc = doc! {};
        let mut is_update = false;

        if let Some(name) = trading.name {
            update_doc.insert("name", name);
            is_update = true;
        }
        if let Some(reason) = trading.reasons {
            update_doc.insert("reasons", reason);
            is_update = true;
        }
        if let Some(username) = trading.username {
            update_doc.insert("username", username);
            is_update = true;
        }
        if let Some(code) = trading.code {
            update_doc.insert("code", code);
            is_update = true;
        }
        if let Some(description) = trading.description {
            update_doc.insert("description", description);
            is_update = true;
        }
        if let Some(trading_type) = trading.trading_type {
            update_doc.insert("trading_type", Bson::String(trading_type.to_string()));
            is_update = true;
        }
        if let Some(schools_id) = trading.schools_id {
            update_doc.insert("schools_id", schools_id);
            is_update = true;
        }
        if let Some(education) = trading.education {
            update_doc.insert("education", Bson::String(education.to_string()));
            is_update = true;
        }

        if let Some(is_active) = trading.is_active {
            update_doc.insert("is_active", is_active);
            is_update = true;
        }

        if update_doc.is_empty() {
            return Err(TradingErr::NoFieldsToUpdate);
        }

        if is_update {
            update_doc.insert("updated_at", DateTime::now());
        }

        let update = self
            .trading
            .find_one_and_update(doc! {"_id" : obj_id}, doc! {"$set" : update_doc})
            .await;

        match update {
            Ok(Some(_)) => Ok(id),
            Ok(None) => Err(TradingErr::CanNotGetTrading),
            Err(_) => Err(TradingErr::CanNotUpdateTrading),
        }
    }

    // Add school_id to the schools_id array if it doesn't already exist
    pub async fn add_school_in_trading_using_id(
        &self,
        id: String,
        school_id: String,
    ) -> TradingResult<String> {
        let obj_id =
            ObjectId::from_str(&id).map_err(|_| TradingErr::CanChangeTradingIdIntoObjectId)?;

        let update = self
            .trading
            .update_one(
                doc! {"_id": obj_id},
                doc! {"$addToSet": {"schools_id": school_id}},
            )
            .await;

        match update {
            Ok(result) if result.matched_count > 0 => Ok(id),
            Ok(_) => Err(TradingErr::NotFoundTrading),
            Err(_) => Err(TradingErr::CanNotUpdateTrading),
        }
    }

    // Remove school_id from the schools_id array
    pub async fn remove_school_in_trading_using_id(
        &self,
        id: String,
        school_id: String,
    ) -> TradingResult<String> {
        let obj_id =
            ObjectId::from_str(&id).map_err(|_| TradingErr::CanChangeTradingIdIntoObjectId)?;

        let update = self
            .trading
            .update_one(
                doc! {"_id": obj_id},
                doc! {"$pull": {"schools_id": school_id}},
            )
            .await;

        match update {
            Ok(result) if result.matched_count > 0 => Ok(id),
            Ok(_) => Err(TradingErr::NotFoundTrading),
            Err(_) => Err(TradingErr::CanNotUpdateTrading),
        }
    }

    pub async fn delete_trading(&self, id: String) -> TradingResult<TradingModel> {
        let obj_id = ObjectId::from_str(&id)
            .map_err(|_| TradingErr::CanChangeTradingIdIntoObjectId)
            .expect(" Can not change trading is into an object");

        let req = self
            .trading
            .find_one_and_delete(doc! {"_id" : obj_id})
            .await;

        match req {
            Ok(Some(trading)) => Ok(trading),
            Ok(None) => Err(TradingErr::NotFoundTrading),
            Err(_) => Err(TradingErr::CanNotDeleteTrading),
        }
    }
}
