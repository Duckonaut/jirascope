use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ureq::serde_json::Value;

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
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "emailAddress")]
    pub email_address: String,
    #[serde(rename = "displayName")]
    pub display_name: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueEvent {
    pub id: isize,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCreationMeta {
    pub projects: Vec<ProjectIssueCreationMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIssueCreationMeta {
    pub id: String,
    pub key: String,
    pub name: String,
    #[serde(rename = "issuetypes")]
    pub issue_types: Vec<IssueType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueType {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "subtask")]
    pub is_subtask: bool,
    pub fields: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueEditMeta {
    pub fields: HashMap<String, IssueEditMetaField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueEditMetaField {
    pub required: bool,
    pub name: String,
    pub key: String,
    pub schema: MetaFieldSchema,
    pub allowed_values: Option<Vec<Value>>,
    pub operations: Option<Vec<String>>,
    pub has_default_value: Option<bool>,
    pub default_value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaFieldSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub system: Option<String>,
    pub items: Option<String>,
    pub custom: Option<String>,
    pub custom_id: Option<isize>,
}
