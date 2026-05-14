use std::fmt;

#[derive(Debug)]
pub enum EmailError {
    MessageBuild(String),
    Transport(String),
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailError::MessageBuild(s) | EmailError::Transport(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for EmailError {}
