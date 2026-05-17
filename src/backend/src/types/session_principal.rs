use crate::types::SessionUser;

#[derive(Debug)]
pub enum SessionPrincipal {
    User(SessionUser),
    SystemAdmin { email: String },
}
