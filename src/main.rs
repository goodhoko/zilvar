use std::time::Duration;

use eyre::Result;
use jiff::Timestamp;
use model::Doggo;
use persistence::Kennel;
use tokio::time::sleep;
use uuid::Uuid;

mod cyklobazar_scraper;
mod model;
mod notification;
mod persistence;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut kennel = Kennel::new("./db.json".parse()?).await;

    let my_doggo = Doggo::new_with_id(
        Uuid::nil(),
        "Azor".to_string(),
        "jentak@hey.com".to_string(),
        "https://www.cyklobazar.cz/sedlovky?filter%5Blist_types_id%5D=15"
            .parse()
            .expect("valid url"),
    );
    kennel.doggos.entry(my_doggo.id).or_insert(my_doggo);
    kennel.persist().await?;

    loop {
        run_pending_doggos(&mut kennel).await;

        if let Err(err) = kennel.persist().await {
            println!("Failed to persist kennel: {err:#?}");
        }

        let until_next_run = kennel.until_next_run().unwrap_or(Duration::from_secs(1));
        println!(
            "Sleeping {until_next_run:?} until the next closest run at {:?}",
            Timestamp::now() + until_next_run
        );
        sleep(until_next_run).await
    }
}

async fn run_pending_doggos(kennel: &mut Kennel) {
    let pending_doggos = kennel
        .doggos
        .values_mut()
        .filter(|doggo| doggo.should_run_now());
    for doggo in pending_doggos {
        match doggo.run().await {
            Ok(new_ads) => {
                println!("would notify about: {new_ads:?}");
            }
            Err(err) => println!("Failed to run {doggo:?}: {err:#}"),
        }
    }
}
