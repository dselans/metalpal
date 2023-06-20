use crate::config::Release;
use log::info;
use prettytable::{Cell, Row, Table};

pub fn display(releases_today: &Vec<Release>) {
    if releases_today.is_empty() {
        crate::exit("No releases_today today".to_string());
    }

    let valid_releases = releases_today
        .into_iter()
        .filter(|r| !r.skip)
        .collect::<Vec<&Release>>();

    info!(
        "There are '{}' releases today; out of those, '{}' look interesting!\n",
        releases_today.len(),
        valid_releases.len(),
    );

    let mut sorted_releases = valid_releases.clone();

    sorted_releases.sort_by(|a, b| {
        b.spotify
            .clone()
            .unwrap()
            .followers
            .cmp(&a.spotify.clone().unwrap().followers)
    });

    let mut iter = 1;

    // Display release in tables, sorted by follower count
    for release in sorted_releases {
        let mut table = Table::new();

        // Header
        let mut header = format!("{}. {} - {}", iter, release.artist, release.album);

        if release.spotify.clone().unwrap().followers > 100_000 {
            header = "🔥 ".to_string() + header.as_str() + " 🔥";
        }

        table.set_titles(Row::new(vec![
            Cell::new(header.as_str()).style_spec("bFgcH2")
        ]));

        // unwrap()'s are fine because we non-skipped entries have both
        let spotify_metadata = release.spotify.clone().unwrap();
        let metallum_metadata = release.metallum.clone().unwrap();

        table.add_row(Row::new(vec![
            Cell::new("Metallum URL"),
            Cell::new(metallum_metadata.url.as_str()),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Metallum Genre"),
            Cell::new(metallum_metadata.genre.as_str()),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Metallum Origin Country"),
            Cell::new(metallum_metadata.country_origin.as_str()),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Spotify Genres"),
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
