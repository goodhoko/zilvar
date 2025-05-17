use eyre::{Context, Result};
use mail_send::{SmtpClientBuilder, mail_builder::MessageBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    println!("{:#?}", send().await);

    Ok(())
}

async fn send() -> Result<()> {
    let message = MessageBuilder::new()
        .from(("test sender", "test@zilvar.jentak.co"))
        .to(vec!["goodhoko@gmail.com"])
        .subject("Testing out direct SMTP delivery")
        .text_body("Does this work?");

    SmtpClientBuilder::new("gmail-smtp-in.l.google.com", 25)
        // .credentials(("john", "p4ssw0rd"))
        .implicit_tls(false)
        .connect()
        .await
        .wrap_err("connecting")?
        .send(message)
        .await
        .wrap_err("sending")?;

    Ok(())
}
