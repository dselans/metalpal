use chrono::prelude::NaiveDate;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::Write;

use crate::error::AppError;

const CONFIG_FILE: &str = ".metalpal.json";

// Q: This needs to be in main.rs for some reason, otherwise it panics; how can
// I move this into config.rs?
#[derive(Parser, Debug)]
pub struct CLI {
    /// Enable debug output
    #[arg(short, long, env = "METALPAL_DEBUG")]
    pub debug: bool,

    #[arg(
        long,
        env = "METALPAL_SPOTIFY_CLIENT_ID",
        default_value = "",
        required_unless_present = "interactive"
    )]
    pub spotify_client_id: String,

    #[arg(
        long,
        env = "METALPAL_SPOTIFY_CLIENT_SECRET",
        default_value = "",
        required_unless_present = "interactive"
    )]
    pub spotify_client_secret: String,

    #[arg(long, env = "METALPAL_SLACK_TOKEN", default_value = "")]
    pub slack_token: String,

    #[arg(long, env = "METALPAL_SLACK_CHANNELS", default_value = "")]
    pub slack_channels: Vec<String>,

    #[arg(long, env = "METALPAL_WHITELISTED_GENRE_KEYWORDS")]
    pub whitelisted_genre_keywords: Vec<String>,

    #[arg(long, env = "METALPAL_BLACKLISTED_GENRE_KEYWORDS")]
    pub blacklisted_genre_keywords: Vec<String>,

    #[arg(
        long,
        short,
        help = "Path to metalpal config file",
        default_value = ".metalpal.json"
    )]
    pub config_path: String,

    #[arg(long, short, help = "Run in interactive mode")]
    pub interactive: bool,

    #[arg(long, help = "Disable slack notifications")]
    pub disable_slack: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub full_path: String,
    pub last_update: chrono::DateTime<chrono::Utc>,
    pub releases: Vec<Release>,
    pub slack_channels: Vec<String>,
    pub slack_bot_token: String,
    pub spotify_client_id: String,
    pub spotify_client_secret: String,
    pub whitelisted_genre_keywords: Vec<String>,
    pub blacklisted_genre_keywords: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Release {
    pub date: NaiveDate,
    pub artist: String,
    pub album: String,
    pub label: String,
    #[serde(default)]
    pub skip: bool,
    pub skip_reasons: Vec<String>,
    pub spotify: Option<SpotifyArtistInfo>,
    pub metallum: Option<MetallumArtistInfo>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpotifyArtistInfo {
    pub id: String,
    pub url: String,
    pub genres: Vec<String>,
    pub popularity: i64,
    pub followers: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MetallumArtistInfo {
    pub name: String,
    pub url: String,
    pub description_short: String,
    pub description_long: String,
    pub country_origin: String,
    pub locations: String,
    pub years_active: String,
    pub formed_in: String,
    pub genre: String,
    pub themes: String,
    pub img_url: String,
    pub status: String,
    pub last_label: String,
}

type Genre = String;
type Country = String;
type Artist = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MetallumSearchResponse {
    pub aa_data: Vec<(Artist, Genre, Country)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            full_path: "".to_string(),
            last_update: Default::default(),
            releases: vec![],
            slack_channels: vec![],
            slack_bot_token: "".to_string(),
            spotify_client_id: "".to_string(),
            spotify_client_secret: "".to_string(),
            whitelisted_genre_keywords: vec![],
            blacklisted_genre_keywords: vec![],
        }
    }
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
pub fn setup_config(cli: &CLI) -> Result<Config, AppError> {
    if cli.interactive {
        setup_interactive()?;
    }

    return setup_cli(cli);
}

pub fn setup_cli(cli: &CLI) -> Result<Config, AppError> {
    Ok(Config {
        full_path: cli.config_path.clone(),
        last_update: Default::default(),
        releases: vec![],
        slack_channels: cli.slack_channels.clone(),
        slack_bot_token: cli.slack_token.clone(),
        spotify_client_id: cli.spotify_client_id.clone(),
        spotify_client_secret: cli.spotify_client_secret.clone(),
        whitelisted_genre_keywords: cli.whitelisted_genre_keywords.clone(),
        blacklisted_genre_keywords: cli.blacklisted_genre_keywords.clone(),
    })
}

pub fn setup_interactive() -> Result<Config, AppError> {
    // Q: There are no zero values (or nil/null) - what is the idiomatic way to instantiate a struct with default values?
    let mut config = Config::default();

    config.spotify_client_id = ask_question("Spotify client id (required): ", true)?;
    config.spotify_client_secret = ask_question("Spotify client secret (required): ", true)?;
    config.slack_bot_token =
        ask_question("Slack bot token (optional; leave blank to skip): ", false)?;
    config.slack_channels = ask_question_multi(
        "Slack channels (optional, comma separated; leave blank to skip): ",
        false,
    )?;
    config.whitelisted_genre_keywords = ask_question_multi(
        "Whitelisted genre keywords (optional, comma separated; leave blank to skip): ",
        false,
    )?;
    config.blacklisted_genre_keywords = ask_question_multi(
        "Blacklisted genre keywords (optional, comma separated; leave blank to skip): ",
        false,
    )?;

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), AppError> {
    let json_str = serde_json::to_string_pretty(&config)?;
    fs::write(&config.full_path, json_str).map_err(|e| AppError::GenericError {
        0: format!("Could not write config file '{}': {}", config.full_path, e),
    })?;

    Ok(())
}

fn ask_question_multi(prompt: &str, required: bool) -> Result<Vec<String>, AppError> {
    let answer = ask_question(prompt, required)?;

    // Q: Why do I need this? .split() or map seems to create 1 element if the string is empty
    if answer.is_empty() {
        return Ok(vec![]);
    }

    let answer = answer.replace(" ", "");
    let answer: Vec<String> = answer.split(",").map(str::to_string).collect();

    Ok(answer)
}

fn ask_question(prompt: &str, required: bool) -> Result<String, AppError> {
    loop {
        print!("{}", prompt);
        io::stdout().flush()?; // Need to do this to ensure print! shows immediate output

        let mut input = String::new();

        std::io::stdin().read_line(&mut input)?;

        // If required && press enter -> ask question again
        if required && input.trim().is_empty() {
            continue;
        }

        return Ok(input.trim().to_string());
    }
}
