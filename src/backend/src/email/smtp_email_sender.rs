use async_trait::async_trait;
use lettre::{
    message::{header::ContentType, Mailbox, Message},
    transport::smtp::authentication::Credentials,
    transport::smtp::client::Tls,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};

use crate::configs::SmtpConfig;
use crate::email::{EmailError, EmailSender, InvitationEmail};

pub struct SmtpEmailSender {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_name: String,
    from_email: String,
}

impl SmtpEmailSender {
    pub fn try_new(cfg: &SmtpConfig) -> Result<Self, String> {
        let creds = Credentials::new(cfg.username.to_string(), cfg.password.to_string());
        // `relay()` uses implicit TLS (SMTPS). MailHog and similar dev relays use plain SMTP on
        // 1025 — use `Tls::None` when STARTTLS is disabled.
        let transport = if cfg.starttls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(cfg.host.as_str())
                .map_err(|e| format!("SMTP STARTTLS relay setup failed: {e}"))?
                .credentials(creds)
                .port(cfg.port)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(cfg.host.as_str())
                .port(cfg.port)
                .tls(Tls::None)
                .credentials(creds)
                .build()
        };
        Ok(Self {
            transport,
            from_name: cfg.from_name.to_string(),
            from_email: cfg.from_email.to_string(),
        })
    }
}

#[async_trait]
impl EmailSender for SmtpEmailSender {
    async fn send_invitation(&self, to: &str, payload: InvitationEmail) -> Result<(), EmailError> {
        let from_line = format!("{} <{}>", self.from_name, self.from_email);
        let from: Mailbox = from_line
            .parse()
            .map_err(|e| EmailError::MessageBuild(format!("invalid from mailbox: {e}")))?;
        let to_mailbox: Mailbox = to
            .parse()
            .map_err(|e| EmailError::MessageBuild(format!("invalid to mailbox: {e}")))?;

        let subject = format!("Invitation to join {}", payload.company_name);
        let body = payload.plain_body();

        let email = Message::builder()
            .from(from)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)
            .map_err(|e| EmailError::MessageBuild(format!("build message: {e}")))?;

        self.transport
            .send(email)
            .await
            .map_err(|e| EmailError::Transport(format!("{e}")))?;

        Ok(())
    }
}
