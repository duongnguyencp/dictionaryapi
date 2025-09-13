use base64::{Engine, engine::general_purpose};
use dotenv::dotenv;
use gcp_bigquery_client::model::field_type::FieldType;
use gcp_bigquery_client::model::row::Row;
use gcp_bigquery_client::model::table::Table;
use gcp_bigquery_client::model::table_row::TableRow;
use gcp_bigquery_client::model::table_schema::TableSchema;
use gcp_bigquery_client::{Client, model::query_request::QueryRequest};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use yup_oauth2::ServiceAccountKey;
pub struct BigQueryWrapper {
    client: Client,
    project_id: String,
}

impl BigQueryWrapper {
    /// Initialize wrapper by reading base64 encoded key from env and project ID
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        dotenv().ok();

        let encoded_key = env::var("GOOGLE_SERVICE_ACCOUNT_KEY_BASE64")?;
        let decoded_key = general_purpose::STANDARD.decode(&encoded_key)?;
        let decoded_key_string: String = String::from_utf8(decoded_key)?;
        let service_account = serde_json::from_str::<ServiceAccountKey>(&decoded_key_string)?;

        let project_id = env::var("GOOGLE_PROJECT_ID")?.to_string();
        let client = Client::from_service_account_key(service_account, true).await?;
        Ok(Self { client, project_id })
    }

    pub fn map_to_schema(&self, row: &TableRow, schema: &TableSchema) -> Value {
        let mut value = json!({});
        if let Some(fields) = schema.fields.clone() {
            for (index, field) in fields.into_iter().enumerate() {
                if let Some(columns) = row.columns.clone() {
                    if let Some(cell) = columns.get(index) {
                        let value_cell = cell.value.clone().unwrap_or_default();
                        value[field.name.clone()] = value_cell.clone();
                    }
                }
            }
        }
        value
    }
    /// Run a SQL query, return each row as serde_json::Value
    pub async fn query(&self, sql: &str) -> Result<Vec<Value>, Box<dyn Error>> {
        let request = QueryRequest::new(sql.to_string());

        match self.client.job().query(&self.project_id, request).await {
            Ok(result_set) => {
                let mut results = Vec::<Value>::new();
                let scheme: TableSchema = result_set.schema.unwrap();
                if let Some(rows) = result_set.rows {
                    for row in rows {
                        let value = self.map_to_schema(&row, &scheme);
                        results.push(value);
                    }
                }

                Ok(results)
            }
            Err(error) => {
                println!("{}", error);
                return Err(Box::new(error));
            }
        }
    }
}
