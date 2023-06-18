use crate::config::Release;
use log::info;
use prettytable::{Cell, Row, Table};

pub fn display(releases_match: Vec<Release>, releases_today: Vec<Release>) {
    if releases_today.is_empty() {
        crate::exit("No releases_today today".to_string());
    }

    info!(
        "There are '{}' releases today; out of those, '{}' look interesting!",
        releases_today.len(),
        releases_match.len()
    );

    let mut releases_match = releases_match.clone();
    releases_match.sort_by(|a, b| {
        b.spotify
            .clone()
            .unwrap()
            .followers
            .cmp(&a.spotify.clone().unwrap().followers)
    });

    let mut iter = 1;

    // Display release in tables, sorted by follower count
    for release in releases_match {
        let mut table = Table::new();

        // Header
        let mut header = format!("{}. {} - {}", iter, release.artist, release.album);

        if release.spotify.clone().unwrap().followers > 100_000 {
            header = "🔥 ".to_string() + header.as_str() + " 🔥";
        }

        table.set_titles(Row::new(vec![
            Cell::new(header.as_str()).style_spec("bFgcH2")
        ]));

        let spotify_metadata = release.spotify.unwrap();

        table.add_row(Row::new(vec![
            Cell::new("Genres"),
            Cell::new(spotify_metadata.genres.join(", ").as_str()),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Spotify ID"),
            Cell::new(spotify_metadata.id.as_str()),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Spotify Popularity"),
            Cell::new(spotify_metadata.popularity.to_string().as_str()),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Spotify Followers"),
            Cell::new(spotify_metadata.followers.to_string().as_str()),
        ]));

        table.printstd();

        iter += 1;
    }
}
