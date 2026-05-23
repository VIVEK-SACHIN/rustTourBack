//! Transactional email (TravelAndTour `utils/newemail.js`) — Mailtrap dev / SendGrid production.

use lettre::message::header::ContentType;
use lettre::message::{Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::AppConfig;
use crate::utils::error::AppError;

pub struct Email<'a> {
    pub to: &'a str,
    pub first_name: String,
    pub url: &'a str,
    config: &'a AppConfig,
}

impl<'a> Email<'a> {
    pub fn new(user_name: &'a str, user_email: &'a str, url: &'a str, config: &'a AppConfig) -> Self {
        let first_name = user_name.split_whitespace().next().unwrap_or(user_name).to_string();
        Self {
            to: user_email,
            first_name,
            url,
            config,
        }
    }

    pub async fn send_welcome(&self) -> Result<(), AppError> {
        let subject = "Welcome to the TravelAndTour Family!";
        let html = welcome_html(&self.first_name, self.url);
        let text = html_to_text(&html);
        self.send(subject, &html, &text).await
    }

    pub async fn send_password_reset(&self) -> Result<(), AppError> {
        let subject = "Your password reset token (valid for only 10 minutes)";
        let html = password_reset_html(&self.first_name, self.url);
        let text = html_to_text(&html);
        self.send(subject, &html, &text).await
    }

    async fn send(&self, subject: &str, html: &str, text: &str) -> Result<(), AppError> {
        let mailer = match build_mailer(self.config)? {
            Some(m) => m,
            None => {
                eprintln!(
                    "[email] transport not configured — skipping send to {} ({subject})",
                    self.to
                );
                return Ok(());
            }
        };

        let from = format!(
            "{} <{}>",
            self.config.email_from_name, self.config.email_from
        );
        let from: Mailbox = from
            .parse()
            .map_err(|e| AppError::internal(format!("Invalid EMAIL_FROM: {e}")))?;
        let to: Mailbox = self
            .to
            .parse()
            .map_err(|e| AppError::bad_request(format!("Invalid recipient email: {e}")))?;

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text.to_string()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html.to_string()),
                    ),
            )
            .map_err(|e| AppError::internal(e.to_string()))?;

        mailer
            .send(email)
            .await
            .map_err(|e| AppError::internal(format!("Could not send email: {e}")))?;

        Ok(())
    }
}

fn build_mailer(
    config: &AppConfig,
) -> Result<Option<AsyncSmtpTransport<Tokio1Executor>>, AppError> {
    if config.is_production() && !config.sendgrid_api_key.is_empty() {
        let creds = Credentials::new("apikey".to_string(), config.sendgrid_api_key.clone());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.sendgrid.net")
            .map_err(|e| AppError::internal(format!("SendGrid relay error: {e}")))?
            .port(587)
            .credentials(creds)
            .build();
        return Ok(Some(mailer));
    }

    if config.email_host.is_empty() {
        return Ok(None);
    }

    let creds = Credentials::new(
        config.email_username.clone(),
        config.email_password.clone(),
    );
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.email_host)
        .map_err(|e| AppError::internal(format!("SMTP relay error: {e}")))?
        .port(config.email_port)
        .credentials(creds)
        .build();
    Ok(Some(mailer))
}

fn html_to_text(html: &str) -> String {
    html.replace("<p>", "\n")
        .replace("</p>", "\n")
        .replace("<a ", "\n")
        .replace("</a>", "")
        .split(|c| c == '<' || c == '>')
        .filter(|s| !s.is_empty() && !s.starts_with("href"))
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string()
}

fn welcome_html(first_name: &str, url: &str) -> String {
    format!(
        r#"<p>Hi {first_name},</p><p>Welcome to TravelAndTour! We're excited to have you.</p>
        <p><a href="{url}">View your account</a></p>"#
    )
}

fn password_reset_html(first_name: &str, url: &str) -> String {
    format!(
        r#"<p>Hi {first_name},</p><p>Forgot your password? Reset it here (10 min):</p>
        <p><a href="{url}">{url}</a></p>"#
    )
}
