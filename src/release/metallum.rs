use crate::config::{MetallumMetadata, MetallumSearchResponse};
use crate::AppError;
use log::info;

const SEARCH_URL: &str = "https://www.metal-archives.com/search/ajax-band-search";

pub async fn get_artists(artist_name: &str) -> Result<Vec<MetallumMetadata>, AppError> {
    let client = reqwest::Client::builder().user_agent("metalpal").build()?;

    let request = client
        .get(SEARCH_URL)
        .query(&[("field", "name"), ("query", artist_name)]);

    let response: MetallumSearchResponse = request.send().await?.json().await?;

    if response.aa_data.is_empty() {
        info!("No artists found in metallum for artist {}", artist_name);
        return Ok(Vec::new());
    }

    info!(
        "Found {} artists in metallum for artist {}",
        response.aa_data.len(),
        artist_name
    );

    for entry in response.aa_data {
        info!("Found artist: {:?}", entry.0);
    }

    Ok(Vec::new())
}
