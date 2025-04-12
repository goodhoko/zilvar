use std::{collections::HashMap, time::Duration};

use eyre::Result;
use jiff::Timestamp;
use reqwest::Url;

use crate::cyklobazar_scraper::get_ads;

const SCRAPING_INTERVAL: Duration = Duration::from_secs(60 * 10);

#[derive(Debug, Clone)]
pub struct Ad {
    pub title: String,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct Doggo {
    email: String,
    url: Url,
    last_run: Option<Timestamp>,
    seen_ads: HashMap<String, Ad>,
}

impl Doggo {
    pub fn new(email: String, url: Url) -> Self {
        Self {
            email,
            url,
            last_run: None,
            seen_ads: HashMap::new(),
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

    pub async fn run(&mut self) -> Result<()> {
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

        // TODO: notify email instead
        println!("{new_ads:#?}");

        for ad in new_ads.iter() {
            self.see_ad(ad.clone());
        }
        self.last_run = Some(Timestamp::now());

        Ok(())
    }

    fn seen_ad(&self, ad: &Ad) -> bool {
        // TODO: ask DB
        self.seen_ads.contains_key(&ad.id)
    }

    fn see_ad(&mut self, ad: Ad) {
        // TODO: persist to DB
        self.seen_ads.insert(ad.id.clone(), ad);
    }
}
