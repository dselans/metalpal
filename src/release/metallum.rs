use crate::config::{MetallumAristInfo, MetallumSearchResponse};
use crate::AppError;
use log::{debug, error, info};
use regex::Regex;
use reqwest::Client;

const SEARCH_URL: &str = "https://www.metal-archives.com/search/ajax-band-search";

pub async fn get_artists(artist_name: &str) -> Result<Vec<MetallumAristInfo>, AppError> {
    let client = reqwest::Client::builder().user_agent("metalpal").build()?;

    let mut artists: Vec<MetallumAristInfo> = Vec::new();

    let request = client
        .get(SEARCH_URL)
        .query(&[("field", "name"), ("query", artist_name)]);

    let response: MetallumSearchResponse = request.send().await?.json().await?;

    if response.aa_data.is_empty() {
        info!("No artists found in metallum for artist {}", artist_name);
        return Ok(artists); // This should probably be &
    }

    // At least one artist found - use it
    for artist in response.aa_data {
        // artist.0 == html with URL to artist
        // artist.1 == genre
        // artist.2 == country

        debug!(
            "Found potential match '{}' on Metallum for artist '{}'",
            artist.0, artist_name
        );

        let mut artist_url = "".to_string();

        if let Some(url) = get_artist_url(&artist.0) {
            info!("Found artist URL: {}", url);
            artist_url = url.clone();
        } else {
            error!("Could not determine artist URL in '{}'", artist.0);
            continue;
        }

        info!(
            "Figured out URL for artist '{}': {}",
            artist_name, artist_url
        );

        // match get_artist_info(&client, &artist.0).await {
        //     Ok(artist_info) => {
        //         artists.push(artist_info);
        //         continue;
        //     }
        //     Err(e) => {
        //         error!(
        //             "Could not fetch artist info for artist '{}': {:?}",
        //             artist.0, e
        //         );
        //
        //         continue;
        //     }
        // }
    }

    // Return whatever we managed to create
    Ok(artists)
}

async fn get_artist_info(client: &Client, artist_url: &str) -> Result<MetallumAristInfo, AppError> {
    info!("Looking up artist url: {}", artist_url);
    Err(AppError::GenericError("Not implemented".to_string()))
}

fn get_artist_url(html: &str) -> Option<String> {
    let end = html.find("\">")?;

    // Beginning of line is <a href=\"
    Some(html[9..end].to_string())
}
