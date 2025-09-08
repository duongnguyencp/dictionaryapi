use base64::{Engine, engine::general_purpose};
use dotenv::dotenv;
use gcp_bigquery_client::Client;
use serde_json::Value;
use std::env;

pub struct BigQueryWrapper {
    client: Client,
    project_id: String,
}

impl BigQueryWrapper {:w
    /// Initialize wrapper by reading base64 encoded key from env and project ID
    pub async fn new() -> Result<Self, String> {
        dotenv().ok();

        let encoded_key = env::var("GOOGLE_SERVICE_ACCOUNT_KEY_BASE64").map_or_else(
            |e| "Missing GOOGLE_SERVICE_ACCOUNT_KEY_BASE64 env var",
            |v| &v,
        );
        let decoded_key = general_purpose::STANDARD.decode(&encoded_key).unwrap();

        let project_id =
            env::var("GOOGLE_PROJECT_ID").map_or_else(|e| "Fail to get project id", |v| v);

        let client = Client::from_service_account_key(String::from_utf8(decoded_key), true)
            .await
            .ok();
        if let None = client {
            Err("Fail authentication".to_string())
        } else {
            Ok(Self { client, project_id })
        }
    }

    /// Run a SQL query, return each row as serde_json::Value
    pub async fn query(&self, sql: &str) -> Result<Vec<Value>> {
        let request = QueryRequest::new(sql.to_string());

        let mut result_set = self.client.job().query(&self.project_id, request).await?;

        let mut results = Vec::new();

        while result_set.next_row() {
            let row_json = result_set.row_as_json()?;
            results.push(row_json);
        }

        Ok(results)
    }
}
