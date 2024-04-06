use chrono::NaiveDateTime;
use reqwest::{blocking, header};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use std::io::Read;

use crate::error::{Error, Result};
use crate::utils::config_helper::{Config, Token};

const BACKEND: &str = "http://100.97.63.15:10001";

pub struct Api {
    token: Token,
}

#[derive(Debug, Deserialize, Serialize)]
struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Deserialize, Serialize)]
struct ErrorDetail {
    req_uuid: String,
    #[serde(rename = "type")]
    error_type: String,
}

#[derive(Deserialize, Serialize)]
pub struct TableCharacteristicsResponse {
    pub res: Vec<TableCharacteristicsResponseDetails>,
}

#[derive(Deserialize, Serialize)]
pub struct TableCharacteristicsResponseDetails {
    pub has_due: bool,
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct GetTaskResponse {
    pub res: Vec<GetTaskResponseDetail>,
}

#[derive(Deserialize, Serialize)]
#[skip_serializing_none]
pub struct GetTaskResponseDetail {
    description: String,
    group: String,
    due: Option<NaiveDateTime>,
}

impl Api {
    pub fn new() -> Result<Api> {
        let token = Config::load_token()?;
        Ok(Api { token })
    }

    //TODO: return gettaskresponse
    pub fn get_tasks(&self, tablename: Option<&str>, opts: HashMap<&str, &str>) -> Result<()> {
        let client = blocking::Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|_| Error::FailedToConnectToServer)?;

        let token: String = self.token.clone().into();

        let mut url = format!("{}/{}", BACKEND, tablename.unwrap_or("list"));

        if !opts.is_empty() {
            let mut encoded_params = String::new();
            for (key, value) in opts.iter() {
                let encoded_key = urlencoding::encode(key);
                let encoded_value = urlencoding::encode(value);
                encoded_params.push_str(&format!("{}={}&", encoded_key, encoded_value));
            }
            // remove trailing '&'
            encoded_params.pop();
            url.push_str(&format!("?{}", encoded_params));
        }

        let mut response = client
            .get(url)
            .header(header::COOKIE, token)
            .send()
            .map_err(|_| Error::FailedToConnectToServer)?;

        let mut body = String::new();
        response
            .read_to_string(&mut body)
            .map_err(|_| Error::InvalidServerResponse)?;

        // TODO: move this into the formatter
        let pretty_res = if body.contains("error") {
            let json_response: ErrorResponse =
                serde_json::from_str(&body).map_err(|_| Error::FailedtoReadServerResponse)?;
            serde_json::to_string_pretty(&json_response)
                .map_err(|_| Error::FailedtoReadServerResponse)?
        } else {
            if tablename.is_some() {
                let json_response: GetTaskResponse =
                    serde_json::from_str(&body).map_err(|_| Error::FailedtoReadServerResponse)?;
                serde_json::to_string_pretty(&json_response)
                    .map_err(|_| Error::FailedtoReadServerResponse)?
            } else {
                let json_response: TableCharacteristicsResponse =
                    serde_json::from_str(&body).map_err(|_| Error::FailedtoReadServerResponse)?;
                serde_json::to_string_pretty(&json_response)
                    .map_err(|_| Error::FailedtoReadServerResponse)?
            }
        };

        println!("{}", pretty_res);
        Ok(())
    }
}
