use crate::AppError;
use log::debug;
use rspotify::model::{Page, SearchResult};
use rspotify::{model::FullArtist, model::SearchType, prelude::*, ClientCredsSpotify, Credentials};

pub struct Spotify {
    pub client: ClientCredsSpotify,
}

impl Spotify {
    // Q: Since both args are strings, any way to shorten this?
    // Q: Is it OK to return an error from a constructor?
    pub async fn new(client_id: &str, client_secret: &str) -> Result<Self, AppError> {
        let creds = Credentials::new(client_id, client_secret);
        let client = ClientCredsSpotify::new(creds);

        // Obtaining the access token. Requires to be mutable because the internal
        // token will be modified. We don't need OAuth for this specific endpoint,
        // so `...` is used instead of `prompt_for_user_token`.
        client.request_token().await?;

        Ok(Spotify { client })
    }

    pub async fn get_artists(&self, artist_name: &str) -> Result<Vec<FullArtist>, AppError> {
        debug!("Looking up artist info on spotify for '{}'", artist_name);

        let search_result = self
            .client
            .search(artist_name, SearchType::Artist, None, None, Some(10), None)
            .await?;

        let artists = match search_result {
            SearchResult::Artists(artists) => artists,
            _ => {
                return Err(AppError::GenericError(
                    "Unexpected search result type does not contain artists".to_string(),
                ));
            }
        };

        Ok(self.filter_artists(artist_name, &artists))
    }

    /// Improve the results by reducing the number of bad matches
    fn filter_artists(&self, artist_name: &str, artists: &Page<FullArtist>) -> Vec<FullArtist> {
        let mut filtered_artists: Vec<FullArtist> = Vec::new();

        // Spotify returns artists in no particular order - we want to inspect
        // only the top artists.
        let mut artists = artists.items.clone();
        artists.sort_by(|a, b| b.followers.total.cmp(&a.followers.total));

        'main: for artist in artists {
            // Max 3 artists in response
            if filtered_artists.len() == 3 {
                break;
            }

            if artist.name.to_lowercase() == artist_name.to_lowercase() {
                // debug!("Found a perfect artist name match for '{}'", artist_name);
                filtered_artists.push(artist.clone());
                continue;
            }

            if artist.popularity < 10 {
                // debug!("Band '{}' is not popular enough", artist.name);
                continue;
            }

            if artist.followers.total < 1000 {
                // debug!("Band '{}' does not have enough followers", artist.name);
                continue;
            }

            if artist.genres.is_empty() {
                // debug!("Band '{}' does not have any genres", artist.name);
                continue;
            }

            for genre in artist.genres.clone() {
                if !genre.to_lowercase().contains("metal") {
                    // debug!("Band '{}' does not have the correct genre", artist.name);
                    continue 'main;
                }
            }

            filtered_artists.push(artist.clone());
        }

        filtered_artists
    }
}
