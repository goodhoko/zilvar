use std::env;

use eyre::{Context, Result, eyre};
use hickory_resolver::{
    Resolver,
    config::ResolverConfig,
    name_server::{GenericConnector, TokioConnectionProvider},
    proto::runtime::TokioRuntimeProvider,
};
use mail_send::{
    SmtpClientBuilder,
    mail_auth::{
        common::crypto::{RsaKey, Sha256},
        dkim::{DkimSigner, Done},
    },
    mail_builder::MessageBuilder,
};
use tokio::fs;

use crate::model::{Ad, Doggo};

const DKIM_PRIVATE_KEY_PATH_ENV_VAR: &str = "DKIM_PRIVATE_KEY_PATH";
const DKIM_SELECTOR: &str = "z1";
const DOMAIN: &str = "zilvar.jentak.co";

pub struct Mailer {
    dns_resolver: Resolver<GenericConnector<TokioRuntimeProvider>>,
    dkim_signer: DkimSigner<RsaKey<Sha256>, Done>,
}

impl Mailer {
    pub async fn new() -> Result<Self> {
        let private_key_path = env::var(DKIM_PRIVATE_KEY_PATH_ENV_VAR).wrap_err_with(|| format!("getting path to DKIM private key from the environment variable {DKIM_PRIVATE_KEY_PATH_ENV_VAR}"))?;
        let private_key = fs::read_to_string(&private_key_path)
            .await
            .wrap_err_with(|| format!("reading DKIM private key from {private_key_path}"))?;
        let pk_rsa = RsaKey::<Sha256>::from_rsa_pem(&private_key).unwrap();

        Ok(Self {
            dns_resolver: Resolver::builder_with_config(
                ResolverConfig::default(),
                TokioConnectionProvider::default(),
            )
            .build(),
            dkim_signer: DkimSigner::from_key(pk_rsa)
                .domain(DOMAIN)
                .selector(DKIM_SELECTOR)
                .headers(["From", "To", "Subject"]),
        })
    }

    pub async fn notify(&self, doggo: &Doggo, new_ads: &[Ad]) -> Result<()> {
        let addressee = doggo.email.as_str();

        let mail_exchange = self
            .get_mail_exchange_address(addressee)
            .await
            .wrap_err_with(|| format!("getting mail exchange address for {addressee}"))?;

        let message = MessageBuilder::new()
            .from((doggo.name.clone(), format!("doggo@{DOMAIN}")))
            .to(vec![addressee])
            .subject("Vyčmuchal jsem nové inzeráty!")
            .text_body(build_message(new_ads));

        SmtpClientBuilder::new(&mail_exchange, 25)
            .implicit_tls(false)
            .connect()
            .await
            .wrap_err_with(|| format!("connecting to mail exchange at {mail_exchange}"))?
            .send_signed(message, &self.dkim_signer)
            .await
            .wrap_err_with(|| format!("sending email to {addressee}"))?;

        println!("Notified {} about {} new adds.", addressee, new_ads.len());

        Ok(())
    }

    async fn get_mail_exchange_address(&self, email_addres: &str) -> Result<String> {
        let domain = get_email_domain(email_addres)?;

        let lookup = self
            .dns_resolver
            .mx_lookup(domain)
            .await
            .wrap_err_with(|| format!("looking up MX DNS records for {domain}"))?;
        let mx_record = lookup
            .iter()
            .next()
            .ok_or(eyre!("No MX records found for {domain}."))?;

        Ok(mx_record.exchange().to_string())
    }
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
