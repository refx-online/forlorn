use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub replay_path: PathBuf,
    pub screenshot_path: PathBuf,
    pub osz_path: PathBuf,
    pub mirror_endpoint: String,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub datadog: DatadogConfig,
    pub r2: CloudflareR2Config,
    pub omajinai: OmajinaiConfig,
    pub webhook: DiscordWebhookConfig,
    pub osu: OsuConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatadogConfig {
    pub host: String,
    pub port: u16,
    pub namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareR2Config {
    pub account_id: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmajinaiConfig {
    pub base_url: String,
    // not quite. omajinai does not serve beatmaps itself
    // but its feature to check for beatmap relies on beatmap service
    // since making another config for that seems overkill, i just put it here!
    pub beatmap_service_url: String,
    pub beatmap_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordWebhookConfig {
    pub score: String,
    pub debug: String, // dev server channel
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsuConfig {
    pub api_key: String,
    // TODO: use v2?
    //pub client_id: i32,
    //pub client_secret: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 3030,
            replay_path: PathBuf::new(),
            screenshot_path: PathBuf::new(),
            osz_path: PathBuf::new(),
            mirror_endpoint: "https://osu.direct".into(),
            database: DatabaseConfig::default(),
            redis: RedisConfig::default(),
            datadog: DatadogConfig::default(),
            r2: CloudflareR2Config::default(),
            omajinai: OmajinaiConfig::default(),
            webhook: DiscordWebhookConfig::default(),
            osu: OsuConfig::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            port: 3306,
            username: "root".into(),
            password: "password".into(),
            database: "gulag".into(),
            max_connections: 10,
            min_connections: 5,
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            port: 6379,
            password: None,
            db: 0,
        }
    }
}

impl Default for DatadogConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8125,
            namespace: "refx".into(),
        }
    }
}

impl Default for CloudflareR2Config {
    fn default() -> Self {
        Self {
            account_id: "account_id".into(),
            access_key: "access_key".into(),
            secret_key: "secret_key".into(),
            bucket: "none".into(),
        }
    }
}

impl Default for OmajinaiConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:9292".into(),
            beatmap_service_url: "https://b.refx.online".into(),
            beatmap_path: PathBuf::from(".data/osu"),
        }
    }
}

impl Default for DiscordWebhookConfig {
    fn default() -> Self {
        Self {
            score: "https://discord.com/api/webhooks/123".into(),
            debug: "https://discord.com/api/webhooks/123".into(),
        }
    }
}

impl Default for OsuConfig {
    fn default() -> Self {
        Self { api_key: "".into() }
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let mut config = Config::default();

        if let Ok(port) = std::env::var("PORT") {
            config.port = port.parse()?;
        }
        if let Ok(replay_path) = std::env::var("REPLAY_PATH") {
            config.replay_path = PathBuf::from(replay_path);
        }
        if let Ok(screenshot_path) = std::env::var("SCREENSHOT_PATH") {
            config.screenshot_path = PathBuf::from(screenshot_path);
        }
        if let Ok(osz_path) = std::env::var("OSZ_PATH") {
            config.osz_path = PathBuf::from(osz_path);
        }
        if let Ok(mirror_endpoint) = std::env::var("MIRROR_ENDPOINT") {
            config.mirror_endpoint = mirror_endpoint;
        }

        if let Ok(db_host) = std::env::var("DATABASE_HOST") {
            config.database.host = db_host;
        }
        if let Ok(db_port) = std::env::var("DATABASE_PORT") {
            config.database.port = db_port.parse()?;
        }
        if let Ok(db_user) = std::env::var("DATABASE_USERNAME") {
            config.database.username = db_user;
        }
        if let Ok(db_pass) = std::env::var("DATABASE_PASSWORD") {
            config.database.password = db_pass;
        }
        if let Ok(db_name) = std::env::var("DATABASE_NAME") {
            config.database.database = db_name;
        }
        if let Ok(max_conn) = std::env::var("DATABASE_MAX_CONNECTIONS") {
            config.database.max_connections = max_conn.parse()?;
        }
        if let Ok(min_conn) = std::env::var("DATABASE_MIN_CONNECTIONS") {
            config.database.min_connections = min_conn.parse()?;
        }

        if let Ok(redis_host) = std::env::var("REDIS_HOST") {
            config.redis.host = redis_host;
        }
        if let Ok(redis_port) = std::env::var("REDIS_PORT") {
            config.redis.port = redis_port.parse()?;
        }
        if let Ok(redis_pass) = std::env::var("REDIS_PASSWORD") {
            config.redis.password = Some(redis_pass);
        }
        if let Ok(redis_db) = std::env::var("REDIS_DB") {
            config.redis.db = redis_db.parse()?;
        }

        if let Ok(datadog_host) = std::env::var("DATADOG_HOST") {
            config.datadog.host = datadog_host;
        }
        if let Ok(datadog_port) = std::env::var("DATADOG_PORT") {
            config.datadog.port = datadog_port.parse()?;
        }
        if let Ok(datadog_namespace) = std::env::var("DATADOG_NAMESPACE") {
            config.datadog.namespace = datadog_namespace;
        }

        if let Ok(r2_account_id) = std::env::var("R2_ACCOUNT_ID") {
            config.r2.account_id = r2_account_id;
        }
        if let Ok(r2_access_key) = std::env::var("R2_ACCESS_KEY") {
            config.r2.access_key = r2_access_key;
        }
        if let Ok(r2_secret_key) = std::env::var("R2_SECRET_KEY") {
            config.r2.secret_key = r2_secret_key;
        }
        if let Ok(r2_bucket) = std::env::var("R2_BUCKET") {
            config.r2.bucket = r2_bucket;
        }

        if let Ok(omajinai_url) = std::env::var("OMAJINAI_BASE_URL") {
            config.omajinai.base_url = omajinai_url;
        }
        if let Ok(omajinai_beatmap_service_url) = std::env::var("OMAJINAI_BEATMAP_SERVICE_URL") {
            config.omajinai.beatmap_service_url = omajinai_beatmap_service_url;
        }
        if let Ok(omajinai_beatmap_path) = std::env::var("OMAJINAI_BEATMAP_PATH") {
            config.omajinai.beatmap_path = PathBuf::from(omajinai_beatmap_path);
        }

        if let Ok(discord_score_webhook) = std::env::var("DISCORD_SCORE_WEBHOOK") {
            config.webhook.score = discord_score_webhook;
        }
        if let Ok(discord_debug_webhook) = std::env::var("DISCORD_DEBUG_WEBHOOK") {
            config.webhook.debug = discord_debug_webhook;
        }

        if let Ok(osu_api_key) = std::env::var("OSU_API_KEY") {
            config.osu.api_key = osu_api_key;
        }

        Ok(config)
    }
}
