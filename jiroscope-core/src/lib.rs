use jira::Issues;
use serde::{Deserialize, Serialize};
use ureq::post;

mod auth;
mod config;
mod error;
pub mod jira;

pub use auth::Auth;
pub use config::Config;
pub use error::Error;

use crate::jira::Issue;

// Empty struct for now. Will almost certainly be expanded later to hold state like tokens and
// cached data.
pub struct Jiroscope {
    config: Config,
    auth: Auth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: Option<usize>,
    pub message: String,
}

impl Jiroscope {
    pub fn new(config: Config, auth: Auth) -> Jiroscope {
        Jiroscope { config, auth }
    }

    pub fn init(&mut self) -> Result<(), crate::Error> {
        self.auth.login(&self.config)?;

        Ok(())
    }

    pub fn register_note(&self, message: String) -> Result<Note, crate::Error> {
        let note = Note { id: None, message };

        let time = std::time::Instant::now();
        let response = post("http://localhost:1937/notes").send_json(note)?;
        println!("Time to register note: {:?}", time.elapsed());

        let note: Note = response.into_json()?;

        Ok(note)
    }

    pub fn get_notes(&self) -> Result<Vec<Note>, crate::Error> {
        let response = ureq::get("http://localhost:1937/notes").call()?;

        let notes: Vec<Note> = response.into_json()?;

        Ok(notes)
    }

    pub fn get_note_by_id(&self, id: usize) -> Result<Note, crate::Error> {
        let response = ureq::get(&format!("http://localhost:1937/notes/{}", id)).call()?;

        let note: Note = response.into_json()?;

        Ok(note)
    }

    pub fn update_note_by_id(&self, id: usize, message: String) -> Result<Note, crate::Error> {
        let note = Note {
            id: Some(id),
            message,
        };

        let response = ureq::put(&format!("http://localhost:1937/notes/{}", id)).send_json(note)?;

        let note: Note = response.into_json()?;

        Ok(note)
    }

    pub fn get_issue<'a>(&mut self, issue_id: impl Into<&'a str>) -> Result<Issue, crate::Error> {
        let response = self.api_get(format!("issue/{}", issue_id.into()).as_str())?;

        let issue = response.into_json()?;

        Ok(issue)
    }

    pub fn get_all_issues(&mut self) -> Result<Issues, crate::Error> {
        let response = self.api_get("search")?;

        let issues: Issues = response.into_json()?;

        Ok(issues)
    }

    fn api_get(&mut self, path: &str) -> Result<ureq::Response, crate::Error> {
        let response = self
            .auth
            .auth(ureq::get(
                format!("{}/rest/api/3/{}", &self.config.api_url, path).as_str(),
            ))
            .call()?;

        Ok(response)
    }
}
