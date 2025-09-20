use core::fmt;
use std::error::Error;

use config::{Config, ConfigError, File};

#[derive(Debug)]
pub enum ConfigLoadError {
    Source(ConfigError),
    Field(ConfigError),
}

impl fmt::Display for ConfigLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigLoadError::Source(err) => {
                write!(f, "[config load error] source error: {}", err)
            }
            ConfigLoadError::Field(err) => {
                write!(f, "[config load error] field error: {}", err)
            }
        }
    }
}

impl Error for ConfigLoadError {}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub interval_ms: u64,
}

pub fn load_config() -> Result<AppConfig, ConfigLoadError> {
    let source = Config::builder()
        .add_source(File::with_name("config.yml"))
        .build();
    let config = match source {
        Ok(config) => config,
        Err(e) => {
            return Err(ConfigLoadError::Source(e));
        }
    };
    let port = match config.get_int("port") {
        Ok(port) => port as u16,
        Err(e) => {
            return Err(ConfigLoadError::Field(e));
        }
    };
    let interval_ms = match config.get_int("interval_ms") {
        Ok(interval_ms) => interval_ms as u64,
        Err(e) => {
            return Err(ConfigLoadError::Field(e));
        }
    };
    Ok(AppConfig { port, interval_ms })
}
