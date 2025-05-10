use eyre::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{Url, get};
use scraper::{ElementRef, Html, Selector};

use crate::model::Ad;

pub async fn get_ads(url: Url) -> Result<Vec<Ad>> {
    let response = get(url.clone())
        .await
        .with_context(|| format!("fetching {url}"))?;
    let response_body = response
        .text()
        .await
        .with_context(|| format!("decoding response body from {url}"))?;
    let html = Html::parse_document(&response_body);

    let ad_selector = Selector::parse(
        ".content-layout__main
        > ul.cb-offer-list:not(.cb-offer-list--vertical)
        > li.cb-offer-list__item
        > a.cb-offer:not(.cb-offer--ad)",
    )
    .expect("valid selector");
    let ads = html
        .select(&ad_selector)
        .filter_map(|item| {
            Some(Ad {
                cb_id: parse_id(item)?.to_string(),
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
