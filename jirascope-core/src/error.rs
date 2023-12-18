use std::fmt::{Display, Formatter};

use crate::jira;

#[derive(Debug)]
pub enum Error {
    Jirascope { message: String },
    Auth { message: String },
    Io(std::io::Error),
    Ureq(Box<ureq::Error>), // ureq::Error is Big
    Jira(u16, jira::ErrorCollection),
}

impl Error {
    pub fn jirascope(message: impl Into<String>) -> Error {
        Error::Jirascope {
            message: message.into(),
        }
    }

    pub fn auth(message: impl Into<String>) -> Error {
        Error::Auth {
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<ureq::Error> for Error {
    fn from(error: ureq::Error) -> Self {
        Error::Ureq(Box::new(error))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Jirascope { message } => write!(f, "Jirascope error: {}", message),
            Error::Auth { message } => write!(f, "Auth error: {}", message),
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Ureq(e) => write!(f, "Ureq error: {}", e),
            Error::Jira(code, e) => write!(f, "Jira error {}: {}", code, e),
        }
    }
}

impl std::error::Error for Error {}
