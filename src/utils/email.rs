//! Transactional email (Natours `utils/newemail.js`) — SMTP via Mailtrap / env.

use lettre::message::header::ContentType;
use lettre::message::Mailbox;
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
        let subject = "Welcome to the Natours Family!";
        let html = welcome_html(&self.first_name, self.url);
        self.send(subject, &html).await
    }

    pub async fn send_password_reset(&self) -> Result<(), AppError> {
        let subject = "Your password reset token (valid for only 10 minutes)";
        let html = password_reset_html(&self.first_name, self.url);
        self.send(subject, &html).await
    }

    async fn send(&self, subject: &str, html: &str) -> Result<(), AppError> {
        if self.config.email_host.is_empty() {
            eprintln!(
                "[email] EMAIL_HOST not set — skipping send to {} ({subject})",
                self.to
            );
            return Ok(());
        }

        let from = format!(
            "{} <{}>",
            self.config.email_from_name, self.config.email_from
        );
        let from: Mailbox = from.parse().map_err(|e| {
            AppError::internal(format!("Invalid EMAIL_FROM: {e}"))
        })?;
        let to: Mailbox = self.to.parse().map_err(|e| {
            AppError::bad_request(format!("Invalid recipient email: {e}"))
        })?;

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html.to_string())
            .map_err(|e| AppError::internal(e.to_string()))?;

        let creds = Credentials::new(
            self.config.email_username.clone(),
            self.config.email_password.clone(),
        );

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.email_host)
            .map_err(|e| AppError::internal(format!("SMTP relay error: {e}")))?
            .port(self.config.email_port)
            .credentials(creds)
            .build();

        mailer
            .send(email)
            .await
            .map_err(|e| AppError::internal(format!("Could not send email: {e}")))?;

        Ok(())
    }
}

fn welcome_html(first_name: &str, url: &str) -> String {
    format!(
        r#"<p>Hi {first_name},</p><p>Welcome to Natours! We're excited to have you.</p>
        <p><a href="{url}">View your account</a></p>"#
    )
}

fn password_reset_html(first_name: &str, url: &str) -> String {
    format!(
        r#"<p>Hi {first_name},</p><p>Forgot your password? Reset it here (10 min):</p>
        <p><a href="{url}">{url}</a></p>"#
    )
}
