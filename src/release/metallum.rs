use crate::config::{MetallumArtistInfo, MetallumSearchResponse};
use crate::AppError;
use log::{debug, error, info, warn};
use reqwest::Client;
use scraper::html::Select;
use scraper::{Html, Selector};
use std::thread::current;

const SEARCH_URL: &str = "https://www.metal-archives.com/search/ajax-band-search";

pub struct Metallum {
    pub client: Client,
}

impl Metallum {
    pub fn new() -> Result<Self, AppError> {
        // Q: What is a quick way to figure out what errors this can return?
        let client = reqwest::Client::builder().user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.113 Safari/537.36").build()?;

        Ok(Metallum { client })
    }

    pub async fn get_artists(
        &self,
        artist_name: &str,
    ) -> Result<Vec<MetallumArtistInfo>, AppError> {
        let mut artists: Vec<MetallumArtistInfo> = Vec::new();

        let request = self
            .client
            .get(SEARCH_URL)
            .query(&[("field", "name"), ("query", artist_name)]);

        let response: MetallumSearchResponse = request.send().await?.json().await?;

        if response.aa_data.is_empty() {
            debug!("No artists found in metallum for artist {}", artist_name);
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
                debug!("Found artist URL: {}", url);
                artist_url = url.clone();
            } else {
                error!("Could not determine artist URL in '{}'", artist.0);
                continue;
            }

            debug!(
                "Figured out URL for artist '{}': {}",
                artist_name, artist_url
            );

            // temporary
            let artist_url = "https://www.metal-archives.com/bands/Burzum/88";

            match self.get_artist_info(&artist_name, &artist_url).await {
                Ok(artist_info) => {
                    artists.push(artist_info);
                    break;
                }
                Err(e) => {
                    error!(
                        "Could not fetch artist info for artist '{}': {:?}",
                        artist.0, e
                    );

                    continue;
                }
            }
        }

        // Return whatever we managed to create
        Ok(artists)
    }

    async fn get_artist_info(
        &self,
        artist_name: &str,
        artist_url: &str,
    ) -> Result<MetallumArtistInfo, AppError> {
        warn!("Looking up artist url: {}", artist_url);

        let resp = reqwest::get(artist_url).await?;

        if resp.status() != reqwest::StatusCode::OK {
            return Err(AppError::GenericError {
                0: format!(
                    "Received non-200 status code from metallum: {}",
                    resp.status()
                ),
            });
        }

        let body = resp.text().await?;

        // Parse the document

        let document = Html::parse_document(&body);

        Ok(self.parse_band_info(&artist_name, &artist_url, &document)?)
    }

    pub fn parse_band_info(
        &self,
        artist_name: &str,
        artist_url: &str,
        document: &Html,
    ) -> Result<MetallumArtistInfo, AppError> {
        // Get band name
        let band_info_selector = Selector::parse("#band_info > #band_stats")?;

        let entry = document
            .select(&band_info_selector)
            .next()
            .ok_or(AppError::GenericError {
                0: "Could not find band info".to_string(),
            })?;

        let vector = entry
            .select(&Selector::parse("dl > dd")?)
            .map(|x| x.inner_html())
            .collect::<Vec<_>>();

        if vector.len() != 8 {
            return Err(AppError::GenericError {
                0: format!(
                    "Unexpected number of elements in band_info (expected 8, got {})",
                    vector.len()
                ),
            });
        }

        let country_origin = &vector[0];
        let location = &vector[1];
        let status = &vector[2];
        let formed_in = &vector[3];
        let genre = &vector[4];
        let lyrical_themes = &vector[5];
        let last_label = &vector[6];
        let years_active = &vector[7];

        // Origin is a link
        let country_origin = Html::parse_fragment(&country_origin)
            .select(&Selector::parse("a")?)
            .next()
            .unwrap()
            .inner_html();

        // Label is a link as well
        let last_label = Html::parse_fragment(&last_label)
            .select(&Selector::parse("a")?)
            .next()
            .unwrap()
            .inner_html();

        Ok(MetallumArtistInfo {
            name: artist_name.to_string(),
            country_origin: country_origin.to_string(),
            url: artist_url.to_string(),
            description_short: "".to_string(),
            locations: location.to_string(),
            status: status.to_string(),
            formed_in: formed_in.to_string(),
            genre: genre.to_string(),
            themes: lyrical_themes.to_string(),
            last_label: last_label.to_string(),
            years_active: years_active.to_string(),
            description_long: "".to_string(),
            img_url: "".to_string(),
        })
    }
}

fn get_artist_url(html: &str) -> Option<String> {
    let end = html.find("\">")?;

    // Beginning of line is <a href=\"
    Some(html[9..end].to_string())
}
