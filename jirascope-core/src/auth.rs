use base64::Engine;
use ureq::Request;

use crate::Config;

pub struct Auth {
    username: String,
    api_token: String,
    cached_basic_auth: Option<String>,
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

    pub fn get_basic_auth(&mut self) -> String {
        self.ensure_cached_basic_auth();

        self.cached_basic_auth.as_ref().unwrap().clone()
    }

    pub fn auth(&mut self, request: Request) -> Request {
        request.set("Authorization", format!("Basic {}", self.get_basic_auth()).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_auth() {
        let mut auth = Auth::new("username", "api_token");
        assert_eq!(auth.get_basic_auth(), "dXNlcm5hbWU6YXBpX3Rva2Vu");
    }

    #[test]
    fn auth() {
        let mut auth = Auth::new("username", "api_token");
        let request = ureq::get("https://example.com");
        let request = auth.auth(request);
        assert_eq!(request.header("Authorization").unwrap(), "Basic dXNlcm5hbWU6YXBpX3Rva2Vu");
    }

    #[test]
    fn auth_with_login() {
        let mut auth = Auth::new("username", "api_token");
        let config = Config::new("https://example.atlassian.net");
        auth.login(&config).unwrap();

        let request = ureq::get("https://example.com");
        let request = auth.auth(request);
        assert_eq!(request.header("Authorization").unwrap(), "Basic dXNlcm5hbWU6YXBpX3Rva2Vu");
    }
}
