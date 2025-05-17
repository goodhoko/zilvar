use eyre::{Context, Result, eyre};
use hickory_resolver::{Resolver, config::ResolverConfig, name_server::TokioConnectionProvider};
use mail_send::{SmtpClientBuilder, mail_builder::MessageBuilder};

use crate::model::{Ad, Doggo};

const DOMAIN: &str = "doggo.jentak.co";

pub async fn notify(doggo: &Doggo, new_ads: &[Ad]) -> Result<()> {
    let addressee = doggo.email.as_str();

    let message = MessageBuilder::new()
        .from((doggo.name.clone(), format!("doggo@{DOMAIN}")))
        .to(vec![addressee])
        .subject("Vyčmuchal jsem nové inzeráty!")
        .text_body(build_message(new_ads));

    let mail_exchange = get_mail_exchange_address(addressee)
        .await
        .wrap_err_with(|| format!("getting mail exchange address for {addressee}"))?;

    SmtpClientBuilder::new(&mail_exchange, 25)
        .implicit_tls(false)
        .connect()
        .await
        .wrap_err_with(|| format!("connecting to mail exchange at {mail_exchange}"))?
        .send(message)
        .await
        .wrap_err_with(|| format!("sending email to {addressee}"))?;

    println!("Notified {} about {} new adds.", addressee, new_ads.len());

    Ok(())
}

async fn get_mail_exchange_address(email_addres: &str) -> Result<String> {
    let domain = get_email_domain(email_addres)?;

    let resolver = Resolver::builder_with_config(
        ResolverConfig::default(),
        TokioConnectionProvider::default(),
    )
    .build();

    let lookup = resolver
        .mx_lookup(domain)
        .await
        .wrap_err_with(|| format!("looking up MX DNS records for {domain}"))?;
    let mx_record = lookup
        .iter()
        .next()
        .ok_or(eyre!("No MX records found for {domain}."))?;

    Ok(mx_record.exchange().to_string())
}

fn get_email_domain(email: &str) -> Result<&str> {
    email
        .split('@')
        .nth(1)
        .ok_or(eyre!("'{email}' seems like invalid email address"))
}

fn build_message(ads: &[Ad]) -> String {
    let mut res = "=== NOVÉ INZERÁTY ===\n\n".to_string();
    for ad in ads.iter() {
        res += &format!("- {}: {}\r\n", &ad.title, ad.url());
    }

    res
}
