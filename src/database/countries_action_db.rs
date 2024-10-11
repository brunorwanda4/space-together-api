use mongodb::options::{FindOptions, IndexOptions};
use mongodb::results::InsertOneResult;
use mongodb::IndexModel;
use mongodb::{bson::{doc , oid::ObjectId}, Collection};
use serde::{Deserialize, Serialize};

use crate::models::country_model::CountryModel;
use crate::errors::{Result , MyError};

#[derive(Debug , Clone,)]
pub struct CountyActionDb {
    pub country: Collection<CountryModel>
}

impl CountyActionDb {
    pub async fn create_country (&self , country: &CountryModel) -> Result<InsertOneResult>{
        let index_model = IndexModel::builder()
            .keys(doc! {"name" : 1})
            .options(IndexOptions::builder().unique(true).build())
            .build();

        self.country.create_index(index_model).await;

        let qry = self
            .country
            .insert_one(country)
            .await;
        
        match qry {
            Ok(res) => Ok (res),
            Err(e) => {
                panic!("Error inserting country: {:?}" , e);
                Err(MyError::CanNotCreateCountry)
            }
        }
    }

}