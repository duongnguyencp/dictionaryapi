use std::fmt::{self};

use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
#[derive(Debug, Serialize)]
pub enum AppError {
    BadRequest,
    Internal,
    NotFound,
    Timeout,
}
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::BadRequest => write!(f, "Invalid Input provided"),
            AppError::Internal => write!(f, "Internal Server Error"),
            AppError::NotFound => write!(f, "Not found"),
            AppError::Timeout => write!(f, "Request timeout"),
        }
    }
}
impl ResponseError for AppError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let body = serde_json::json!({
            "error": self.to_string()
        });
        match self {
            AppError::BadRequest => HttpResponse::BadRequest().json(body),
            AppError::NotFound => HttpResponse::NotFound().json(body),
            AppError::Internal => HttpResponse::InternalServerError().json(body),
            AppError::Timeout => HttpResponse::GatewayTimeout().json(body),
        }
    }
}
