/// # Api Module: User Operations
///
/// This module provides functionality for user operations such as signup, login, logout, and recovering lost keys.
///
/// ## Struct
///
/// - `Api`: Implementation of the API structure.
///
/// ## Methods
///
/// - `post_signup`: Method to sign up a new user.
/// - `post_login`: Method to log in a user.
/// - `post_logout`: Method to log out a user.
/// - `post_lostkey`: Method to recover a lost key for a user.
use std::io::Read;

use chrono::{DateTime, Utc};
use reqwest::{blocking, header};
use serde_json::json;

use super::{Api, ErrorResponse, SuccessfulResponse, BACKEND};
use crate::{
    error::{Error, Result},
    utils::table_formatter::FormattedResponse,
};

impl Api {
    // -- singup region
    pub fn post_signup(&self, usr: &str, pwd: &str) -> Result<Box<dyn FormattedResponse>> {
        let client = blocking::Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|_| Error::FailedToConnectToServer)?;

        let token: String = self.token.clone().unwrap_or_default().into();
        let url = format!("{}/signup", BACKEND);
        let payload = json!({
            "username": usr.trim(),
            "password": pwd.trim(),
        })
        .to_string();

        let mut response = client
            .post(url)
            .header(header::COOKIE, token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(payload)
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
            let success_response: SuccessfulResponse =
                serde_json::from_str(&body).map_err(|_| Error::FailedtoReadServerResponse)?;
            Box::new(success_response)
        };

        Ok(json_response_obj)
    }
    // -- end singup region

    // -- login region
    pub fn post_login(&self, key: &str) -> Result<(Box<dyn FormattedResponse>, String)> {
        let client = blocking::Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|_| Error::FailedToConnectToServer)?;

        let token: String = self.token.clone().unwrap_or_default().into();
        let url = format!("{}/login", BACKEND);
        let payload = json!({
            "key": key.trim(),
        })
        .to_string();

        let mut response = client
            .post(url)
            .header(header::COOKIE, token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(payload)
            .send()
            .map_err(|_| Error::FailedToConnectToServer)?;

        let token = response
            .cookies()
            .into_iter()
            .map(|cookie| {
                // tries to retrive the exp date in it cant it retrives the mag_age one and
                // calculates it
                let expires_string = match cookie.expires() {
                    Some(expires) => {
                        let datetime: DateTime<Utc> = expires.into();
                        datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
                    }
                    None => match cookie.max_age() {
                        Some(duration) => {
                            let datetime = Utc::now() + duration;
                            datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
                        }
                        None => String::from(""),
                    },
                };

                format!(
                    "{}={}; Path={}; HttpOnly; Expires={}",
                    cookie.name(),
                    cookie.value(),
                    cookie.path().unwrap_or("/"),
                    expires_string,
                )
            })
            .collect::<Vec<String>>()
            .join("; ");

        let mut body = String::new();

        response
            .read_to_string(&mut body)
            .map_err(|_| Error::InvalidServerResponse)?;

        let json_response_obj: Box<dyn FormattedResponse> = if body.contains("error") {
            let err_response: ErrorResponse =
                serde_json::from_str(&body).map_err(|_| Error::FailedtoReadServerResponse)?;
            Box::new(err_response)
        } else {
            let success_response: SuccessfulResponse =
                serde_json::from_str(&body).map_err(|_| Error::FailedtoReadServerResponse)?;
            Box::new(success_response)
        };

        Ok((json_response_obj, token))
    }
    // -- end login region

    // -- logout region
    pub fn post_logout(&self, logout: bool) -> Result<Box<dyn FormattedResponse>> {
        let client = blocking::Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|_| Error::FailedToConnectToServer)?;

        let token: String = self.token.clone().unwrap_or_default().into();
        let url = format!("{}/logout", BACKEND);
        let payload = json!({
            "logout": logout
        })
        .to_string();

        let mut response = client
            .post(url)
            .header(header::COOKIE, token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(payload)
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
            let success_response: SuccessfulResponse =
                serde_json::from_str(&body).map_err(|_| Error::FailedtoReadServerResponse)?;
            Box::new(success_response)
        };

        Ok(json_response_obj)
    }
    // -- end logout region

    // -- lostkey region
    pub fn post_lostkey(&self, usr: &str, pwd: &str) -> Result<Box<dyn FormattedResponse>> {
        let client = blocking::Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|_| Error::FailedToConnectToServer)?;

        let token: String = self.token.clone().unwrap_or_default().into();
        let url = format!("{}/lostkey", BACKEND);
        let payload = json!({
            "username": usr.trim(),
            "password": pwd.trim(),
        })
        .to_string();

        let mut response = client
            .post(url)
            .header(header::COOKIE, token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(payload)
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
            let success_response: SuccessfulResponse =
                serde_json::from_str(&body).map_err(|_| Error::FailedtoReadServerResponse)?;
            Box::new(success_response)
        };

        Ok(json_response_obj)
    }
    // -- end lostkey region
}
