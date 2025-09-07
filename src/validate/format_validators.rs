use actix_web::ResponseError;
use core::fmt;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use validator::ValidationErrors;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub errors: HashMap<String, Vec<String>>,
}
impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}])", "")
    }
}
impl ResponseError for ErrorResponse {}
pub trait ValidationErrorFormatter {
    fn format_errors(&self) -> ErrorResponse;
}
impl ValidationErrorFormatter for ValidationErrors {
    fn format_errors(&self) -> ErrorResponse {
        format_validation_errors(self)
    }
}
pub fn format_validation_errors(errors: &ValidationErrors) -> ErrorResponse {
    let mut formatted: HashMap<String, Vec<String>> = HashMap::new();

    for (field, field_errors) in errors.field_errors().iter() {
        let messages = field_errors
            .iter()
            .map(|e| {
                e.message
                    .clone()
                    .unwrap_or_else(|| Cow::Owned("Invalid value".to_string()))
                    .into_owned()
            })
            .collect();

        formatted.insert(field.to_string(), messages);
    }

    ErrorResponse { errors: formatted }
}
