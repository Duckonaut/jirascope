use jira::{
    CreatedIssue, IssueCreation, IssueCreationMeta, IssueEdit, IssueEditMeta, IssueEvent,
    IssueTransition, IssueTransitionDescriptor, IssueTransitionDescriptors, Issues, Paginated,
    ProjectCategory, ProjectCreate, ProjectCreated, ProjectDetailed, ProjectEdit,
    ProjectIssueSecurityScheme, User,
};
use serde::Serialize;

mod auth;
mod config;
mod error;
pub mod jira;
mod utils;

pub use auth::Auth;
pub use config::Config;
pub use error::Error;

pub use ureq;

use crate::jira::{FieldConfigurationScheme, Issue};

pub struct Jirascope {
    config: Config,
    pub auth: Auth,
}

#[cfg(feature = "test_server")]
#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: Option<usize>,
    pub message: String,
}

impl Jirascope {
    pub fn new(config: Config, auth: Auth) -> Jirascope {
        Jirascope { config, auth }
    }

    pub fn init(&mut self) -> Result<(), crate::Error> {
        self.auth.login(&self.config)?;

        Ok(())
    }

    #[cfg(feature = "test_server")]
    pub fn register_note(&self, message: String) -> Result<Note, crate::Error> {
        let note = Note { id: None, message };

        let response = ureq::post("http://localhost:1937/notes").send_json(note)?;

        let note: Note = response.into_json()?;

        Ok(note)
    }

    #[cfg(feature = "test_server")]
    pub fn get_notes(&self) -> Result<Vec<Note>, crate::Error> {
        let response = ureq::get("http://localhost:1937/notes").call()?;

        let notes: Vec<Note> = response.into_json()?;

        Ok(notes)
    }

    #[cfg(feature = "test_server")]
    pub fn get_note_by_id(&self, id: usize) -> Result<Note, crate::Error> {
        let response = ureq::get(&format!("http://localhost:1937/notes/{}", id)).call()?;

        let note: Note = response.into_json()?;

        Ok(note)
    }

    #[cfg(feature = "test_server")]
    pub fn update_note_by_id(&self, id: usize, message: String) -> Result<Note, crate::Error> {
        let note = Note {
            id: Some(id),
            message,
        };

        let response = ureq::put(&format!("http://localhost:1937/notes/{}", id)).send_json(note)?;

        let note: Note = response.into_json()?;

        Ok(note)
    }

    pub fn get_users(&mut self) -> Result<Vec<User>, crate::Error> {
        let users: Vec<User> = self.api_get("users/search")?.into_json()?;

        Ok(users)
    }

    pub fn get_projects(&mut self) -> Result<Vec<ProjectDetailed>, crate::Error> {
        let response = self.api_get("project?expand=description,lead,url")?;

        let projects: Vec<ProjectDetailed> = response.into_json()?;

        Ok(projects)
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

    pub fn get_issue_events(&mut self) -> Result<Vec<IssueEvent>, crate::Error> {
        let response = self.api_get("events")?;

        let issue_events: Vec<IssueEvent> = response.into_json()?;

        Ok(issue_events)
    }

    pub fn get_issue_creation_meta(&mut self) -> Result<IssueCreationMeta, crate::Error> {
        let response = self.api_get("issue/createmeta")?;

        let issue_events: IssueCreationMeta = response.into_json()?;

        Ok(issue_events)
    }

    pub fn get_issue_edit_meta<'a>(
        &mut self,
        issue_id: impl Into<&'a str>,
    ) -> Result<IssueEditMeta, crate::Error> {
        let response = self.api_get(format!("issue/{}/editmeta", issue_id.into()).as_str())?;

        let issue_events: IssueEditMeta = response.into_json()?;

        Ok(issue_events)
    }

    pub fn create_issue(&mut self, issue: IssueCreation) -> Result<CreatedIssue, crate::Error> {
        println!(
            "Creating issue: {}",
            ureq::serde_json::to_string(&issue).unwrap()
        );
        let response = self.api_post("issue", issue)?;

        let created_issue: CreatedIssue = response.into_json()?;

        Ok(created_issue)
    }

