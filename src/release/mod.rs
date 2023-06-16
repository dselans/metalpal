mod spotify;

use crate::config::{Config, Release, SpotifyMetadata};
use crate::release::spotify::Spotify;
use crate::AppError;
use chrono::prelude::{Local, NaiveDate, Utc};
use log::debug;
use regex::Regex;
use scraper::{Html, Selector};

const LOUDWIRE_URL: &str = "https://loudwire.com/2023-hard-rock-metal-album-release-calendar/";

// Fetches latest releases from release
pub async fn fetch_releases() -> Result<Vec<Release>, AppError> {
    let resp = reqwest::get(LOUDWIRE_URL).await?;

    if resp.status() != reqwest::StatusCode::OK {
        return Err(AppError::GenericError {
            0: format!("Received non-200 status code: {}", resp.status()),
        });
    }

    let body = resp.text().await?;

    // Parse the document
    let fragment = Html::parse_document(&body);

    let closer = Selector::parse("div.pod-content > p")?;

    let mut releases = Vec::new();

    for entry in fragment.select(&closer) {
        if entry.html().starts_with("<p><strong>") {
            match parse_releases(entry.html()) {
                Ok(partial_releases) => {
                    for release in partial_releases {
                        releases.push(release);
                    }
                }
                Err(e) => {
                    // Only explode on issues unrelated to date parsing
                    if e.to_string().contains("Could not parse date") {
                        continue;
                    } else {
                        return Err(AppError::GenericError {
                            0: format!("Could not parse date: {}", e.to_string()),
                        });
                    }
                }
            };
        }
    }

    Ok(releases)
}

fn parse_releases(html: String) -> Result<Vec<Release>, AppError> {
    let mut releases = Vec::new();

    // Parse date
    let date_re = Regex::new(r"^<p><strong>(\w+ \d{1,2}, \d{4})</strong>")?;

    let date = match date_re.captures(&html) {
        Some(caps) => {
            let date_str = caps.get(1).map_or("", |m| m.as_str());
            NaiveDate::parse_from_str(date_str, "%B %d, %Y")?
        }
        None => {
            return Err(AppError::GenericError {
                0: format!("Could not parse date from {}", html),
            })
        }
    };

    // Parse releases
    let split_releases = html.split("<br>");

    for s in split_releases {
        // If regex doesn't match, move on to next entry
        let re = Regex::new(r"^(.+) - <em>(.+)</em>(?:\s+)?\(?(.+)\)(?:</p>)?$")?;
        let caps = match re.captures(s) {
            Some(caps) => caps,
            None => {
                // Regex didn't match
                continue;
            }
        };

        // Not sure if this is even possible
        if caps.len() != 4 {
            continue;
        }

        let artist = caps.get(1).map_or("", |m| m.as_str());
        let album = caps.get(2).map_or("", |m| m.as_str());
        let label = caps.get(3).map_or("", |m| m.as_str());

        let release = Release {
            date,
            artist: String::from(artist),
            album: String::from(album),
            label: label.replace("(", ""),
            spotify: None,
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
    let mut releases_today: Vec<Release> = Vec::new();

    for release in releases {
        if release.date == Local::now().date_naive() {
            // Q: I am creating a copy here; how can I return a slice of refs to existing releases?
            releases_today.push(release.clone());
        }
    }

    releases_today
}

// Q: I only want to return an error - is this the way to do it?
pub async fn enrich_with_spotify(
    client_id: String,
    client_secret: String,
    releases: &mut Vec<Release>,
) -> Result<(), AppError> {
    let spotify_client = Spotify::new(client_id.as_str(), client_secret.as_str()).await?;

    for release in releases {
        // Fetch release.spotify data here
        debug!("looking up artist info for: {}", release.artist);

        let spotify_artist_info = spotify_client.get_artists(release.artist.as_str()).await?;

        if spotify_artist_info.len() == 0 {
            debug!("No artist info found for {}", release.artist);
            continue;
        }

        if spotify_artist_info.len() == 1 {
            release.spotify = Some(SpotifyMetadata {
                id: spotify_artist_info[0].id.to_string(),
                url: spotify_artist_info[0].href.clone(),
                genres: spotify_artist_info[0].genres.clone(),
                popularity: i64::from(spotify_artist_info[0].popularity),
                followers: i64::from(spotify_artist_info[0].popularity),
            });

            continue;
        }
    }

    Ok(())
}
