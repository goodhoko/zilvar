use eyre::{Context, Result};
use futures::StreamExt;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::get;
use scraper::{ElementRef, Html, Selector};
use sqlx::{Executor, Pool, Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let url = "https://www.cyklobazar.cz/sedlovky?filter%5Blist_types_id%5D=15";

    let ads = get_ads(url)
        .await
        .with_context(|| format!("fetching ads from {url}"))?;
    println!("{ads:#?}");

    let db = setup_db().await?;

    let new_ads = futures::stream::iter(ads.iter())
        .filter(|ad| async {
            let query = sqlx::query("INSERT INTO ads ( id, title ) VALUES ($1, $2);")
                .bind(&ad.id)
                .bind(&ad.title);

            let Ok(res) = db.execute(query).await else {
                return false;
            };

            res.rows_affected() == 1
        })
        .collect::<Vec<_>>()
        .await;

    println!("NEW: {new_ads:#?}");

    Ok(())
}

async fn setup_db() -> Result<Pool<Sqlite>> {
    let db_url = "./sqlite:db";

    if !Sqlite::database_exists(db_url).await? {
        Sqlite::create_database(db_url).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    let query = sqlx::query("CREATE TABLE IF NOT EXISTS ads (title TEXT, id TEXT PRIMARY KEY);");
    pool.execute(query).await.context("creating ads table")?;

    Ok(pool)
}

async fn get_ads(url: &str) -> Result<Vec<Ad>> {
    let response = get(url).await.with_context(|| format!("fetching {url}"))?;
    let response_body = response
        .text()
        .await
        .with_context(|| format!("decoding response body from {url}"))?;
    let html = Html::parse_document(&response_body);

    // TODO: filter out topped ads - they're unrelated junk mostly.
    let ad_selector = Selector::parse("ul.cb-offer-list li.cb-offer-list__item a.cb-offer")
        .expect("valid selector");
    let ads = html
        .select(&ad_selector)
        .filter_map(|item| {
            Some(Ad {
                id: parse_id(item)?.to_string(),
                title: parse_title(item)?,
            })
        })
        .collect();

    Ok(ads)
}

fn parse_id(ad: ElementRef) -> Option<&str> {
    let href = ad.attr("href")?;
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"/inzerat/(?P<id>.+?)/").expect("valid regex"));
    Some(RE.captures(href)?.name("id")?.as_str())
}

fn parse_title(ad: ElementRef) -> Option<String> {
    let heading_selector = Selector::parse("div.cb-offer__header h4").expect("valid selector");
    let text_nodes = ad.select(&heading_selector).next()?.text();
    Some(text_nodes.collect::<Vec<_>>().join(" "))
}

#[derive(Debug, Clone)]
struct Ad {
    title: String,
    id: String,
}
