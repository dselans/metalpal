mod config;
mod display;
mod error;
mod release;
mod slack;

// Q: What's the diff between 'extern' and 'use'
extern crate prettytable;
extern crate term;

use crate::config::Config;
use crate::error::AppError;
use clap::Parser;
use log::{debug, error, info};
use std::env;

#[tokio::main]
async fn main() {
    let cli = setup();

    let mut config = match load_or_setup_config(&cli) {
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

    // Do spotify-based filtering
    release::set_skip_spotify(&config, &mut releases_today);

    // // Enrich matching releases with metallum metadata
    if let Err(e) = release::enrich_with_metallum(&mut releases_today).await {
        fatal_error(e.to_string())
    };

    release::set_skip_metallum(&config, &mut releases_today);

    // Merge today's releases with existing releases
    release::merge_releases(&mut config.releases, &releases_today);

    // Save again
    if let Err(e) = config::save_config(&config) {
        // Q: My IDE can't tell that to_string exists - why not?
        fatal_error(e.to_string());
    }

    display::display(&releases_today);

    // Send Slack notices
    let slack_client = slack::Slack::new(&config);

    // slack_rust crate doesn't correctly handle Slack responses so we need to
    // catch this particular error and ignore it
    if let Err(e) = slack_client.post_releases(&releases_today).await {
        if !e.to_string().contains("Serde") {
            fatal_error(e.to_string());
        }
    }
}

fn setup() -> config::CLI {
    let cli = config::CLI::parse();

    if cli.debug {
        env::set_var("RUST_LOG", "metalpal=debug");
    } else {
        env::set_var("RUST_LOG", "metalpal=info");
    }

    env_logger::init();

    cli
}

// Q: Should I return a String for errors or my own custom error?
// My guess: implement Display trait on my custom type so it can be println!'d. Is this correct?
fn load_or_setup_config(cli: &config::CLI) -> Result<Config, AppError> {
    match config::load_config() {
        Ok(config) => {
            debug!("Successfully loaded existing config");
            Ok(config)
        }
        Err(e) => {
            error!("Error loading config: {:?}", e);
            config::setup_config(cli)
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
