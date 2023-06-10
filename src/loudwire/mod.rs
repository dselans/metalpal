use chrono::prelude::*;
use scraper::{Html, Selector};
use regex::Regex;

const LOUDWIRE_URL: &str = "https://loudwire.com/2023-hard-rock-metal-album-release-calendar/";

#[derive(Debug)]
pub struct Release {
    pub date: NaiveDate,
    pub artist: String,
    pub album: String,
    pub label: String,
}

pub fn get_releases(date_start: &DateTime<Local>, date_end: &DateTime<Local>) -> Result<Vec<Release>, String> {
    // fetch releases from loudwire
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
        None => return Result::Err(String::from("Could not parse date")),
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
