use actix_governor::PeerIpKeyExtractor;
use base64::{Engine, engine::general_purpose};
use dotenv::dotenv;
use gcp_bigquery_client::model::table_row::TableRow;
use gcp_bigquery_client::model::table_schema::TableSchema;
use gcp_bigquery_client::{Client, model::query_request::QueryRequest};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::env;
use std::error::Error;
use tracing::field;
use yup_oauth2::ServiceAccountKey;
pub struct BigQueryWrapper {
    client: Client,
    project_id: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Field {
    name: Option<String>,
    fields: Option<Vec<Field>>,
    #[serde(rename = "type")]
    r#type: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Row {
    #[serde(default, deserialize_with = "deserialize_v")]
    v: Option<VType>,
    f: Option<Vec<Row>>,
    name: Option<String>,
}

#[derive(Debug, Clone)]
enum VType {
    Str(String),
    Arr(Vec<Row>),
    Object(Box<Row>),
}
fn deserialize_v<'de, D>(deserializer: D) -> Result<Option<VType>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let result: Result<Value, _> = Deserialize::deserialize(deserializer);
    println!("{:?}", result);
    match result {
        Ok(value) => match value {
            Value::String(s) => Ok(Some(VType::Str(s))),
            Value::Array(arr) => {
                let items = serde_json::from_value(Value::Array(arr));
                match items {
                    Ok(v) => Ok(Some(VType::Arr(v))),
                    Err(_) => Ok(None),
                }
            }
            Value::Object(ob) => {
                let obj = serde_json::from_value(Value::Object(ob));
                match obj {
                    Ok(v) => Ok(Some(VType::Object(v))),
                    Err(_) => Ok(None),
                }
            }
            _ => Ok(None),
        },
        Err(error) => Ok(None),
    }
}
impl Serialize for Row {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        println!("Serializer {:?}", self);
        if let Some(name) = &self.name {
            match &self.v {
                Some(VType::Str(str_v)) => {
                    map.serialize_entry(name.as_str(), str_v)?;
                }
                Some(VType::Arr(arr_v)) => {
                    map.serialize_entry(name.as_str(), arr_v)?;
                }
                Some(VType::Object(obj_v)) => {
                    map.serialize_entry(name.as_str(), obj_v)?;
                }
                None => {
                    map.serialize_entry(name.as_str(), &"".to_string())?;
                }
                _ => (),
            }
        } else {
            match &self.v {
                Some(VType::Str(str_v)) => {
                    map.serialize_entry("value", str_v)?;
                }
                Some(VType::Object(obj_v)) => {
                    map.serialize_entry("value", obj_v)?;
                }
                _ => (),
            }
        }
        if let Some(f) = &self.f {
            map.serialize_entry("field", f)?;
        }
        map.end()
    }
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

    pub fn map_to_schema2(&self, row: &mut Row, field: &Field) -> Result<(), Box<dyn Error>> {
        if let Some(val_fields) = &mut row.f
            && let Some(f_field) = &field.fields
            && let None = &field.r#type
        {
            for (r, f) in val_fields.iter_mut().zip(f_field.iter()) {
                _ = self.map_to_schema2(r, f);
            }
        }
        if let Some(VType::Arr(val_record)) = &mut row.v
            && let Some(f_field) = &field.fields
            && &Some(String::from("RECORD")) == &field.r#type
        {
            for (r) in val_record.iter_mut() {
                if let Some(VType::Object(each_record)) = &mut r.v
                    && let Some(record_field) = &mut each_record.f
                {
                    for (field_ele_arr, f) in record_field.iter_mut().zip(f_field.iter()) {
                        _ = self.map_to_schema2(field_ele_arr, f);
                    }
                }
            }
        }
        row.name = field.name.clone();
        Ok(())
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
                        let field_val = serde_json::to_value(scheme.clone())?;
                        println!("{:?}", field_val);
                        println!("{:?}", row_val);
                        let mut row_val_struct: Row = serde_json::from_value(row_val)?;
                        let field_val_struct: Field = serde_json::from_value(field_val)?;
                        println!("{:?}", row_val_struct);
                        println!("{:?}", field_val_struct);
                        let _ = self.map_to_schema2(&mut row_val_struct, &field_val_struct);
                        results.push(serde_json::to_value(row_val_struct)?);
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
