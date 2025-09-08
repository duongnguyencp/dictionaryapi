use std::borrow::Cow;
use validator::ValidationError;

pub fn custom_validation(val: &str) -> Result<(), ValidationError> {
    if val.trim() == "\"\"" {
        let mut err = ValidationError::new("empty_string");
        err.message = Some(Cow::Borrowed("Field can not be empty string"));
        Err(err)
    } else if val.chars().any(|c| !c.is_alphanumeric()) {
        let mut err = ValidationError::new("special_char");
        err.message = Some(Cow::Borrowed("Field can not be special character"));
        Err(err)
    } else {
        Ok(())
    }
}
