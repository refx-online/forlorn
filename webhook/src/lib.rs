use reqwest::Error;
use serde::Serialize;
use std::sync::LazyLock;

pub mod author;
pub mod embed;
pub mod field;
pub mod footer;
pub mod image;
pub mod provider;
pub mod thumbnail;
pub mod video;

pub use self::author::Author;
pub use self::embed::Embed;
pub use self::field::Field;
pub use self::footer::Footer;
pub use self::image::Image;
pub use self::provider::Provider;
pub use self::thumbnail::Thumbnail;
pub use self::video::Video;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    #[serde(default)]
    pub embeds: Vec<Embed>,
}

pub struct Webhook {
    url: String,
    content: Option<String>,
    username: Option<String>,
    avatar_url: Option<String>,
    tts: Option<bool>,
    embeds: Vec<Embed>,
}

#[derive(Debug)]
pub enum WebhookError {
    EmptyPayload,
    ContentTooLong,
    RequestFailed(Error),
}

impl From<Error> for WebhookError {
    fn from(err: Error) -> Self {
        Self::RequestFailed(err)
    }
}

impl Webhook {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            content: None,
            username: None,
            avatar_url: None,
            tts: None,
            embeds: Vec::new(),
        }
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn avatar_url(mut self, avatar_url: impl Into<String>) -> Self {
        self.avatar_url = Some(avatar_url.into());
        self
    }

    pub fn add_embed(mut self, embed: Embed) -> Self {
        self.embeds.push(embed);
        self
    }

    fn validate(&self) -> Result<(), WebhookError> {
        if self.content.is_none() && self.embeds.is_empty() {
            return Err(WebhookError::EmptyPayload);
        }

        if let Some(content) = &self.content {
            if content.len() > 2000 {
                return Err(WebhookError::ContentTooLong);
            }
        }

        Ok(())
    }

    fn build_payload(&self) -> WebhookPayload {
        WebhookPayload {
            content: self.content.clone(),
            username: self.username.clone(),
            avatar_url: self.avatar_url.clone(),
            tts: self.tts,
            embeds: self.embeds.clone(),
        }
    }

    pub async fn post(self) -> Result<(), WebhookError> {
        self.validate()?;

        let payload = self.build_payload();

        CLIENT
            .post(&self.url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
