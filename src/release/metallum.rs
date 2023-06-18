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
        return Ok(Vec::new());
    }

    for entry in response.aa_data {
        info!("Found artist: {:?}", entry.0);
    }

    Ok(Vec::new())
}
