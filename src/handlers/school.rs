use axum::extract::State;
use axum::Json;
use mongodb::results::InsertOneResult;

use crate::libs::db::{self, Database , };
use crate::models::school::School;

use crate::errors::{Result , MyError};

pub async fn create_school (
    State(school) : State<Database>,
    Json(school_fc) : Json<School>,
) -> Result<Json<School>> {
    todo!()
}
