use core::fmt;
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
    time::Duration,
};

use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::util::current_time_millis;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DollarRates {
    eur: f64,
    cny: f64,
    #[serde(rename = "try")]
    _try: f64,
    rub: f64,
    usd: f64,
}

impl fmt::Display for DollarRates {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "eur => {} | cny => {} | try => {} | rub => {} | usd => {}",
            self.eur, self.cny, self._try, self.rub, self.usd
        )
    }
}

#[derive(Debug)]
pub enum ScrapeError {
    Request(String),
    Parse(String),
}

impl Error for ScrapeError {}

impl fmt::Display for ScrapeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScrapeError::Request(message) => {
                write!(f, "[scrape error] request error: {}", message)
            }
            ScrapeError::Parse(message) => {
                write!(f, "[scrape error] parse error: {}", message)
            }
        }
    }
}

pub struct Scraper {
    base_url: String,
    client: Client,
    row_selector: Selector,
    currency_name_selector: Selector,
    currency_value_selector: Selector,
    pub updated_at: Mutex<u128>,
    pub cache: Mutex<Option<DollarRates>>,
}

impl Scraper {
    pub fn new() -> Self {
        Self {
            base_url: "https://www.bcv.org.ve/".to_owned(),
            client: Client::builder()
                .danger_accept_invalid_certs(true) // i'm so sorry but the damn website is not helping.
                .build()
                .unwrap(),
            row_selector: Selector::parse(".row.recuadrotsmc").unwrap(),
            currency_name_selector: Selector::parse("span").unwrap(),
            currency_value_selector: Selector::parse("strong").unwrap(),
            updated_at: Mutex::new(0),
            cache: Mutex::new(None),
        }
    }

    pub async fn perform_scrape(&self) -> Result<(), ScrapeError> {
        let response = match self.client.get(&self.base_url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Err(ScrapeError::Request(format!(
                    "failed to send request: {}",
                    e
                )));
            }
        };

        let status = response.status();
        if !status.is_success() {
            return Err(ScrapeError::Request(format!(
                "invalid request status response: {}",
                status.as_u16()
            )));
        }

        let html = match response.text().await {
            Ok(html) => html,
            Err(e) => {
                return Err(ScrapeError::Request(format!(
                    "failed to parse request text: {}",
                    e
                )));
            }
        };

        let document = Html::parse_document(&html);

        let mut fields = HashMap::new();
        for element in document.select(&self.row_selector) {
            let currency_name = match element.select(&self.currency_name_selector).next() {
                Some(currency_name_element) => match currency_name_element.text().next() {
                    Some(name) => name.trim().to_lowercase(),
                    None => {
                        return Err(ScrapeError::Parse(format!(
                            "currency name for the following row was empty: {}",
                            element.html()
                        )));
                    }
                },
                None => {
                    return Err(ScrapeError::Parse(format!(
                        "failed to find currency name for the following row: {}",
                        element.html()
                    )));
                }
            };
            let currency_value = match element.select(&self.currency_value_selector).next() {
                Some(currency_value_element) => match currency_value_element.text().next() {
                    Some(raw_value) => match raw_value.trim().replace(",", ".").parse::<f64>() {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(ScrapeError::Parse(format!(
                                "failed to parse currency value as float: {}",
                                e
                            )));
                        }
                    },
                    None => {
                        return Err(ScrapeError::Parse(format!(
                            "currency value for the following row was empty: {}",
                            element.html()
                        )));
                    }
                },
                None => {
                    return Err(ScrapeError::Parse(format!(
                        "failed to find currency value for the following row: {}",
                        element.html()
                    )));
                }
            };
            fields.insert(currency_name, currency_value);
        }

        let dollar_rates: DollarRates =
            match serde_json::to_value(fields).and_then(|v| serde_json::from_value(v)) {
                Ok(rates) => rates,
                Err(e) => {
                    return Err(ScrapeError::Parse(format!(
                        "failed to map data to the dollar rates struct: {}",
                        e
                    )));
                }
            };

        let mut cache = self.cache.lock().unwrap();
        *cache = Some(dollar_rates.clone());

        let mut updated_at = self.updated_at.lock().unwrap();
        *updated_at = current_time_millis();

        tracing::info!("successfully scraped dollar rates: {}", dollar_rates);

        Ok(())
    }

    pub fn start_refresh_task(self: Arc<Self>, interval: Duration) {
        let scraper = self.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;

                if let Err(error) = scraper.clone().perform_scrape().await {
                    tracing::error!("periodic scrape failed: {}", error);
                }
            }
        });
    }
}
