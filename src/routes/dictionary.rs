use crate::validate::func_validation::custom_validation;
use crate::validate::validator::ValidateQuery;
use crate::{models::error::AppError, utils::bigquery::BigQueryWrapper};
use actix_web::{App, Error, HttpResponse, Responder, get};
use core::fmt;
use serde::Deserialize;
use std::fmt::Display;
use validator::Validate;

#[derive(Deserialize, Validate)]
struct InputData {
    #[validate(
        required(message = "Search query is required"),
        length(min = 1, message = "Field can not be empty string"),
        custom = "custom_validation"
    )]
    value: Option<String>,
}
impl Display for InputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}])", self.value.clone().unwrap_or_default())
    }
}
#[get("/search")]
pub async fn search(query: ValidateQuery<InputData>) -> Result<impl Responder, Error> {
    let word = query.0;
    let query_literal: String = format!(
        "SELECT *  FROM `dictionary-project-471510.dictionary.dictionary` where word  = '{}' LIMIT 1000",
        word.value.unwrap_or_default()
    );
    let connect_bq = BigQueryWrapper::new().await;
    match connect_bq {
        Ok(connection) => {
            let query = connection.query(&query_literal).await;
            match query {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(_) => Err(Error::from(AppError::Internal)),
            }
        }
        Err(_) => Err(Error::from(AppError::Internal)),
    }
}
