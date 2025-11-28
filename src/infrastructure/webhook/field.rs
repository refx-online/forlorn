use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub struct Field {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub inline: bool,
}

impl Field {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            inline: false,
        }
    }

    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }
}
