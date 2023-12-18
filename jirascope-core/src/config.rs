#[derive(Debug, Default)]
pub struct Config {
    pub api_url: String,
}

impl Config {
    pub fn new(jira_url: impl Into<String>) -> Config {
        Config {
            api_url: jira_url.into(),
        }
    }
}
