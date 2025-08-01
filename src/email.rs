use crate::config::Config;
use lettre::message::{header, Mailbox, MultiPart, SinglePart};
use lettre::{Message, SmtpTransport, Transport};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("failed to build email: {0}")]
    Build(#[from] lettre::error::Error),
    #[error("failed to send email: {0}")]
    Send(#[from] lettre::transport::smtp::Error),
}

pub struct Emailer {
    mailer: SmtpTransport,
    from: Mailbox,
    base_link: String,
}

impl Emailer {
    pub fn new(cfg: &Config) -> Self {
        let creds = lettre::transport::smtp::authentication::Credentials::new(
            cfg.smtp_username.clone(),
            cfg.smtp_password.clone(),
        );
        let mailer = SmtpTransport::starttls_relay(&cfg.smtp_host)
            .unwrap()
            .port(cfg.smtp_port)
            .credentials(creds)
            .build();
        let from = cfg
            .email_from
            .parse::<Mailbox>()
            .expect("invalid from email");
        Self {
            mailer,
            from,
            base_link: cfg.magic_link_base_url.clone(),
        }
    }

    pub fn send_magic_link(&self, to_email: &str, token: &str) -> Result<(), EmailError> {
        let magic_url = format!("{}?token={}", self.base_link, token);
        let subject = "Your Magic Login Link";
        let html_body = format!(
            "<p>Click the link to login (valid for a short time):<br/><a href=\"{0}\">{0}</a></p>",
            magic_url
        );
        let text_body = format!("Login: {}", magic_url);

        let email = Message::builder()
            .from(self.from.clone())
            .to(to_email.parse().unwrap())
            .subject(subject)
            .multipart(MultiPart::alternative() // This is composed of two parts.
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(text_body),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html_body),
                ),
            )?;

        self.mailer.send(&email)?;
        Ok(())
    }
}
