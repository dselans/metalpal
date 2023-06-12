mod release;
mod config;

use crate::config ::Config;

// Logic
//
// 1. Load cnfig or setup
//      -> If config does not exist, it's a first-run
//          -> build config for first time
//      -> If config exists, check last_update
//
// 2. If last_update is older than a week, re-fetch releases
//
// 3. Get releases that occur today
//
// 4. Enrich releases with release.spotify metadata
//      -> if release.spotify data already there, skip
//      -> release.spotify metadata: release.spotify artist URL, popularity
//
// 5. Enrich releases with metallum metadata
//     -> if metallum data already there, skip
//     -> metallum metadata: genre, country, year, metallum URL, description
//
// 6. Sort today's releases by popularity
//
// 7. Slack alert

fn main() {
    let mut config = match load_or_setup_config() {
        Ok(config) => config,
        Err(e) => fatal_error(e),
    };

    // Outdated releases?
    if release::out_of_date(&config) {
        match release::fetch_releases() {
            Ok(releases) => {
                println!("Fetched {} releases", releases.len()); // Q: Intellij marks releases.len() as an error: `usize` doesn't implement `Display` (required by {}); any way to fix?
                config.last_update = chrono::Utc::now();
                config.releases = releases
            }
            Err(e) => fatal_error(e),
        };
    } else {
        println!("Config is up to date; skipping fetch...");
    }

    // Save config to file
    if let Err(e) = config::save_config(&config) {
        // DS: I understand why there's no traditional string concat - is this the accepted/idiomatic way to do it?
        fatal_error(["Failed to save config: ", e.as_str()].join(""));
    }

    // Get today's releases
    let mut releases_today = release::get_releases_today(&config.releases);

    if releases_today.len() == 0 {
        exit(String::from("No releases today"));
    } else {
        println!("There are {} releases today!", releases_today.len()); // Q: Same intellij problem here
    }

    // Enrich today's releases with release.spotify metadata
    if let Err(e) = release::enrich_with_spotify(config.spotify_client_id, config.spotify_client_secret,&mut releases_today) {
        fatal_error(e);
    };

    // // Enrich today's releases with metallum metadata
    // let releases_today = match release::enrich_with_metallum(releases_today) {
    //     Ok(releases_today) => releases_today,
    //     Err(e) => fatal_error(e),
    // };

    // TODO: Sort by popularity, genre, etc.

    // TODO: Slack alert releases
}

// Q: Should I return a String for errors or my own custom error?
// My guess: implement Display trait on my custom type so it can be println!'d. Is this correct?
fn load_or_setup_config() -> Result<Config, String> {
    if let Ok(load_result) = config::load_config() {
        println!("Loaded existing config");
        return Ok(load_result);
    }

    println!("WARNING: Unable to find existing config; performing initial setup...");

    config::setup_config()
}

fn exit(m: String) -> ! {
    println!("{}", m);
    std::process::exit(0);
}

fn fatal_error(m: String) -> ! {
    println!("Fatal error: {}", m);
    std::process::exit(1);
}
