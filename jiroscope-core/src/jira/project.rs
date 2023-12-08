use serde::{Deserialize, Serialize};

use super::User;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    // id is sometimes a string, sometimes a number.
    // if it's a string, parse to i64,
    // if it's a number, parse to i64,
    #[serde(deserialize_with = "crate::utils::deserialize_id")]
    pub id: i64,
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectDetailed {
    // id is sometimes a string, sometimes a number.
    // if it's a string, parse to i64,
    // if it's a number, parse to i64,
    #[serde(deserialize_with = "crate::utils::deserialize_id")]
    pub id: i64,
    pub key: String,
    pub name: String,
    pub description: String,
    pub lead: User,
    pub url: Option<String>,
}

impl ProjectDetailed {
    pub fn to_project(self) -> Project {
        Project {
            id: self.id,
            key: self.key,
            name: self.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCreated {
    pub id: i64,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectList {
    pub projects: Vec<Project>,
}

pub const PROJECT_TYPE_KEYS: [&str; 3] = ["software", "business", "service_desk"];

pub const PROJECT_TYPE_NAMES_TO_TEMPLATE_RANGE: [(&str, usize, usize); 3] = [
    ("business", 0, 8),
    ("service_desk", 8, 32),
    ("software", 32, 37),
];

pub const PROJECT_TEMPLATES: [ProjectTemplate; 37] = [
    ProjectTemplate {
        id: "com.atlassian.jira-core-project-templates:jira-core-simplified-content-management",
        description: "Simplified content management",
    },
    ProjectTemplate {
        id: "com.atlassian.jira-core-project-templates:jira-core-simplified-document-approval",
        description: "Simplified document approval",
    },
    ProjectTemplate {
        id: "com.atlassian.jira-core-project-templates:jira-core-simplified-lead-tracking",
        description: "Simplified lead tracking",
    },
    ProjectTemplate {
        id: "com.atlassian.jira-core-project-templates:jira-core-simplified-process-control",
        description: "Simplified process control",
    },
    ProjectTemplate {
        id: "com.atlassian.jira-core-project-templates:jira-core-simplified-procurement",
        description: "Simplified procurement",
    },
    ProjectTemplate {
        id: "com.atlassian.jira-core-project-templates:jira-core-simplified-project-management",
        description: "Simplified project management",
    },
    ProjectTemplate {
        id: "com.atlassian.jira-core-project-templates:jira-core-simplified-recruitment",
        description: "Simplified recruitment",
    },
    ProjectTemplate {
        id: "com.atlassian.jira-core-project-templates:jira-core-simplified-task-tracking",
        description: "Simplified task tracking",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:simplified-it-service-management",
        description: "Simplified IT service management",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:simplified-general-service-desk-it",
        description: "Simplified general service desk (IT)",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:simplified-general-service-desk-business",
        description: "Simplified general service desk (Business)",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:simplified-external-service-desk",
        description: "Simplified external service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:simplified-hr-service-desk",
        description: "Simplified HR service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:simplified-facilities-service-desk",
        description: "Simplified facilities service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:simplified-legal-service-desk",
        description: "Simplified legal service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:simplified-analytics-service-desk",
        description: "Simplified analytics service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:simplified-marketing-service-desk",
        description: "Simplified marketing service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:simplified-design-service-desk",
        description: "Simplified design service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:simplified-sales-service-desk",
        description: "Simplified sales service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:simplified-finance-service-desk",
        description: "Simplified finance service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:next-gen-it-service-desk",
        description: "Next-gen IT service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:next-gen-hr-service-desk",
        description: "Next-gen HR service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:next-gen-legal-service-desk",
        description: "Next-gen legal service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:next-gen-marketing-service-desk",
        description: "Next-gen marketing service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:next-gen-facilities-service-desk",
        description: "Next-gen facilities service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:next-gen-general-service-desk",
        description: "Next-gen general service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:next-gen-general-it-service-desk",
        description: "Next-gen general service desk (IT)",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:next-gen-general-business-service-desk",
        description: "Next-gen general service desk (Business)",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:next-gen-analytics-service-desk",
        description: "Next-gen analytics service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:next-gen-finance-service-desk",
        description: "Next-gen finance service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:next-gen-design-service-desk",
        description: "Next-gen design service desk",
    },
    ProjectTemplate {
        id: "com.atlassian.servicedesk:next-gen-sales-service-desk",
        description: "Next-gen sales service desk",
    },
    ProjectTemplate {
        id: "com.pyxis.greenhopper.jira:gh-simplified-agility-kanban",
        description: "Agile Kanban",
    },
    ProjectTemplate {
        id: "com.pyxis.greenhopper.jira:gh-simplified-agility-scrum",
        description: "Agile Scrum",
    },
    ProjectTemplate {
        id: "com.pyxis.greenhopper.jira:gh-simplified-basic",
        description: "Basic",
    },
    ProjectTemplate {
        id: "com.pyxis.greenhopper.jira:gh-simplified-kanban-classic",
        description: "Kanban Classic",
    },
    ProjectTemplate {
        id: "com.pyxis.greenhopper.jira:gh-simplified-scrum-classic",
        description: "Scrum Classic",
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCreate {
    pub key: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "leadAccountId")]
    pub lead_account_id: String,
    #[serde(rename = "categoryId")]
    pub category_id: Option<i64>,
    #[serde(rename = "projectTypeKey")]
    pub project_type_key: String,
    #[serde(rename = "assigneeType")]
    pub assignee_type: AssigneeType,
    #[serde(flatten)]
    pub details: ProjectCreateDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProjectCreateDetails {
    Template {
        #[serde(rename = "projectTemplateKey")]
        project_template_key: String,
    },
    Granular {
        #[serde(rename = "issueSecurityScheme")]
        issue_security_scheme: i64,
        #[serde(rename = "permissionScheme")]
        permission_scheme: i64,
        #[serde(rename = "notificationScheme")]
        notification_scheme: i64,
        #[serde(rename = "issueTypeScheme")]
        issue_type_scheme: i64,
        #[serde(rename = "issueTypeScreenScheme")]
        issue_type_screen_scheme: i64,
        #[serde(rename = "fieldConfigurationScheme")]
        field_configuration_scheme: i64,
        #[serde(rename = "workflowScheme")]
        workflow_scheme: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectEdit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Option<String>>,
    #[serde(rename = "leadAccountId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_account_id: Option<String>,
    #[serde(rename = "categoryId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Option<i64>>,
    #[serde(rename = "assigneeType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_type: Option<AssigneeType>,
}

#[derive(Debug, Clone)]
pub struct ProjectTemplate {
    pub id: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssigneeType {
    #[serde(rename = "PROJECT_LEAD")]
    ProjectLead,
    #[serde(rename = "UNASSIGNED")]
    Unassigned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCategory {
    pub id: i64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfigurationScheme {
    pub id: i64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIssueSecurityScheme {
    pub id: i64,
    pub name: String,
    pub description: String,
    #[serde(rename = "defaultSecurityLevelId")]
    pub default_security_level_id: String,
    pub levels: Vec<ProjectIssueSecurityLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIssueSecurityLevel {
    pub id: i64,
    pub name: String,
    pub description: String,
}
