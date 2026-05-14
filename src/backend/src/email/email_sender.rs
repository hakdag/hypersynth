use async_trait::async_trait;

use crate::email::{EmailError, InvitationEmail};

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_invitation(&self, to: &str, payload: InvitationEmail) -> Result<(), EmailError>;
}
