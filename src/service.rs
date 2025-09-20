use std::error::Error;
use std::sync::Arc;

use actix_web::{
    App, HttpServer, Responder,
    middleware::Logger,
    web::{Data, Json, get},
};
use serde::{Deserialize, Serialize};

use crate::{
    config::AppConfig,
    scraper::{DollarRates, Scraper},
};
pub struct AppState {
    pub scraper: Arc<Scraper>,
}

const PREFIX: &str = "/api/v1";

pub async fn run_app(config: AppConfig, scraper: Arc<Scraper>) -> Result<(), Box<dyn Error>> {
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                scraper: scraper.clone(),
            }))
            .wrap(Logger::new(
                "address: %a | \"%r\" | status code: %s | time: %Ts",
            ))
            .route(
                format!("{PREFIX}/rates").as_str(),
                get().to(get_rates_handler),
            )
    })
    .bind(("127.0.0.1", config.port))?
    .run()
    .await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RatesResponse {
    pub rates: DollarRates,
    #[serde(rename = "updatedAt")]
    pub updated_at: u128,
}

impl RatesResponse {
    pub async fn build(scraper: Arc<Scraper>) -> RatesResponse {
        let rates = scraper.cache.lock().unwrap().as_ref().unwrap().clone();
        let updated_at = scraper.updated_at.lock().unwrap();
        Self {
            rates,
            updated_at: *updated_at,
        }
    }
}

pub async fn get_rates_handler(state: Data<AppState>) -> impl Responder {
    Json(RatesResponse::build(state.scraper.clone()).await)
}
