use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipType {
    Text,
    Code,
    Link,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: i64,
    pub content: String,
    #[serde(rename = "type")]
    pub clip_type: ClipType,
    pub source: Option<String>,
    pub created_at: i64,
    pub pinned: bool,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: Theme,
    pub ai_enabled: bool,
    pub auto_hide: bool,
    pub max_clips: i64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            ai_enabled: false,
            auto_hide: true,
            max_clips: 500,
        }
    }
}
