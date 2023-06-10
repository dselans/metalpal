use chrono::prelude::{Utc, NaiveDate};
use scraper::{Html, Selector};
use regex::Regex;
use std::fs;
use serde::{Serialize, Deserialize};

const LOUDWIRE_URL: &str = "https://loudwire.com/2023-hard-rock-metal-album-release-calendar/";
const CONFIG_FILE: &str = ".metalpal.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct MetalPalConfig {
    pub full_path: String,
    pub last_update:chrono::DateTime<chrono::Utc>,
    pub releases: Vec<Release>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Release {
    pub date: NaiveDate,
    pub artist: String,
    pub album: String,
    pub label: String,
}

// Wrapper for fetching latest releases for given date range
//
// First attempts to fetch data from local file; if that fails OR if it's too old
// re-fetch data from release.
pub fn get_config() -> std::result::Result<MetalPalConfig, String> {
    let config_result = fetch_config_from_file();

    if let Ok(config) = config_result {
        // We have found releases from file, return those instead of fetching from release
        println!("Found config '{}'", CONFIG_FILE);
        return Ok(config);
    } else {
        println!("Failed to find config file '{}'; fetching from release...", CONFIG_FILE);
    }

    // TODO: Check last_update

    // File lookup failed, fetch releases from release instead
    let releases = fetch_releases_from_loudwire()?;
    let config = MetalPalConfig {
        full_path: full_path().unwrap(),
        last_update: Utc::now(),
        releases,
    };

    // Attempt to write releases to file
    write_releases_to_file(&config)?;

    Ok(config)
}

fn full_path() -> Result<String, String> {
    let home_dir_opt = home::home_dir();
    let home_dir = match home_dir_opt {
        Some(home_dir) => home_dir.display().to_string(),
        None => return Err(String::from("Failed to get home directory"))
    };

    Ok(home_dir.to_owned() + "/" + CONFIG_FILE)
}

fn fetch_config_from_file() -> std::result::Result<MetalPalConfig, String> {
    // Try to get homedir
    let full_path = full_path().unwrap();

    // Try to lookup file
    if !std::path::Path::new(&full_path).exists() {
        return Err(format!("File {} does not exist", full_path));
    }

    // Try to read + parse
    let contents = fs::read_to_string(full_path).map_err(|e| e.to_string())?;
    let config : MetalPalConfig = serde_json::from_str(contents.as_str()).map_err(|e| e.to_string())?; // How to avoid this map_err boilerplate?

    Ok(config)
}

fn write_releases_to_file(config: &MetalPalConfig) -> std::result::Result<(), String> {
    let json_str = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config.full_path, json_str).map_err(|e| e.to_string())?;

    Ok(())
}

// Fetches latest releases from release
fn fetch_releases_from_loudwire() -> std::result::Result<Vec<Release>, String> {
    let resp = reqwest::blocking::get(LOUDWIRE_URL).map_err(|e| e.to_string())?;

    if resp.status() != reqwest::StatusCode::OK {
        return Err(format!("Received non-200 status code {}", resp.status()));
    }

    let body = resp.text().map_err(|e| e.to_string())?;

    // Parse the document
    let fragment = Html::parse_document(&body);

    let closer = Selector::parse("div.pod-content > p").unwrap();

    let mut releases = Vec::new();

    for entry in fragment.select(&closer) {
        // let re = Regex::new(r"^<p>").map_err(|e| e.to_string())?;
        if entry.html().starts_with("<p><strong>") {
            match parse_releases(entry.html()) {
                Ok(partial_releases) => {
                    for release in partial_releases {
                        releases.push(release);
                    }
                },
                Err(e) => {
                    // Only explode on issues unrelated to date parsing
                    if e.contains("Could not parse date") {
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            };
        }
    }

    Ok(releases)
}

fn parse_releases(html: String) -> std::result::Result<Vec<Release>, String> {
    let mut releases = Vec::new();

    // Parse date
    let date_re = Regex::new(r"^<p><strong>(\w+ \d{1,2}, \d{4})</strong>").unwrap(); // can I unwrap or default?

    let date = match date_re.captures(&html) {
        Some(caps) => {
            let date_str = caps.get(1).map_or("", |m| m.as_str());
            NaiveDate::parse_from_str(date_str, "%B %d, %Y").unwrap()
        },
        None => return Err(String::from("Could not parse date")),
    };

    // Parse releases
    let split_releases = html.split("<br>");

    for s in split_releases {
        // If regex doesn't match, move on to next entry
        let re = Regex::new(r"^(.+) - <em>(.+)</em>(?:\s+)?\(?(.+)\)(?:</p>)?$").unwrap();
        let caps = match re.captures(s) {
            Some(caps) => caps,
            None => {
                // Regex didn't match
                continue;
            },
        };

        // Not sure if this is even possible
        if caps.len() != 4 {
            continue;
        }

        let artist = caps.get(1).map_or("", |m| m.as_str());
        let album = caps.get(2).map_or("", |m| m.as_str());
        let mut label = caps.get(3).map_or("", |m| m.as_str());

        let release = Release {
            date,
            artist: String::from(artist),
            album: String::from(album),
            label: label.replace("(", ""),
        };

        releases.push(release);
    }

    releases.sort_by_key(|r| r.date);

    Ok(releases)
}
