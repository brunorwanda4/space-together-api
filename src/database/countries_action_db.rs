use mongodb::options::IndexOptions;
use mongodb::results::InsertOneResult;
use mongodb::IndexModel;
use mongodb::{bson::doc, Collection};

use crate::errors::{MyError, Result};
use crate::models::country_model::CountryModel;

#[derive(Debug, Clone)]
pub struct CountyActionDb {
    pub country: Collection<CountryModel>,
}

impl CountyActionDb {
    pub async fn create_country(&self, country: &CountryModel) -> Result<InsertOneResult> {
        let index_model = IndexModel::builder()
            .keys(doc! {"name" : 1})
            .options(IndexOptions::builder().unique(true).build())
            .build();

        self.country.create_index(index_model).await;

        let qry = self.country.insert_one(country).await;

        match qry {
            Ok(res) => Ok(res),
            Err(_) => Err(MyError::CanNotCreateCountry),
        }
    }
}
