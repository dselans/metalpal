mod spotify;

use scraper::{Html, Selector};
use regex::Regex;
use crate::config::{Config, Release};
use chrono::prelude::{Utc, NaiveDate, Local};

const LOUDWIRE_URL: &str = "https://loudwire.com/2023-hard-rock-metal-album-release-calendar/";

// Fetches latest releases from release
pub fn fetch_releases() -> Result<Vec<Release>, String> {
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

fn parse_releases(html: String) -> Result<Vec<Release>, String> {
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
            spotify: vec![], // Q: It seems like this _must_ be instantiated - is this OK?
            metallum: vec![]
        };

        releases.push(release);
    }

    releases.sort_by_key(|r| r.date);

    Ok(releases)
}

pub fn out_of_date(config: &Config) -> bool {
    let now = Utc::now();
    let last_update = config.last_update;

    // If last update was more than 24 hours ago, return true
    now.signed_duration_since(last_update).num_hours() > 24
}

// Q: Read somewhere that it's better to accept a slice than a vector? Should I do that?
pub fn get_releases_today(releases: &Vec<Release>) -> Vec<Release> {
    // Q: Should I only specify the type when the compiler can't infer it or should I always do it?
    let mut releases_today : Vec<Release> = Vec::new();

    for release in releases {
        if release.date == Local::now().date_naive() {
            // Q: I am creating a copy here; how can I return a slice of refs to existing releases?
            releases_today.push(release.clone());
        }
    }

    releases_today
}

// Q: I only want to return an error - is this the way to do it?
pub fn enrich_with_spotify(client_id: String, client_secret: String, releases: &mut Vec<Release>) -> Result<(), String> {
    let spotify_client = spotify::Spotify::new(client_id.as_str(), client_secret.as_str())?;

    for release in releases {
        // Fetch release.spotify data here
        let _ = spotify_client.find_artist(&release.artist);
    }

    Ok(())
}


// Wrapper for fetching latest releases for given date range
//
// First attempts to fetch data from local file; if that fails OR if it's too old
// re-fetch data from release.
// pub fn get_config() -> Result<MetalPalConfig, String> {
//     let config_result = fetch_config_from_file();
//
//     if let Ok(config) = config_result {
//         // We have found releases from file, return those instead of fetching from release
//         println!("Found config '{}'", CONFIG_FILE);
//         return Ok(config);
//     } else {
//         println!("Failed to find config file '{}'; fetching from release...", CONFIG_FILE);
//     }
//
//     // TODO: Check last_update
//
//     // File lookup failed, fetch releases from release instead
//     let releases = fetch_releases()?;
//     let config = MetalPalConfig {
//         full_path: full_path().unwrap(),
//         last_update: Utc::now(),
//         releases,
//     };
//
//     // Attempt to write releases to file
//     write_releases_to_file(&config)?;
//
//     Ok(config)
// }
