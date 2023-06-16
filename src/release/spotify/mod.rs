use crate::AppError;
use log::debug;
use rspotify::model::{Page, SearchResult};
use rspotify::{model::FullArtist, model::SearchType, prelude::*, ClientCredsSpotify, Credentials};

pub struct Spotify {
    pub client: ClientCredsSpotify,
}

// pub struct Artist {
//     pub name: String,
//     pub url: String,
//     pub country: String,
//     pub years_active: String,
//     pub genre: String,
//     pub monthly_listeners: i64,
// }

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

    fn filter_artists(&self, artist_name: &str, artists: &Page<FullArtist>) -> Vec<FullArtist> {
        let mut filtered_artists: Vec<FullArtist> = Vec::new();

        'main: for artist in artists.items.clone() {
            if filtered_artists.len() == 3 {
                break;
            }

            if artist.name.to_lowercase() == artist_name.to_lowercase() {
                debug!("Found a perfect artist name match for '{}'", artist_name);
                filtered_artists.push(artist.clone());
                continue;
            }

            if artist.popularity < 10 {
                debug!("Band '{}' is not popular enough", artist.name);
                continue;
            }

            if artist.followers.total < 1000 {
                debug!("Band '{}' does not have enough followers", artist.name);
                continue;
            }

            if artist.genres.is_empty() {
                debug!("Band '{}' does not have any genres", artist.name);
                continue;
            }

            for genre in artist.genres.clone() {
                if !genre.to_lowercase().contains("metal") {
                    debug!("Band '{}' does not have the correct genre", artist.name);
                    continue 'main;
                }
            }

            filtered_artists.push(artist.clone());
        }

        filtered_artists
    }
}
