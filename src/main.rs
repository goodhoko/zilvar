use db::setup_db;
use eyre::Result;
use jiff::{SignedDuration, Timestamp};
use model::Doggo;
use tokio::time::sleep;

mod cyklobazar_scraper;
mod db;
mod model;

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
