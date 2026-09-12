//! Clean, asynchronous SMTP email dispatch service (Gmail / standard SMTP) with local dev fallback.

use super::template::{otp_email_html, otp_email_text};
use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub from: String,
}

#[derive(Debug, Clone)]
pub struct EmailService {
    smtp_config: Option<SmtpConfig>,
}

impl EmailService {
    /// Initializes email service by reading SMTP environment variables.
    ///
    /// Supports:
    /// - `GMAIL_USER` and `GMAIL_APP_PASSWORD` (defaults to smtp.gmail.com:465)
    /// - `SMTP_USER`, `SMTP_PASS`, `SMTP_HOST`, `SMTP_PORT`, `SMTP_FROM`
    pub fn from_env() -> Self {
        let smtp_user = std::env::var("GMAIL_USER")
            .or_else(|_| std::env::var("SMTP_USER"))
            .ok()
            .filter(|s| !s.trim().is_empty());

        let smtp_pass = std::env::var("GMAIL_APP_PASSWORD")
            .or_else(|_| std::env::var("SMTP_PASS"))
            .ok()
            .filter(|s| !s.trim().is_empty());

        let smtp_config = if let (Some(user), Some(pass)) = (smtp_user, smtp_pass) {
            let host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
            let port = std::env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(465);
            let from =
                std::env::var("SMTP_FROM").unwrap_or_else(|_| format!("Sealed Books <{}>", user));

            tracing::info!(
                "Configured SMTP email delivery via {} (user: {})",
                host,
                user
            );

            Some(SmtpConfig {
                host,
                port,
                user,
                pass,
                from,
            })
        } else {
            None
        };

        Self { smtp_config }
    }

    /// Dispatches a 6-digit OTP verification email via SMTP.
    /// If no SMTP credentials are configured, falls back to logging the OTP to the terminal.
    pub async fn send_otp(&self, to_email: &str, code: &str) -> Result<(), String> {
        let subject = format!("Your Sealed Books Security Code: {}", code);
        let html_content = otp_email_html(code);
        let text_content = otp_email_text(code);

        if let Some(ref smtp) = self.smtp_config {
            tracing::info!(
                "Dispatching OTP email via SMTP ({}) to {}",
                smtp.host,
                to_email
            );

            let email_msg = Message::builder()
                .from(
                    smtp.from
                        .parse()
                        .map_err(|e| format!("Invalid 'from' address: {e}"))?,
                )
                .to(to_email
                    .parse()
                    .map_err(|e| format!("Invalid 'to' address: {e}"))?)
                .subject(&subject)
                .multipart(
                    MultiPart::alternative()
                        .singlepart(SinglePart::plain(text_content))
                        .singlepart(SinglePart::html(html_content)),
                )
                .map_err(|e| format!("Failed to construct email: {e}"))?;

            let mailer: AsyncSmtpTransport<Tokio1Executor> =
                AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp.host)
                    .map_err(|e| format!("Failed to resolve SMTP host: {e}"))?
                    .credentials(Credentials::new(smtp.user.clone(), smtp.pass.clone()))
                    .port(smtp.port)
                    .build();

            mailer
                .send(email_msg)
                .await
                .map_err(|e| format!("SMTP send failed: {e}"))?;

            tracing::info!("OTP email successfully sent via SMTP to {}", to_email);
            Ok(())
        } else {
            // Local dev fallback when no SMTP credentials are provided
            tracing::info!(
                "[AUTH] (Dev Fallback) Verification OTP for {}: {}",
                to_email,
                code
            );
            Ok(())
        }
    }
}

impl Default for EmailService {
    fn default() -> Self {
        Self::from_env()
    }
}
