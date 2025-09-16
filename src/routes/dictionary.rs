use crate::validate::func_validation::custom_validation;
use crate::validate::validator::ValidateQuery;
use crate::{models::error::AppError, utils::bigquery::BigQueryWrapper};
use actix_web::{App, Error, HttpResponse, Responder, get};
use core::fmt;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Value, json};
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
#[derive(Debug, Serialize, Deserialize)]
struct Meaning {
    part_of_speech: String,
    definition: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DictionaryEntry {
    source_url: String,
    word: String,
    phonetic: String,
    meanings: Vec<Meaning>,
}
#[get("/search")]
pub async fn search(query: ValidateQuery<InputData>) -> Result<impl Responder, Error> {
    let word = query.0;
    let query_literal: String = format!(
        "SELECT  word,audio_save, source_url, phonetic, meanings 
                FROM `dictionary-project-471510.dictionary.dictionary` where word  = '{}' LIMIT 1000",
        word.value.unwrap_or_default()
    );
    let connect_bq = BigQueryWrapper::new().await;
    match connect_bq {
        Ok(connection) => {
            let query = connection.query(&query_literal).await;
            match query {
                Ok(data) => {
                    if let Some(data_first) = data.first() {
                        Ok(HttpResponse::Ok().json(data))
                    } else {
                        Err(Error::from(AppError::NotFound))
                    }
                }
                Err(error) => {
                    println!("{:?}", error);
                    Err(Error::from(AppError::Internal))
                }
            }
        }
        Err(_) => Err(Error::from(AppError::Internal)),
    }
}
