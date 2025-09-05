use crate::models::word::WordEntry;
use actix_web::{HttpResponse, Responder, get};
use std::fs;

#[get("/search")]
pub async fn search(word: String) -> impl Responder {
    let path: String = format!("words/{}.json", word);
    match fs::read_to_string(&path) {
        Ok(data) => {
            let entry: WordEntry = serde_json::from_str(&data).unwrap();
            HttpResponse::Ok().json(entry)
        }
        Err(_) => HttpResponse::NotFound().body("Word not found"),
    }
}
