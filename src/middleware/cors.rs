use actix_cors::Cors;
use actix_web::http;
use dotenv::dotenv;
use std::env;
pub fn init_cors() -> Cors {
    dotenv().ok();
    let fe_endpoint_rs = env::var("FRONT_END_ENDPOINT");
    match fe_endpoint_rs {
        Ok(fe_endpoint) => Cors::default()
            .allowed_origin(&fe_endpoint)
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600),

        Err(error) => {
            println!("{}", error);
            return Cors::default();
        }
    }
}
