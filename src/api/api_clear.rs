/// # Api Module: Table Operations
///
/// This module provides functionality for table operations such as clearing a table.
///
/// ## Struct
///
/// - `Api`: Implementation of the API structure.
///
/// ## Methods
///
/// - `clear_table`: Method to clear a table.
use std::io::Read;

use reqwest::{blocking, header};

use crate::api::{ErrorResponse, SuccessfulResponse};
use crate::error::{Error, Result};
use crate::utils::table_formatter::FormattedResponse;

use super::{Api, BACKEND};

impl Api {
    pub fn clear_table(&self, tablename: String) -> Result<Box<dyn FormattedResponse>> {
        let client = blocking::Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|_| Error::FailedToConnectToServer)?;

        let tablename = match tablename {
            x if ["reminder", "todo"].contains(&x.as_str()) => x.to_owned(),

            name => format!("user/{}", name),
        };
        let token: String = self.token.clone().unwrap_or_default().into();
        let url = format!("{}/{}/clear", BACKEND, tablename);

        let mut response = client
            .delete(url)
            .header(header::COOKIE, token)
            .send()
            .map_err(|_| Error::FailedToConnectToServer)?;

        let mut body = String::new();
        response
            .read_to_string(&mut body)
            .map_err(|_| Error::InvalidServerResponse)?;

        let json_response_obj: Box<dyn FormattedResponse> = if body.contains("error") {
            let err_response: ErrorResponse =
                serde_json::from_str(&body).map_err(|_| Error::FailedtoReadServerResponse)?;
            Box::new(err_response)
        } else {
            let task_response: SuccessfulResponse =
                serde_json::from_str(&body).map_err(|_| Error::FailedtoReadServerResponse)?;
            Box::new(task_response)
        };

        Ok(json_response_obj)
    }
}
