mod loudwire;

use std::ops::{Sub};
use chrono::Duration;
use chrono::prelude::*;

fn main() {
    // V1
    //
    // 1. Check loudwire for today’s releases
    // 	1. For each release, try to get name of artist
    // 2. Check generated list against `reported` list
    // 	1. Each reported entry as `date:sha:name` in file/pickle
    // 	2. If band is already reported -> remove from list
    // 3. Go through cleaned up list, lookup if artist is in metallum
    // 	1. If yes, check if part of black listed genres
    // 		1. If not blacklisted, add to list of new releases, include metadata about band
    // 	2. If no, skip
    // 4. Go through remaining list, determine artist popularity via Spotify
    // 	1. Top 5 tracks >= 1,000,000 == high
    // 	2. >= 100,000 == medium
    // 	3. >= 10,000 == low
    // 	4. <10,000 == niche
    // 	5. MAYBE: Include link to listen to artist on Spotify?
    // 5. Sort by top popularity
    // 6. Report list via Slack / Discord / etc.
    // 7. Save reported data to file/pickle

    // V2
    //
    // All of the above, but don't spawn from main() - set it up as a forever
    // running service

    let date_start: DateTime<Local> = Local::now();
    let date_end = date_start.sub(Duration::days(1));

    let result = loudwire::get_releases(&date_start, &date_end);

    let releases = match result {
        Ok(releases) => releases,
        Err(e) => { fatal_error(e) }
    };

    for release in &releases {
        println!("Date: {} Artist: {} Album: {} Label: {}", release.date, release.artist, release.album, release.label);
    }

    println!("Found {} releases", releases.len());

    // let mut releases_result = loudwire::get_releases(`$DATE`);
    // match releases_result {
    //     Ok(releases) => {
    //         println!("Found {} releases", releases.len());
    //
    //     },
    //     Err(e) => {
    //         println!("Error: {}", e);
    //     }
    // }
    //
    // // Filter known releases
    // let filter_result = filter_known_releases(releases);
    // match filter_result {
    //     Ok(num_filtered) => {
    //         println!("Filtered {} releases", filtered_releases.len());
    //     },
    //     Err(e) => {
    //         println!("Error: {}", e);
    //     }
    // }
    //
    // // Filter bands that are blacklist
    // let _ = filter_blacklisted_bands(filtered_releases);
    //
    // // Enrich metadata via metallum
    // let mut filtered_result = metallum::enrich_metadata(filtered_releases);
    // match enriched_result {
    //     Ok(enriched_releases) => {
    //         println!("Enriched {} releases", enriched_releases.len());
    //     },
    //     Err(e) => {
    //         println!("Error: {}", e);
    //     }
    // }
    //
    // // Filter blacklisted genres
    // let _ = filter_blacklisted_genres(enriched_releases);
    //
    // // Filter by popularity
    // let spotify_result = spotify::get_popularity(enriched_releases);
    // match spotify_result {
    //     Ok(popular_releases) => {
    //         println!("Found {} popular releases", popular_releases.len());
    //     },
    //     Err(e) => {
    //         println!("Error: {}", e);
    //     }
    // }
    //
    // // Report releases
    // let report_result = reporter::report(popular_releases);
    // match report_result {
    //     Ok(_) => {
    //         println!("Reported {} releases", popular_releases.len());
    //     },
    //     Err(e) => {
    //         println!("Error: {}", e);
    //     }
    // }
}

fn fatal_error(m: String) -> ! {
    println!("Fatal error: {}", m);
    std::process::exit(1);
}
