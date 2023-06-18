mod config;
mod display;
mod error;
mod release;

extern crate prettytable;
extern crate term;

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

    // Get today's releases
    let mut releases_today = release::get_releases_today(&config.releases);

    // Save again
    if let Err(e) = config::save_config(&config) {
        // Q: My IDE can't tell that to_string exists - why not?
        fatal_error(e.to_string());
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

    // Enrich today's releases with release.spotify metadata
    // if let Err(e) = release::enrich_with_metallum(&mut releases_today).await {
    //     fatal_error(e.to_string())
    // };

    // Mark releases as skip that do not contain needed data or do not match our criteria
    let releases_match = release::set_skip(&config, &mut releases_today);

    // Merge today's releases with existing releases
    release::merge_releases(&mut config.releases, &releases_today);

    // Save again
    if let Err(e) = config::save_config(&config) {
        // Q: My IDE can't tell that to_string exists - why not?
        fatal_error(e.to_string());
    }

    display::display(releases_match, releases_today);

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
