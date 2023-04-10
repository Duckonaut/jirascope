use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Jiroscope { message: String },
    Auth { message: String },
    Io(std::io::Error),
    Ureq(Box<ureq::Error>), // ureq::Error is Big
}

impl Error {
    pub fn jiroscope(message: impl Into<String>) -> Error {
        Error::Jiroscope {
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
            Error::Jiroscope { message } => write!(f, "Jiroscope error: {}", message),
            Error::Auth { message } => write!(f, "Auth error: {}", message),
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Ureq(e) => write!(f, "Ureq error: {}", e),
        }
    }
}

impl std::error::Error for Error {}
