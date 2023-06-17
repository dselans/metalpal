use chrono::prelude::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::Write;
use std::ops::Sub;

use crate::error::AppError;

const CONFIG_FILE: &str = ".metalpal.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub full_path: String,
    pub last_update: chrono::DateTime<chrono::Utc>,
    pub releases: Vec<Release>,
    pub channels: Vec<String>,
    pub slack_bot_token: String,
    pub spotify_client_id: String,
    pub spotify_client_secret: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Release {
    pub date: NaiveDate,
    pub artist: String,
    pub album: String,
    pub label: String,
    pub spotify: Option<SpotifyMetadata>,
    // pub metallum: MetallumMetadata,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpotifyMetadata {
    pub id: String,
    pub url: String,
    pub genres: Vec<String>,
    pub popularity: i64,
    pub followers: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MetallumMetadata {
    pub url: String,
    pub country: String,
    pub years_active: String,
    pub genre: String,
}

pub fn full_path() -> Result<String, AppError> {
    let home_dir_opt = home::home_dir();
    let home_dir = match home_dir_opt {
        Some(home_dir) => home_dir.display().to_string(),
        None => {
            return Err(AppError::GenericError {
                0: "Could not find home directory".to_string(),
            })
        }
    };

    Ok(home_dir.to_owned() + "/" + CONFIG_FILE)
}

pub fn load_config() -> Result<Config, AppError> {
    // Try to get homedir
    let full_path = full_path()?;

    // Try to lookup file
    if !std::path::Path::new(&full_path).exists() {
        return Err(AppError::GenericError {
            0: format!("File '{}' does not exist", full_path.as_str()),
        });
    }

    // Try to read + parse
    let contents = fs::read_to_string(full_path)?;
    let config: Config = serde_json::from_str(contents.as_str())?;

    Ok(config)
}

// Interactive setup
pub fn setup_config() -> Result<Config, AppError> {
    // Q: There are no zero values (or nil/null) - what is the idiomatic way to instantiate a struct with default values?
    let mut config = Config {
        full_path: full_path()?,
        last_update: Utc::now().sub(chrono::Duration::days(100)), // Default to force update
        releases: Vec::new(),
        channels: vec![],
        slack_bot_token: "".to_string(),
        spotify_client_id: "".to_string(),
        spotify_client_secret: "".to_string(),
    };

    let slack_bot_token = ask_question("Slack bot token: ")?;
    let channels = ask_question("Slack channels (comma separated): ")?;
    let spotify_client_id = ask_question("Spotify client id: ")?;
    let spotify_client_secret = ask_question("Spotify client secret: ")?;

    config.slack_bot_token = slack_bot_token;
    config.channels = channels.split(",").map(|s| s.trim().to_string()).collect();
    config.spotify_client_id = spotify_client_id;
    config.spotify_client_secret = spotify_client_secret;

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), AppError> {
    let json_str = serde_json::to_string_pretty(&config)?;
    fs::write(&config.full_path, json_str).map_err(|e| AppError::GenericError {
        0: format!("Could not write config file '{}': {}", config.full_path, e),
    })?;

    Ok(())
}

fn ask_question(question: &str) -> Result<String, AppError> {
    loop {
        print!("{}", question);
        io::stdout().flush()?; // Need to do this to ensure print! shows immediate output

        let mut input = String::new();

        std::io::stdin().read_line(&mut input)?;

        if input.trim().is_empty() {
            continue;
        }

        return Ok(input.trim().to_string());
    }
}
