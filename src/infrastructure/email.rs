//! Email service (SMTP via lettre) mirroring the Go `infrastructure/email` package.

use async_trait::async_trait;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::error::{AppError, AppResult};

#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send_otp(&self, to: &str, code: &str, purpose: &str) -> AppResult<()>;
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> AppResult<()>;
}

/// Basic email validation, mirroring Go's `ValidateEmail`.
pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

fn otp_body(code: &str, purpose: &str) -> String {
    let purpose_text = match purpose {
        "email_verification" => "email verification",
        "password_reset" => "password reset",
        "login" => "login verification",
        _ => "verification",
    };

    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
	<style>
		body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
		.container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
		.header {{ background-color: #4CAF50; color: white; padding: 20px; text-align: center; border-radius: 5px 5px 0 0; }}
		.content {{ background-color: #f9f9f9; padding: 30px; border: 1px solid #ddd; }}
		.otp-code {{ font-size: 32px; font-weight: bold; color: #4CAF50; text-align: center; padding: 20px; background-color: #fff; border: 2px dashed #4CAF50; border-radius: 5px; margin: 20px 0; letter-spacing: 5px; }}
		.footer {{ text-align: center; padding: 20px; font-size: 12px; color: #777; }}
		.warning {{ color: #f44336; font-size: 14px; margin-top: 20px; }}
	</style>
</head>
<body>
	<div class="container">
		<div class="header">
			<h1>Verification Code</h1>
		</div>
		<div class="content">
			<h2>Hello!</h2>
			<p>You have requested a verification code for <strong>{purpose_text}</strong>.</p>
			<p>Please use the following code to complete your verification:</p>
			<div class="otp-code">{code}</div>
			<p>This code will expire in <strong>10 minutes</strong>.</p>
			<p class="warning">If you did not request this code, please ignore this email and ensure your account is secure.</p>
		</div>
		<div class="footer">
			<p>This is an automated email. Please do not reply.</p>
			<p>&copy; 2025 Ezra. All rights reserved.</p>
		</div>
	</div>
</body>
</html>
"#
    )
}

pub struct SmtpEmailService {
    host: String,
    port: u16,
    username: String,
    password: String,
    from: String,
    secure: String,
}

impl SmtpEmailService {
    pub fn new(host: String, port: String, username: String, password: String, from: String, secure: String) -> Self {
        let port = port.parse::<u16>().unwrap_or(587);
        SmtpEmailService { host, port, username, password, from, secure }
    }

    fn build_transport(&self) -> AppResult<AsyncSmtpTransport<Tokio1Executor>> {
        let creds = Credentials::new(self.username.clone(), self.password.clone());
        let secure = self.secure.trim().to_lowercase();
        let secure = if secure.is_empty() {
            if self.port == 465 {
                "ssl".to_string()
            } else {
                "starttls".to_string()
            }
        } else {
            secure
        };

        let builder = match secure.as_str() {
            "ssl" => AsyncSmtpTransport::<Tokio1Executor>::relay(&self.host)
                .map_err(|e| AppError::Internal(e.to_string()))?,
            "plain" => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.host),
            _ => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.host)
                .map_err(|e| AppError::Internal(e.to_string()))?,
        };

        Ok(builder.port(self.port).credentials(creds).build())
    }
}

#[async_trait]
impl EmailService for SmtpEmailService {
    async fn send_otp(&self, to: &str, code: &str, purpose: &str) -> AppResult<()> {
        self.send_email(to, "Your Verification Code", &otp_body(code, purpose))
            .await
    }

    async fn send_email(&self, to: &str, subject: &str, body: &str) -> AppResult<()> {
        let message = Message::builder()
            .from(self.from.parse().map_err(|_| AppError::Internal("invalid from address".to_string()))?)
            .to(to.parse().map_err(|_| AppError::BadRequest("invalid recipient address".to_string()))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body.to_string())
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let transport = self.build_transport()?;
        transport
            .send(message)
            .await
            .map_err(|e| AppError::Internal(format!("failed to send email: {e}")))?;
        Ok(())
    }
}

pub struct DummyEmailService;

#[async_trait]
impl EmailService for DummyEmailService {
    async fn send_otp(&self, to: &str, code: &str, purpose: &str) -> AppResult<()> {
        tracing::info!("[DUMMY EMAIL] Would send OTP {code} for {purpose} to {to}");
        Ok(())
    }

    async fn send_email(&self, to: &str, subject: &str, _body: &str) -> AppResult<()> {
        tracing::info!("[DUMMY EMAIL] Would send email to {to} with subject {subject:?}");
        Ok(())
    }
}
