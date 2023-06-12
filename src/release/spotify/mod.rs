use rspotify::{
    model::{Country, Market, SearchType},
    prelude::*,
    ClientCredsSpotify, Credentials,
};

pub struct Spotify {
    pub client: ClientCredsSpotify,
}

pub struct Artist {
    pub name: String,
    pub url: String,
    pub country: String,
    pub years_active: String,
    pub genre: String,
    pub monthly_listeners: i64,
}

impl Spotify {
    // Q: Since both args are strings, any way to shorten this?
    // Q: Is it OK to return an error from a constructor?
    pub fn new(client_id: &str, client_secret: &str) -> Result<Self, String> {
        let creds = rspotify::Credentials::new(client_id, client_secret);
        let client = rspotify::ClientCredsSpotify::new(creds);

        // This will get a token and save it internally
        client.request_token(); // Q: Tried to map_err here but got compiler errors. Why?

        Ok(Spotify {
            client,
        })
    }

    pub fn find_artist(&self, artist_name: &str) -> Result<Vec<Artist>, String> {
        let result = self.client.search(
            artist_name,
            SearchType::Artist,
            Some(Market::Country(Country::UnitedStates)),
            None,
            Some(10),
            None,
        );
        let artists = match result {
            Ok(artists) => artists,
            Err(e) => return Err(e.to_string()),
        };

        for artist in artists {
            println!("Artist: {:?}", artist);
        }

        //
        let return_artists: Vec<Artist> = Vec::new();

        Ok(return_artists)
    }

    // pub fn get_artist(&self, artist: &str) -> Option<FullArtist> {
    //     let artists = self.client.search_artist(artist, 1, 0, None).unwrap();
    //     if artists.items.len() > 0 {
    //         Some(artists.items[0].clone())
    //     } else {
    //         None
    //     }
    // }
    //
    // pub fn get_album(&self, album: &str) -> Option<FullAlbum> {
    //     let albums = self.client.search_album(album, 1, 0, None).unwrap();
    //     if albums.items.len() > 0 {
    //         Some(albums.items[0].clone())
    //     } else {
    //         None
    //     }
    // }
    //
    // pub fn get_track(&self, track: &str) -> Option<FullTrack> {
    //     let tracks = self.client.search_track(track, 1, 0, None).unwrap();
    //     if tracks.items.len() > 0 {
    //         Some(tracks.items[0].clone())
    //     } else {
    //         None
    //     }
    // }
}
