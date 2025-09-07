use std::fmt::Display;

use actix_web::{Error, FromRequest, HttpRequest, dev::Payload, web::Query};

use crate::validate::format_validators::ValidationErrorFormatter;

use futures_util::{FutureExt, future::LocalBoxFuture};
use serde::de::DeserializeOwned;
use validator::Validate;
pub struct ValidateQuery<T>(pub T);
impl<T> FromRequest for ValidateQuery<T>
where
    T: DeserializeOwned + Validate + 'static + Display,
{
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = Query::<T>::from_request(req, payload);
        async move {
            match fut.await {
                Ok(query) => {
                    let inner = query.0;
                    match inner.validate() {
                        Ok(_) => Ok(ValidateQuery(inner)),
                        Err(error) => Err(Error::from(error.format_errors())),
                    }
                }
                Err(error) => Err(error),
            }
        }
        .boxed_local()
    }
}
