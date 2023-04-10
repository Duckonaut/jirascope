use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    id: String,
    key: String,
    fields: IssueFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IssueFields {
    summary: String,
    description: Option<String>,
    priority: Priority,
    status: Status,
    assignee: Option<User>,
    reporter: User,
    created: String, // ISO 8601 date/time string
    updated: String, // ISO 8601 date/time string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Priority {
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Status {
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    #[serde(rename = "emailAddress")]
    email_address: String,
    #[serde(rename = "displayName")]
    display_name: String
}
