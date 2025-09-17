use actix_web::web::{self, ServiceConfig};
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;
use std::time::Duration;
mod middleware;
mod models;
mod routes;
mod utils;
mod validate;
#[derive(Clone)]
pub struct AppConfig {
    pub account_key: String,
    pub project_id: String,
    pub front_end_point: String,
}
#[derive(Clone)]
struct AppState {
    config: AppConfig,
    // other fields...
}
impl AppConfig {
    pub fn from_secrets(secrets: &SecretStore) -> Self {
        Self {
            account_key: secrets
                .get("GOOGLE_SERVICE_ACCOUNT_KEY_BASE64")
                .expect("MY_API_KEY not found"),
            project_id: secrets
                .get("GOOGLE_PROJECT_ID")
                .expect("DATABASE_URL not found"),
            front_end_point: secrets
                .get("FRONT_END_ENDPOINT")
                .expect("DATABASE_URL not found"),
        }
    }
}
#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = AppConfig::from_secrets(&secrets);
    let state = web::Data::new(AppState { config });

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(
            web::scope("")
                .wrap(middleware::cors::init_cors(state.clone()))
                .wrap(middleware::rate_limit::init_rl())
                .wrap(middleware::timeout::TimeoutHandler::new(
                    Duration::from_secs(10),
                ))
                .wrap(middleware::helmet::SecurityHeaders)
                .wrap(actix_web::middleware::Compress::default())
                .service(routes::dictionary::search)
                .app_data(state),
        );
    };

    Ok(config.into())
}
