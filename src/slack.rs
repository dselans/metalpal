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

    pub async fn post_releases(&self, releases: &Vec<Release>) -> Result<(), AppError> {
        let slack_api_client = default_client();

        for channel in &self.channels {
            for release in releases {
                // TODO: Fill this out nicely
                let param = PostMessageRequest {
                    channel: channel.to_string(),
                    text: Some(format!(
                        "New release: {} - {}",
                        release.artist, release.album
                    )),
                    attachments: Some(vec![Attachment {
                        color: Some("#36a64f".to_string()),
                        author_name: Some("slack-rust".to_string()),
                        author_link: Some("https://www.irasutoya.com/".to_string()),
                        author_icon: Some("https://2.bp.blogspot.com/-3o7K8_p8NNM/WGCRsl8GiCI/AAAAAAABAoc/XKnspjvc0YIoOiSRK9HW6wXhtlnZvHQ9QCLcB/s800/pyoko_hashiru.png".to_string()),
                        title: Some("title".to_string()),
                        title_link: Some("https://www.irasutoya.com/".to_string()),
                        pretext: Some("Optional pre-text that appears above the attachment block".to_string()),
                        text: Some("Optional `text` that appears within the attachment".to_string()),
                        thumb_url: Some("https://2.bp.blogspot.com/-3o7K8_p8NNM/WGCRsl8GiCI/AAAAAAABAoc/XKnspjvc0YIoOiSRK9HW6wXhtlnZvHQ9QCLcB/s800/pyoko_hashiru.png".to_string()),
                        fields: Some(vec![
                            AttachmentField {
                                title: Some("A field's title".to_string()),
                                value: Some("This field's value".to_string()),
                                short: Some(false),
                            },
                        ]),
                        mrkdwn_in: Some(vec!["text".to_string()]),
                        footer: Some("footer".to_string()),
                        footer_icon: Some("https://1.bp.blogspot.com/-46AF2TCkb-o/VW6ORNeQ3UI/AAAAAAAAt_4/TA4RrGVcw_U/s800/pyoko05_cycling.png".to_string(), ),
                        ts: Some(123456789),
                        ..Default::default()
                    }]),
                    ..Default::default()
                };

                if let Err(e) = post_message(&slack_api_client, &param, &self.token).await {
                    // post_message() is broken and generates a serde error; ignore it
                    if !e.to_string().contains("Serde") {
                        return Err(AppError::SlackError(format!(
                            "Error posting release to slack: {}",
                            e.to_string()
                        )));
                    }
                }
            }

            break;
        }

        Ok(())
    }
}
