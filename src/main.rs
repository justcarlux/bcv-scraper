use std::{error::Error, sync::Arc, time::Duration};

use crate::{config::load_config, scraper::Scraper, service::run_app};
use env_logger::Env;
use tokio::main;

mod config;
mod scraper;
mod service;
mod util;

#[main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let config = load_config()?;

    let scraper = Arc::new(Scraper::new());
    scraper.perform_scrape().await?;
    scraper
        .clone()
        .start_refresh_task(Duration::from_millis(config.interval_ms));

    run_app(config, scraper).await
}
