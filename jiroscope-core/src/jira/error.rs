use serde::{Serialize, Deserialize};

/// Schema for the error response from Jira.
///
/// Missing fields:
/// - `errors`: dynamic dictionary of error objects, not useful for us
///
/// From: https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/#status-codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCollection {
    #[serde(rename = "errorMessages")]
    error_messages: Vec<String>,
    status: isize,
}
