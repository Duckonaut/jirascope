use base64::Engine;
use ureq::Request;

use crate::Config;

pub struct Auth {
    pub username: String,
    pub api_token: String,
    pub cached_basic_auth: Option<String>,
}

impl Auth {
    pub fn new(username: impl Into<String>, api_token: impl Into<String>) -> Auth {
        Auth {
            username: username.into(),
            api_token: api_token.into(),
            cached_basic_auth: None,
        }
    }

    pub fn login(&mut self, _config: &Config) -> Result<(), crate::Error> {
        self.ensure_cached_basic_auth();

        Ok(())
    }

    fn ensure_cached_basic_auth(&mut self) {
        if self.cached_basic_auth.is_none() {
            let base64 = base64::engine::GeneralPurpose::new(&base64::alphabet::STANDARD, base64::engine::GeneralPurposeConfig::default());
            let basic_auth = base64.encode(format!("{}:{}", self.username, self.api_token));
            self.cached_basic_auth = Some(basic_auth);
        }
    }

    fn get_basic_auth(&mut self) -> String {
        self.ensure_cached_basic_auth();

        self.cached_basic_auth.as_ref().unwrap().clone()
    }

    pub fn auth(&mut self, request: Request) -> Request {
        request.set("Authorization", format!("Basic {}", self.get_basic_auth()).as_str())
    }
}
