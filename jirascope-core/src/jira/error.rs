use std::{collections::HashMap, fmt::Display};

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
    pub errors: HashMap<String, String>,
}

impl Display for ErrorCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut errors = self
            .error_messages
            .iter()
            .cloned()
            .chain(self.errors.iter().map(|(k, v)| format!("{}: {}", k, v)));

        if let Some(error) = errors.next() {
            write!(f, "{}", error)?;
            for error in errors {
                write!(f, ", {}", error)?;
            }
            Ok(())
        } else {
            write!(f, "Unknown error")
        }
    }
}
