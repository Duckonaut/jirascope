use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ureq::serde_json::Value;

use super::{Project, AtlassianDoc, User};

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
    pub description: Option<AtlassianDoc>,
    pub priority: Priority,
    pub status: Status,
    pub assignee: Option<User>,
    pub reporter: User,
    pub created: String, // ISO 8601 date/time string
    pub updated: String, // ISO 8601 date/time string
    pub project: Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCreation {
    pub fields: IssueCreationFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCreationFields {
    pub project: Project,
    #[serde(rename = "issuetype")]
    pub issue_type: IssueType,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<AtlassianDoc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedIssue {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_link: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueEdit {
    pub fields: IssueEditFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueEditFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<AtlassianDoc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<Option<User>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Priority {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub id: String,
    pub name: String,
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
    pub id: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTransitionDescriptor {
    pub id: String,
    pub name: String,
    pub to: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTransitionDescriptors {
    pub transitions: Vec<IssueTransitionDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTransition {
    pub transition: IssueTransitionDescriptor,
}

