use actix_cors::Cors;
use actix_web::{http, web};

use crate::AppState;
pub fn init_cors(state: web::Data<AppState>) -> Cors {
    let fe_endpoint_rs = state.config.front_end_point.clone();
    println!("{}", fe_endpoint_rs.clone());
    Cors::default()
        .allowed_origin(&fe_endpoint_rs)
        .allowed_methods(vec!["GET"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::CONTENT_TYPE,
        ])
        .supports_credentials()
        .max_age(3600)
}
