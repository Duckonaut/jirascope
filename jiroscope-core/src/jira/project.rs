use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub key: String,
    pub name: String,
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

pub const PROJECT_TEMPLATE_KEYS: [&str; 37] = [
    "com.atlassian.jira-core-project-templates:jira-core-simplified-content-management",
    "com.atlassian.jira-core-project-templates:jira-core-simplified-document-approval",
    "com.atlassian.jira-core-project-templates:jira-core-simplified-lead-tracking",
    "com.atlassian.jira-core-project-templates:jira-core-simplified-process-control",
    "com.atlassian.jira-core-project-templates:jira-core-simplified-procurement",
    "com.atlassian.jira-core-project-templates:jira-core-simplified-project-management",
    "com.atlassian.jira-core-project-templates:jira-core-simplified-recruitment",
    "com.atlassian.jira-core-project-templates:jira-core-simplified-task-tracking",
    "com.atlassian.servicedesk:simplified-it-service-management",
    "com.atlassian.servicedesk:simplified-general-service-desk-it",
    "com.atlassian.servicedesk:simplified-general-service-desk-business",
    "com.atlassian.servicedesk:simplified-external-service-desk",
    "com.atlassian.servicedesk:simplified-hr-service-desk",
    "com.atlassian.servicedesk:simplified-facilities-service-desk",
    "com.atlassian.servicedesk:simplified-legal-service-desk",
    "com.atlassian.servicedesk:simplified-analytics-service-desk",
    "com.atlassian.servicedesk:simplified-marketing-service-desk",
    "com.atlassian.servicedesk:simplified-design-service-desk",
    "com.atlassian.servicedesk:simplified-sales-service-desk",
    "com.atlassian.servicedesk:simplified-finance-service-desk",
    "com.atlassian.servicedesk:next-gen-it-service-desk",
    "com.atlassian.servicedesk:next-gen-hr-service-desk",
    "com.atlassian.servicedesk:next-gen-legal-service-desk",
    "com.atlassian.servicedesk:next-gen-marketing-service-desk",
    "com.atlassian.servicedesk:next-gen-facilities-service-desk",
    "com.atlassian.servicedesk:next-gen-general-service-desk",
    "com.atlassian.servicedesk:next-gen-general-it-service-desk",
    "com.atlassian.servicedesk:next-gen-general-business-service-desk",
    "com.atlassian.servicedesk:next-gen-analytics-service-desk",
    "com.atlassian.servicedesk:next-gen-finance-service-desk",
    "com.atlassian.servicedesk:next-gen-design-service-desk",
    "com.atlassian.servicedesk:next-gen-sales-service-desk",
    "com.pyxis.greenhopper.jira:gh-simplified-agility-kanban",
    "com.pyxis.greenhopper.jira:gh-simplified-agility-scrum",
    "com.pyxis.greenhopper.jira:gh-simplified-basic",
    "com.pyxis.greenhopper.jira:gh-simplified-kanban-classic",
    "com.pyxis.greenhopper.jira:gh-simplified-scrum-classic",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCreate {
    pub key: String,
    pub name: String,
    pub description: String,
    pub url: String,
    #[serde(rename = "leadAccountId")]
    pub lead_account_id: String,
    #[serde(rename = "avatarId")]
    pub avatar_id: i64,
    #[serde(rename = "categoryId")]
    pub category_id: i64,
    #[serde(rename = "projectTypeKey")]
    pub project_type_key: String,
    #[serde(rename = "assigneeType")]
    pub assignee_type: AssigneeType,
    #[serde(rename = "issueSecurityScheme")]
    pub issue_security_scheme: i64,
    #[serde(rename = "permissionScheme")]
    pub permission_scheme: i64,
    #[serde(rename = "notificationScheme")]
    pub notification_scheme: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssigneeType {
    #[serde(rename = "PROJECT_LEAD")]
    ProjectLead,
    #[serde(rename = "UNASSIGNED")]
    Unassigned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCategory {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfigurationScheme {
    pub id: String,
    pub name: String,
    pub description: String,
}
