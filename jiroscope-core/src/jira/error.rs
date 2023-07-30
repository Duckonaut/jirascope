use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Schema for the error response from Jira.
///
/// Missing fields:
/// - `errors`: dynamic dictionary of error objects, not useful for us
///
/// From: https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/#status-codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCollection {
    #[serde(rename = "errorMessages")]
    pub error_messages: Vec<String>,
    #[serde(rename = "errors")]
    pub errors: JiraErrors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraErrors {
    pub description: Option<String>,
}

impl Display for ErrorCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(error) = &self.errors.description {
            write!(f, "{}", error)?;
            for error in &self.error_messages {
                write!(f, ", {}", error)?;
            }
            Ok(())
        } else if let Some(error) = self.error_messages.first() {
            write!(f, "{}", error)?;
            for error in &self.error_messages[1..] {
                write!(f, ", {}", error)?;
            }
            Ok(())
        } else {
            write!(f, "Unknown error")
        }
    }
}
