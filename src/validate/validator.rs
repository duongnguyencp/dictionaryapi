use std::collections::HashMap;

use actix_web::{FromRequest, HttpRequest, dev::Payload, web::Query};

use crate::validate::format_validators::{ErrorResponse, ValidationErrorFormatter};

use futures_util::{FutureExt, future::LocalBoxFuture};
use serde::de::DeserializeOwned;
use validator::Validate;
pub struct ValidateQuery<T>(pub T);
impl<T> FromRequest for ValidateQuery<T>
where
    T: DeserializeOwned + Validate + 'static,
{
    type Error = ErrorResponse;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = Query::<T>::from_request(req, payload);
        async move {
            match fut.await {
                Ok(query) => {
                    let inner = query.0;
                    match inner.validate() {
                        Ok(_) => Ok(ValidateQuery(inner)),
                        Err(error) => Err(error.format_errors()),
                    }
                }
                Err(_) => Err(ErrorResponse {
                    errors: HashMap::new(),
                }),
            }
        }
        .boxed_local()
    }
}
