use base64::{Engine, engine::general_purpose};
use dotenv::dotenv;
use gcp_bigquery_client::model::table_row::TableRow;
use gcp_bigquery_client::model::table_schema::TableSchema;
use gcp_bigquery_client::{Client, model::query_request::QueryRequest};
use serde_json::{Value, json};
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

    pub fn map_to_schema2(
        &self,
        row: &mut Value,
        columns: &mut Value,
    ) -> Result<(), Box<dyn Error>> {
        let mut stack = vec![(columns, row)];
        while !stack.is_empty() {
            let node = stack.pop().ok_or("Error")?;
            let left = node.0;
            let right = node.1;
            match left {
                Value::Object(_) => {
                    if let Some(values) = right["f"].as_array_mut()
                        && let Some(fields) = left["fields"].as_array_mut()
                    {
                        for ele in fields.iter_mut().zip(values.iter_mut()) {
                            stack.push(ele);
                        }
                    } else {
                        if let Some(name_field) = left.get_mut("name")
                            && let Some(val_key) =
                                right.get_mut(name_field.as_str().ok_or("Error")?)
                        {
                            val_key = right.get_mut("v").clone().ok_or("Error")?;
                        }
                    }
                }

                _ => {}
            }
        }
        Ok(())
    }

    pub fn map_to_schema(&self, row: &TableRow, schema: &TableSchema) -> Value {
        let mut value = json!({});
        if let Some(fields) = schema.fields.clone() {
            for (index, field) in fields.into_iter().enumerate() {
                if let Some(columns) = row.columns.clone() {
                    if let Some(cell) = columns.get(index) {
                        let value_cell = cell.value.clone().unwrap_or_default();
                        let mode = String::from("REPEATED");
                        if field.mode.unwrap_or_default().as_str() == "REPEATED" {
                            let list = value_cell
                                .as_array()
                                .unwrap_or(&Vec::<Value>::new())
                                .iter()
                                .map(|ele| {
                                    let temp = ele.clone();
                                    temp["v"].clone()
                                })
                                .collect();
                            value[field.name.clone()] = list;
                        } else {
                            value[field.name.clone()] = value_cell.clone();
                        }
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
                        let row_val = serde_json::to_value(row)?;
                        let field_val = serde_json::to_value(scheme.fields.clone())?;
                        let json_val =
                            self.map_to_schema2(&mut row_val.clone(), &mut field_val.to_owned())?;
                        results.push(json_val);
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
