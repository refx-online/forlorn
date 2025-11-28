use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub struct Thumbnail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

impl Thumbnail {
    pub fn new() -> Self {
        Self {
            url: None,
            proxy_url: None,
            height: None,
            width: None,
        }
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}
