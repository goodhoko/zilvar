use std::{collections::HashMap, time::Duration};

use cyklobazar_scraper::get_ads;
use db::setup_db;
use eyre::Result;
use jiff::{SignedDuration, Timestamp};
use reqwest::Url;
use tokio::time::sleep;

mod cyklobazar_scraper;
mod db;

const SCRAPING_INTERVAL: Duration = Duration::from_secs(60 * 10);

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // TODO: fetch from DB every iteration instead.
    let mut doggos = vec![Doggo::new(
        "jentak@hey.com".to_string(),
        "https://www.cyklobazar.cz/sedlovky?filter%5Blist_types_id%5D=15"
            .parse()
            .expect("valid url"),
    )];

    let _db = setup_db().await?;

    loop {
        let pending_doggos = doggos.iter_mut().filter(|doggo| doggo.should_run_now());

        for doggo in pending_doggos {
            if let Err(err) = doggo.run().await {
                println!("Failed to run {doggo:?}: {err:#}");
            }
        }

        let mut next_runs: Vec<Timestamp> = doggos.iter().map(|doggo| doggo.next_run()).collect();
        next_runs.sort();
        let closest_next_run = next_runs.first();
        let until_next_run = closest_next_run
            .map(|next| Timestamp::now().duration_until(*next))
            // Prevent busy loops if there are no doggos whatsoever.
            .unwrap_or(SignedDuration::from_secs(1))
            // Prevent busy loops, just in case.
            .max(SignedDuration::from_secs(1))
            // We're already positive thanks to max() above but we need to convert the type to
            // Duration to be accepted by sleep().
            .unsigned_abs();

        println!(
            "Sleeping {until_next_run:?} until the next closest run at {:?}",
            closest_next_run
        );

        sleep(until_next_run).await
    }
}

#[derive(Debug, Clone)]
struct Ad {
    title: String,
    id: String,
}

#[derive(Debug, Clone)]
struct Doggo {
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
