use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[allow(dead_code)]
pub type ConfigManager = Arc<Config>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub omajinai: OmajinaiConfig,
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
pub struct OmajinaiConfig {
    pub base_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 3030,
            database: DatabaseConfig::default(),
            redis: RedisConfig::default(),
            omajinai: OmajinaiConfig::default(),
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

impl Default for OmajinaiConfig {
    fn default() -> Self {
        Self { base_url: "http://localhost:9292".into() }
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let mut config = Config::default();

        if let Ok(port) = std::env::var("PORT") {
            config.port = port.parse()?;
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

        if let Ok(omajinai_url) = std::env::var("OMAJINAI_BASE_URL") {
            config.omajinai.base_url = omajinai_url;
        }

        Ok(config)
    }
}
