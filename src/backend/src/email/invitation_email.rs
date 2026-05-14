use chrono::{DateTime, Utc};

pub struct InvitationEmail {
    pub company_name: String,
    pub inviter_name: String,
    pub invited_role_label: String,
    pub project_name: Option<String>,
    pub accept_url: String,
    pub expires_at: DateTime<Utc>,
    pub message: Option<String>,
}

impl InvitationEmail {
    pub fn plain_body(&self) -> String {
        let mut out = String::new();
        out.push_str("You have been invited to join a team on HyperSynth.\n\n");
        out.push_str(&format!("Company: {}\n", self.company_name));
        out.push_str(&format!("Invited by: {}\n", self.inviter_name));
        out.push_str(&format!("Role: {}\n", self.invited_role_label));
        if let Some(ref name) = self.project_name {
            out.push_str(&format!("Project: {name}\n"));
        }
        out.push_str(&format!(
            "\nThis invitation expires on: {} UTC\n\n",
            self.expires_at.format("%Y-%m-%d %H:%M")
        ));
        out.push_str("Accept your invitation here:\n");
        out.push_str(&self.accept_url);
        out.push('\n');
        if let Some(ref msg) = self.message {
            let trimmed = msg.trim();
            if !trimmed.is_empty() {
                out.push_str("\nMessage from the inviter:\n");
                out.push_str(trimmed);
                out.push('\n');
            }
        }
        out
    }
}
