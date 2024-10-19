use crate::{
    error::school::trading_error::{TradingErr, TradingResult},
    models::school::trading_model::{TradingModel, TradingModelNew, TradingModelUpdate},
};
use mongodb::{
    bson::{doc, oid::ObjectId, Bson},
    options::IndexOptions,
    results::InsertOneResult,
    Collection, IndexModel,
};
use std::str::FromStr;

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

    // update a trading
    pub async fn update_trading(
        &self,
        id: String,
        trading: TradingModelUpdate,
    ) -> TradingResult<String> {
        // Convert the id from String to ObjectId
        let obj_id =
            ObjectId::from_str(&id).map_err(|_| TradingErr::CanChangeTradingIdIntoObjectId)?;

        // Build the $set update document dynamically
        let mut update_doc = doc! {};

        if let Some(name) = trading.name {
            update_doc.insert("name", name);
        }
        if let Some(username) = trading.username {
            update_doc.insert("username", username);
        }
        if let Some(code) = trading.code {
            update_doc.insert("code", code);
        }
        if let Some(description) = trading.description {
            update_doc.insert("description", description);
        }
        if let Some(trading_type) = trading.trading_type {
            // Convert trading_type to String and insert it
            update_doc.insert("trading_type", Bson::String(trading_type.to_string()));
        }
        if let Some(schools_id) = trading.schools_id {
            update_doc.insert("schools_id", schools_id);
        }
        if let Some(education) = trading.education {
            update_doc.insert("education", Bson::String(education.to_string()));
        }

        // Only proceed with the update if we have something to update
        if update_doc.is_empty() {
            return Err(TradingErr::NoFieldsToUpdate);
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
}
