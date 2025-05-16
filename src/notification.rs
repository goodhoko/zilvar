use eyre::Result;
use mail_send::{SmtpClientBuilder, mail_builder::MessageBuilder};

use crate::model::{Ad, Doggo};

const DOMAIN: &str = "doggo.jentak.co";

pub async fn notify(doggo: &Doggo, new_ads: &Vec<Ad>) -> Result<()> {
    let message = MessageBuilder::new()
        .from((doggo.name.clone(), format!("doggo@{DOMAIN}")))
        .to(vec![doggo.email.clone()])
        .subject("Vyčmuchal jsem nové inzeráty!")
        .text_body(build_message(new_ads));

    SmtpClientBuilder::new("smtp.gmail.com", 587)
        .implicit_tls(true)
        .credentials(("john", "p4ssw0rd"))
        .connect()
        .await?
        .send(message)
        .await?;

    Ok(())
}

fn build_message(ads: &Vec<Ad>) -> String {
    let mut res = "=== NOVÉ INZERÁTY ===\n\n".to_string();
    for ad in ads.iter() {
        res += &format!("- {}: {}", &ad.title, ad.url());
    }

    res
}
