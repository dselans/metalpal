use crate::config::Release;
use crate::{config, AppError};
use slack::attachment::attachment::{Attachment, AttachmentField};
use slack::chat::post_message::{post_message, PostMessageRequest};
use slack::http_client::default_client;
use slack_rust as slack;

pub struct Slack {
    token: String,
    channels: Vec<String>,
}

impl Slack {
    pub fn new(cfg: &config::Config) -> Self {
        Self {
            token: cfg.slack_bot_token.clone(),
            channels: cfg.slack_channels.clone(),
        }
    }

    pub async fn post_releases(&self, releases: &Vec<&Release>) -> Result<(), AppError> {
        let slack_api_client = default_client();
        let unix_ts = chrono::Local::now().timestamp() as i32;

        for channel in &self.channels {
            let param = PostMessageRequest {
                channel: channel.to_string(),
                text: Some(format!(
                    ":tada: There are *{}* releases today! :tada:",
                    releases.len()
                )),
                ..Default::default()
            };

            post_message(&slack_api_client, &param, &self.token).await?;

            let mut iter = 1;

            for release in releases {
                let spotify_metadata = release.spotify.clone().unwrap();
                let metallum_metadata = release.metallum.clone().unwrap();

                let param = PostMessageRequest {
                    channel: channel.to_string(),
                    attachments: Some(vec![Attachment {
                        color: Some("#36a64f".to_string()),
                        title: Some(format!("{}. {} - {}", iter, release.artist, release.album)),
                        title_link: Some(metallum_metadata.url.clone()),
                        // text: Some(format!("\n\n{}\n\n{}", metallum_metadata.description_short.clone(), metallum_metadata.img_url.clone())),
                        thumb_url: Some(metallum_metadata.img_url.clone()),
                        fields: Some(vec![
                            AttachmentField {
                                title: Some("Release Date".to_string()),
                                value: Some(release.date.to_string()),
                                short: Some(true),
                            },
                            AttachmentField {
                                title: Some("Genres".to_string()),
                                value: Some(metallum_metadata.genre.clone()),
                                short: Some(true),
                            },
                            AttachmentField {
                                title: Some("Country".to_string()),
                                value: Some(metallum_metadata.country_origin.clone()),
                                short: Some(true),
                            },
                            AttachmentField {
                                title: Some("Spotify Popularity".to_string()),
                                value: Some(spotify_metadata.popularity.to_string()),
                                short: Some(true),
                            },
                            AttachmentField {
                                title: Some("Spotify Followers".to_string()),
                                value: Some(spotify_metadata.followers.to_string()),
                                short: Some(true),
                            },
                            AttachmentField {
                                title: Some("Spotify Artist ID".to_string()),
                                value: Some(spotify_metadata.id.clone()),
                                short: Some(true),
                            },
                        ]),
                        footer: Some("Metalpal".to_string()),
                        footer_icon: Some("https://emojis.slackmojis.com/emojis/images/1648645351/56886/metal.png?1648645351".to_string(), ),
                        ts: Some(unix_ts.clone()),
                        ..Default::default()
                    }]),
                    ..Default::default()
                };

                post_message(&slack_api_client, &param, &self.token).await?;
                iter += 1;
            }
        }

        Ok(())
    }
}
