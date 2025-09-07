use crate::models::word::WordEntry;
use crate::validate::validator::ValidateQuery;
use actix_web::{HttpResponse, Responder, get};
use core::fmt;
use serde::Deserialize;
use std::borrow::Cow;
use std::fmt::Display;
use std::fs;
use validator::{Validate, ValidationError};

fn not_blank(val: &str) -> Result<(), ValidationError> {
    if val.trim() == "\"\"" {
        let mut err = ValidationError::new("empty_string");
        err.message = Some(Cow::Borrowed("Field can not be empty string"));
        Err(err)
    } else {
        Ok(())
    }
}
#[derive(Deserialize, Validate)]
struct InputData {
    #[validate(
        required(message = "Search query is required"),
        length(min = 1, message = "Field can not be empty string"),
        custom = "not_blank"
    )]
    value: Option<String>,
}
impl Display for InputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}])", self.value.clone().unwrap_or_default())
    }
}
#[get("/search")]
pub async fn search(query: ValidateQuery<InputData>) -> impl Responder {
    let word = query.0;
    let path: String = format!("words/{}.json", word);
    match fs::read_to_string(&path) {
        Ok(data) => {
            let entry: WordEntry = serde_json::from_str(&data).unwrap();
            HttpResponse::Ok().json(entry)
        }
        Err(_) => HttpResponse::NotFound().body("Word not found"),
    }
}