    pub fn edit_issue<'a>(
        &mut self,
        issue_id: impl Into<&'a str>,
        issue: IssueEdit,
    ) -> Result<(), crate::Error> {
        self.api_put(format!("issue/{}", issue_id.into()).as_str(), issue)?;

        Ok(())
    }

    pub fn delete_issue<'a>(&mut self, issue_id: impl Into<&'a str>) -> Result<(), crate::Error> {
        self.api_delete(format!("issue/{}", issue_id.into()).as_str())?;

        Ok(())
    }

    pub fn get_issue_transitions(
        &mut self,
        issue_id: &str,
    ) -> Result<IssueTransitionDescriptors, crate::Error> {
        let response = self.api_get(format!("issue/{}/transitions", issue_id).as_str())?;

        let issue_transitions: IssueTransitionDescriptors = response.into_json()?;

        Ok(issue_transitions)
    }

    pub fn transition_issue(
        &mut self,
        issue_id: &str,
        transition: IssueTransitionDescriptor,
    ) -> Result<(), crate::Error> {
        let transition = IssueTransition { transition };
        self.api_post(
            format!("issue/{}/transitions", issue_id).as_str(),
            transition,
        )?;

        Ok(())
    }

    pub fn create_project(
        &mut self,
        project: ProjectCreate,
    ) -> Result<ProjectCreated, crate::Error> {
        let response = self.api_post("project", project)?;

        let new_project: ProjectCreated = response.into_json()?;

        Ok(new_project)
    }

    pub fn edit_project<'a>(
        &mut self,
        project_id: impl Into<&'a str>,
        project: ProjectEdit,
    ) -> Result<(), crate::Error> {
        self.api_put(format!("project/{}", project_id.into()).as_str(), project)?;

        Ok(())
    }

    pub fn delete_project<'a>(
        &mut self,
        project_id: impl Into<&'a str>,
    ) -> Result<(), crate::Error> {
        self.api_delete(format!("project/{}", project_id.into()).as_str())?;

        Ok(())
    }

    pub fn get_project_categories(&mut self) -> Result<Vec<ProjectCategory>, crate::Error> {
        let response = self.api_get("projectCategory")?;

        let project_categories: Vec<ProjectCategory> = response.into_json()?;

        Ok(project_categories)
    }

    pub fn get_issue_security_schemes(
        &mut self,
    ) -> Result<Vec<ProjectIssueSecurityScheme>, crate::Error> {
        let response = self.api_get("issuesecurityschemes")?;

        let issue_security_schemes: Vec<ProjectIssueSecurityScheme> = response.into_json()?;

        Ok(issue_security_schemes)
    }

    pub fn get_field_configuration_schemes(
        &mut self,
    ) -> Result<Vec<FieldConfigurationScheme>, crate::Error> {
        let field_configuration_schemes: Vec<FieldConfigurationScheme> =
            self.api_get_depaginated("fieldconfigurationscheme")?;

        Ok(field_configuration_schemes)
    }

    fn api_get(&mut self, path: &str) -> Result<ureq::Response, crate::Error> {
        match self
            .auth
            .auth(ureq::get(
                format!("{}/rest/api/3/{}", &self.config.api_url, path).as_str(),
            ))
            .call()
        {
            Ok(response) => Ok(response),
            Err(error) => match error {
                ureq::Error::Status(code, response) => {
                    Err(crate::Error::Jira(code, response.into_json()?))
                }
                ureq::Error::Transport(e) => {
                    Err(crate::Error::Ureq(Box::new(ureq::Error::Transport(e))))
                }
            },
        }
    }

    fn api_get_depaginated<T>(&mut self, path: &str) -> Result<Vec<T>, crate::Error>
    where
        for<'a> T: serde::Deserialize<'a>,
    {
        let mut results: Vec<T> = Vec::new();
        let mut start_at = 0;
        let max_results = 50;

        loop {
            let response = self.api_get(
                format!("{}?startAt={}&maxResults={}", path, start_at, max_results).as_str(),
            )?;

            let paginated: Paginated<T> = response.into_json()?;

            results.extend(paginated.values);

            if paginated.is_last {
                break;
            }

            start_at += max_results;
        }

        Ok(results)
    }

    fn api_post(
        &mut self,
        path: &str,
        body: impl Serialize,
    ) -> Result<ureq::Response, crate::Error> {
        match self
            .auth
            .auth(ureq::post(
                format!("{}/rest/api/3/{}", &self.config.api_url, path).as_str(),
            ))
            .send_json(body)
        {
            Ok(response) => Ok(response),
            Err(error) => match error {
                ureq::Error::Status(code, response) => {
                    Err(crate::Error::Jira(code, response.into_json()?))
                }
                ureq::Error::Transport(e) => {
                    Err(crate::Error::Ureq(Box::new(ureq::Error::Transport(e))))
                }
            },
        }
    }

    fn api_delete(&mut self, path: &str) -> Result<ureq::Response, crate::Error> {
        match self
            .auth
            .auth(ureq::delete(
                format!("{}/rest/api/3/{}", &self.config.api_url, path).as_str(),
            ))
            .call()
        {
            Ok(response) => Ok(response),
            Err(error) => match error {
                ureq::Error::Status(code, response) => {
                    Err(crate::Error::Jira(code, response.into_json()?))
                }
                ureq::Error::Transport(e) => {
                    Err(crate::Error::Ureq(Box::new(ureq::Error::Transport(e))))
                }
            },
        }
    }

    fn api_put(
        &mut self,
        path: &str,
        body: impl Serialize,
    ) -> Result<ureq::Response, crate::Error> {
        match self
            .auth
            .auth(ureq::put(
                format!("{}/rest/api/3/{}", &self.config.api_url, path).as_str(),
            ))
            .send_json(body)
        {
            Ok(response) => Ok(response),
            Err(error) => match error {
                ureq::Error::Status(code, response) => {
                    Err(crate::Error::Jira(code, response.into_json()?))
                }
                ureq::Error::Transport(e) => {
                    Err(crate::Error::Ureq(Box::new(ureq::Error::Transport(e))))
                }
            },
        }
    }
}
