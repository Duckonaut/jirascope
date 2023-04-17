use serde::{Deserialize, Serialize};

use super::Project;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issues {
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub key: String,
    pub fields: IssueFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueFields {
    pub summary: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub status: Status,
    pub assignee: Option<User>,
    pub reporter: User,
    pub created: String, // ISO 8601 date/time string
    pub updated: String, // ISO 8601 date/time string
    pub project: Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Priority {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "emailAddress")]
    pub email_address: String,
    #[serde(rename = "displayName")]
    pub display_name: String
}
