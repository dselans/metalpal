mod config;
mod error;
mod release;

use crate::config::Config;
use crate::error::AppError;
use clap::Parser;
use log::{debug, error, info};
use std::env;

#[derive(Parser)]
pub struct CLI {
    /// Enable debug output
    #[arg(short, long)]
    debug: bool,
}

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

#[tokio::main]
async fn main() {
    setup();

    let mut config = match load_or_setup_config() {
        Ok(config) => config,
        Err(e) => fatal_error(e.to_string()),
    };

    // Outdated releases?
    if release::out_of_date(&config) {
        match release::fetch_releases().await {
            Ok(releases) => {
                debug!("Fetched {} releases", releases.len()); // Q: Intellij marks releases.len() as an error: `usize` doesn't implement `Display` (required by {}); any way to fix?
                config.last_update = chrono::Utc::now();
                config.releases = releases
            }
            Err(e) => fatal_error(e.to_string()),
        };
    } else {
        debug!("Config is up to date; skipping fetch...");
    }

    // Save config to file
    if let Err(e) = config::save_config(&config) {
        // Q: My IDE can't tell that to_string exists - why not?
        fatal_error(e.to_string());
    }

    // Get today's releases
    let mut releases_today = release::get_releases_today(&config.releases);

    if releases_today.len() == 0 {
        exit(String::from("No releases today"));
    } else {
        info!("There are {} releases today!", releases_today.len()); // Q: Same intellij problem here
    }

    // Enrich today's releases with release.spotify metadata
    if let Err(e) = release::enrich_with_spotify(
        config.spotify_client_id.clone(),
        config.spotify_client_secret.clone(),
        &mut releases_today,
    )
    .await
    {
        fatal_error(e.to_string())
    };

    // TODO: Merge existing config with new enriched releases
    // Will have to do a "smart" merge to not overwrite existing releases

    // // Enrich today's releases with metallum metadata
    // let releases_today = match release::enrich_with_metallum(releases_today) {
    //     Ok(releases_today) => releases_today,
    //     Err(e) => fatal_error(e),
    // };

    // TODO: Sort by popularity, genre, etc.

    // TODO: Slack alert releases
}

fn setup() {
    let cli = CLI::parse();

    if cli.debug {
        env::set_var("RUST_LOG", "metalpal=debug");
    } else {
        env::set_var("RUST_LOG", "metalpal=info");
    }

    env_logger::init();
}

// Q: Should I return a String for errors or my own custom error?
// My guess: implement Display trait on my custom type so it can be println!'d. Is this correct?
fn load_or_setup_config() -> Result<Config, AppError> {
    match config::load_config() {
        Ok(config) => {
            debug!("Successfully loaded existing config");
            Ok(config)
        }
        Err(e) => {
            error!("Error loading config: {:?}", e);
            config::setup_config()
        }
    }
}

fn exit(m: String) -> ! {
    info!("{}", m);
    std::process::exit(0);
}

fn fatal_error(m: String) -> ! {
    error!("{}", m);
    std::process::exit(1);
}
