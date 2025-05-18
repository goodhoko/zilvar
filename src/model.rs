use std::{collections::HashMap, time::Duration};

use eyre::{Context, Result};
use jiff::Timestamp;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{cyklobazar_scraper::get_ads, notification::Mailer};

const SCRAPING_INTERVAL: Duration = Duration::from_secs(60 * 60);

// A watchdog set to sniff for ads matching a certain search URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doggo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub url: Url,
    last_run: Option<Timestamp>,
    /// Previously sniffed Ads indexed by their cyklobazar id.
    sniffs: HashMap<String, Sniff>,
}

impl Doggo {
    #[expect(unused)]
    pub fn new(name: String, email: String, url: Url) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            email,
            url,
            last_run: None,
            sniffs: HashMap::new(),
        }
    }

    pub fn new_with_id(id: Uuid, name: String, email: String, url: Url) -> Self {
        Self {
            id,
            name,
            email,
            url,
            last_run: None,
            sniffs: HashMap::new(),
        }
    }

    pub fn should_run_now(&self) -> bool {
        self.last_run.is_none_or(|las_run| {
            Timestamp::now().duration_since(las_run)
                > SCRAPING_INTERVAL
                    .try_into()
                    .expect("SCRAPING_INTERVAL fits into Duration")
        })
    }

    pub fn next_run(&self) -> Timestamp {
        let Some(last_run) = self.last_run else {
            return Timestamp::now();
        };

        last_run + SCRAPING_INTERVAL
    }

    /// Fetches latest ads from cyklobazar.cz and returns any that weren't sniffed yet.
    pub async fn run(&mut self, mailer: &Mailer) -> Result<()> {
        let mut new_ads = get_ads(self.url.clone()).await?;
        new_ads.retain(|ad| {
            // TODO: changed price should behave as if the ad is new.
            !self.seen_ad(ad)
        });

        println!(
            "Found {} new ads for {} ({}):",
            new_ads.len(),
            self.email,
            self.url
        );

        if !new_ads.is_empty() {
            mailer.notify(self, &new_ads).await.wrap_err_with(|| {
                format!(
                    "notifying about adds sniffed by doggo {} ({})",
                    self.name, self.id
                )
            })?;
        }

        for ad in new_ads.iter() {
            self.see_ad(ad.clone());
        }
        self.last_run = Some(Timestamp::now());

        Ok(())
    }

    fn seen_ad(&self, ad: &Ad) -> bool {
        self.sniffs.contains_key(&ad.cb_id)
    }

    fn see_ad(&mut self, ad: Ad) {
        let id = ad.cb_id.clone();
        let sniff = Sniff {
            ad,
            last_sniffed_at: Timestamp::now(),
        };

        self.sniffs.insert(id, sniff);
    }
}

/// A single impression of an ad by a Doggo.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Sniff {
    pub last_sniffed_at: Timestamp,
    pub ad: Ad,
}

// A single ad at cyklobazar.cz.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ad {
    pub cb_id: String,
    pub title: String,
}

impl Ad {
    pub fn url(&self) -> Url {
        let id = &self.cb_id;
        let url = format!("https://www.cyklobazar.cz/inzerat/{id}/x");
        Url::parse(&url).expect("we constructed a valid url")
    }
}
