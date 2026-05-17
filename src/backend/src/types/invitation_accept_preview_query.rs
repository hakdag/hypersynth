use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct InvitationAcceptPreviewQuery {
    pub token: String,
}
