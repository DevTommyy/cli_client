use std::{
    fs::File,
    io::{Read, Write},
    process::Command,
};

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

// search for the path where to put the config
fn find_config() -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg("find ~/ -type d -name cli_client")
        .output()
        .expect("Failed to execute command");

    let cli_client_dir =
        String::from_utf8(output.stdout).expect("Invalid UTF-8 for the path of the config file");

    let cli_client_dir = cli_client_dir.trim();

    let mut config_path = cli_client_dir.trim().to_string();
    config_path.push_str("/rsm-conf.json");

    config_path
}

lazy_static::lazy_static! {
    static ref CONFIG_FILE: String = {
        find_config()
    };
}

#[derive(Deserialize, Clone, Default)]
pub struct Token(String);

impl Into<String> for Token {
    fn into(self) -> String {
        self.0
    }
}

impl From<String> for Token {
    fn from(value: String) -> Token {
        Token(value)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub key: Option<String>,
    pub token: Option<String>,
    pub first_run: bool,
}

impl Config {
    pub fn get_config() -> Result<Config> {
        read_file().map_err(|e| {
            log::error!("Error in reading the file {e}");
            Error::InvalidConfig
        })
    }

    pub fn update_config(&self) -> Result<()> {
        write_config(
            &*CONFIG_FILE,
            self.key.as_deref(),
            self.first_run,
            self.token.as_deref(),
        )
        .map_err(|e| {
            log::error!("Error in updating file {e}");
            Error::FailedToUpdateConf
        })
    }

    pub fn load_token() -> Result<Token> {
        let mut file = File::open(&*CONFIG_FILE).map_err(|_| Error::InvalidConfig)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|_| Error::FailedToReadConfig)?;

        let data: Config = serde_json::from_str(&contents).map_err(|_| Error::InvalidConfig)?;
        let token: Token = Token::from(data.token.ok_or(Error::NoAuth)?);
        Ok(token)
    }
}

fn read_file() -> std::io::Result<Config> {
    if !file_exists_or_empty(&*CONFIG_FILE)? {
        write_config(&*CONFIG_FILE, None, true, None)?;
    }

    let mut file = File::open(&*CONFIG_FILE)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let data: Config = serde_json::from_str(&contents)?;
    Ok(data)
}

fn file_exists_or_empty(file_path: &str) -> std::io::Result<bool> {
    if let Ok(metadata) = std::fs::metadata(file_path) {
        if metadata.len() == 0 {
            return Ok(false);
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

fn write_config(
    file_path: &str,
    key: Option<&str>,
    first_run: bool,
    token: Option<&str>,
) -> std::io::Result<()> {
    let default_json = json!({
        "key": key,
        "first_run": first_run,
        "token": token
    });

    let json_string = serde_json::to_string_pretty(&default_json)?;

    let mut file = File::create(&file_path)?;
    file.write_all(json_string.as_bytes())?;
    Ok(())
}
