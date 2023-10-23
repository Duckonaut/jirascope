#![allow(non_snake_case)] // Stops RA from complaining about the Emacs macros.

use std::sync::{Mutex, MutexGuard};

use emacs::{defun, Env, IntoLisp, Result, Value};
use jiroscope_core::{
    jira::{
        AtlassianDoc, Issue, IssueCreation, IssueCreationFields, IssueEdit,
        IssueTransitionDescriptor, Issues, Project, ProjectCreate, ProjectCreateDetails,
        PROJECT_TEMPLATES, PROJECT_TYPE_KEYS, PROJECT_TYPE_NAMES_TO_TEMPLATE_RANGE,
    },
    Auth, Config, Jiroscope,
};

#[cfg(feature = "benchmark")]
mod benchmark;
#[cfg(feature = "test_server")]
mod test_server;
#[allow(dead_code)]
mod utils;

// Emacs won't load the module without this.
emacs::plugin_is_GPL_compatible!();

static mut JIROSCOPE: Option<Mutex<Jiroscope>> = None;
static JIROSCOPE_BUFFER_NAME: &str = "*jiroscope*";

// Register the initialization hook that Emacs will call when it loads the module.
#[emacs::module]
fn init(env: &Env) -> Result<()> {
    env.call(
        "set",
        (
            env.intern("jiroscope-dyn--version")?,
            option_env!("CARGO_PKG_VERSION"),
        ),
    )?;
    Ok(())
}

#[defun]
fn setup(url: String, login: String, api_token: String) -> Result<()> {
    let config = Config::new(url);
    let auth = Auth::new(login, api_token);

    let mut jiroscope = Jiroscope::new(config, auth);
    jiroscope.init()?;

    unsafe {
        JIROSCOPE = Some(Mutex::new(jiroscope));
    }

    Ok(())
}

fn get_jiroscope<'a>() -> MutexGuard<'a, Jiroscope> {
    unsafe { JIROSCOPE.as_ref().unwrap() }.lock().unwrap()
}

#[defun(user_ptr)]
fn get_issue(_: &Env, issue_key: String) -> Result<Issue> {
    let issue = get_jiroscope().get_issue(&*issue_key)?;

    Ok(issue)
}

#[defun(user_ptr)]
fn get_all_issues(_: &Env) -> Result<Issues> {
    let issues = get_jiroscope().get_all_issues()?;

    Ok(issues)
}

#[defun]
fn get_issue_key<'e>(env: &'e Env, issue: &mut Issue) -> Result<Value<'e>> {
    issue.key.clone().into_lisp(env)
}

#[defun]
fn get_issue_summary<'e>(env: &'e Env, issue: &mut Issue) -> Result<Value<'e>> {
    issue.fields.summary.clone().into_lisp(env)
}

#[defun]
fn create_issue(env: &Env) -> Result<Value<'_>> {
    let mut jiroscope = get_jiroscope();
    let mut projects: Vec<Project> = jiroscope.get_projects()?;

    // let user choose project
    let index = utils::prompt_select_index(
        env,
        "Choose which project to create the issue in: ",
        projects
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>()
            .as_slice(),
    );

    if index.is_none() {
        return ().into_lisp(env);
    }

    let project = projects.remove(index.unwrap());

    // let user choose issue type
    let create_meta = jiroscope.get_issue_creation_meta()?;
    let mut issue_types = create_meta
        .projects
        .into_iter()
        .find(|p| p.id == project.id)
        .unwrap()
        .issue_types;

    let index = utils::prompt_select_index(
        env,
        "Choose which issue type to create: ",
        issue_types
            .iter()
            .map(|t| t.name.clone())
            .collect::<Vec<_>>()
            .as_slice(),
    );

    if index.is_none() {
        return ().into_lisp(env);
    }

    let issue_type = issue_types.remove(index.unwrap());

    // let user enter summary
    let summary = utils::prompt_string(env, "Enter issue summary: ");

    if summary.is_none() {
        return ().into_lisp(env);
    }

    // let user enter description
    let description = utils::prompt_string(env, "Enter issue description (or leave empty): ");

    let description = description.filter(|d| !d.is_empty());

    let issue_creation = IssueCreation {
        fields: IssueCreationFields {
            project,
            issue_type,
            summary: summary.unwrap(),
            description: description.map(|d| AtlassianDoc::from_markdown(&d)),
            priority: None,
            assignee: None,
        },
    };

    let issue = jiroscope.create_issue(issue_creation)?;

    let args = vec![format!("Created issue {}.", issue.key).into_lisp(env)?];

    env.call("message", &args)?;

    ().into_lisp(env)
}

fn prompt_issue(env: &Env) -> Option<Issue> {
    let mut jiroscope = get_jiroscope();
    // let user choose issue
    let mut issues = jiroscope.get_all_issues().unwrap().issues;

    let index = utils::prompt_select_index(
        env,
        "Choose issue: ",
        issues
            .iter()
            .map(|t| t.key.clone())
            .collect::<Vec<_>>()
            .as_slice(),
    )?;

    Some(issues.remove(index))
}

fn prompt_issue_transition(env: &Env, issue_key: &str) -> Option<IssueTransitionDescriptor> {
    let mut jiroscope = get_jiroscope();
    // let user choose issue status
    let mut issue_transitions = jiroscope
        .get_issue_transitions(issue_key)
        .unwrap()
        .transitions;

    let index = utils::prompt_select_index(
        env,
        "Choose issue status: ",
        issue_transitions
            .iter()
            .map(|t| t.name.clone())
            .collect::<Vec<_>>()
            .as_slice(),
    )?;

    Some(issue_transitions.remove(index))
}

#[defun]
fn edit_issue_interactive(env: &Env) -> Result<Value<'_>> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return ().into_lisp(env);
    }

    let issue = issue.unwrap();

    let mut issue_edit = IssueEdit::default();

    // let user enter summary
    issue_edit.fields.summary =
        utils::prompt_string(env, "Enter issue summary (or leave empty to leave as is): ");

    if issue_edit.fields.summary.is_none() {
        issue_edit.fields.summary = Some(issue.fields.summary);
    }

    // let user enter description
    issue_edit.fields.description = utils::prompt_string(
        env,
        "Enter issue description (or leave empty to leave as is): ",
    )
    .map(|d| AtlassianDoc::from_markdown(&d));

    get_jiroscope().edit_issue(&*issue.key, issue_edit)?;

    let args = vec![format!("Created issue {}.", issue.key).into_lisp(env)?];

    env.call("message", &args)?;

    ().into_lisp(env)
}

#[defun]
fn delete_issue(env: &Env, issue_key: String) -> Result<Value<'_>> {
    get_jiroscope().delete_issue(&*issue_key)?;

    ().into_lisp(env)
}

#[defun]
fn delete_issue_interactive(env: &Env) -> Result<Value<'_>> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return ().into_lisp(env);
    }

    let issue = issue.unwrap();

    get_jiroscope().delete_issue(&*issue.key)?;

    let args = vec![format!("Deleted issue {}.", issue.key).into_lisp(env)?];

    env.call("message", &args)?;

    ().into_lisp(env)
}

#[defun]
fn transition_issue_interactive(env: &Env) -> Result<Value<'_>> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return ().into_lisp(env);
    }

    let issue = issue.unwrap();

    let transition = prompt_issue_transition(env, &issue.key);

    if transition.is_none() {
        return ().into_lisp(env);
    }

    let transition = transition.unwrap();

    get_jiroscope().transition_issue(issue.key.as_str(), transition)?;

    let args = vec![format!("Transitioned issue {}.", issue.key).into_lisp(env)?];

    env.call("message", &args)?;

    ().into_lisp(env)
}

#[defun]
fn display_issue(env: &Env, issue_key: String) -> Result<Value<'_>> {
    let issue = get_jiroscope().get_issue(&*issue_key)?;
    open_jiroscope_buffer(env)?;

    let args = vec![format!("* {} *", issue_key).into_lisp(env)?];

    env.call("insert", &args)?;

    // create overlay with face "jiroscope-issue-key" for issue key

    let args = vec![0.into_lisp(env)?, (issue_key.len() + 4).into_lisp(env)?];

    let overlay = env.call("make-overlay", &args)?;

    let args = vec![
        overlay,
        env.intern("face")?,
        env.intern("jiroscope-issue-key")?,
    ];

    env.call("overlay-put", &args)?;

    env.call("newline", [])?;

    let args = vec![format!("Summary: {}", issue.fields.summary).into_lisp(env)?];

    env.call("insert", &args)?;

    env.call("newline", [])?;

    if let Some(description) = issue.fields.description {
        let args = vec![format!("Description: {}", description.to_markdown()).into_lisp(env)?];

        env.call("insert", &args)?;

        let args = vec![];

        env.call("newline", &args)?;
    }

    let args = vec![format!("Status: {}", issue.fields.status.name).into_lisp(env)?];

    env.call("insert", &args)?;

    let args = vec![];

    env.call("newline", &args)?;

    ().into_lisp(env)
}

#[defun]
fn display_issue_interactive(env: &Env) -> Result<Value<'_>> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return ().into_lisp(env);
    }

    let issue = issue.unwrap();

    display_issue(env, issue.key)?;

    ().into_lisp(env)
}

#[defun]
fn create_project(env: &Env) -> Result<Value<'_>> {
    let mut jiroscope = get_jiroscope();
    let key = utils::force_prompt_string(env, "Enter project key: ")?;
    let name = utils::force_prompt_string(env, "Enter project name: ")?;
    let description = utils::force_prompt_string(env, "Enter project description: ")?;
    let url = utils::prompt_string(env, "Enter project info URL (or leave empty): ");

    let users = jiroscope
        .get_users()?
        .into_iter()
        .filter(|u| u.active && u.account_type == "atlassian")
        .collect::<Vec<_>>();

    let index = utils::prompt_select_index(
        env,
        "Choose project lead: ",
        users
            .iter()
            .map(|u| u.display_name.clone())
            .collect::<Vec<_>>()
            .as_slice(),
    );

    if index.is_none() {
        return ().into_lisp(env);
    }

    let lead_account_id = users[index.unwrap()].account_id.clone();

    let project_categories = jiroscope.get_project_categories()?;

    let category_id = if project_categories.is_empty() {
        None
    } else {
        let index = utils::prompt_select_index(
            env,
            "Choose project category: ",
            project_categories
                .iter()
                .map(|c| c.name.clone())
                .collect::<Vec<_>>()
                .as_slice(),
        );

        if index.is_none() {
            return ().into_lisp(env);
        }

        Some(project_categories[index.unwrap()].id)
    };

    let index = utils::prompt_select_index(
        env,
        "Choose project type: ",
        PROJECT_TYPE_KEYS.to_vec().as_slice(),
    );

    if index.is_none() {
        return ().into_lisp(env);
    }

    let project_type_key = PROJECT_TYPE_KEYS[index.unwrap()].to_string();

    let index = utils::prompt_select_index(
        env,
        "Choose assignee type: ",
        ["Project Lead", "Unassigned"].to_vec().as_slice(),
    );

    if index.is_none() {
        return ().into_lisp(env);
    }

    let assignee_type = match index.unwrap() {
        0 => jiroscope_core::jira::AssigneeType::ProjectLead,
        1 => jiroscope_core::jira::AssigneeType::Unassigned,
        _ => unreachable!(),
    };

    let template_range = PROJECT_TYPE_NAMES_TO_TEMPLATE_RANGE
        .iter()
        .find(|(name, _, _)| *name == project_type_key)
        .unwrap();

    let index = utils::prompt_select_index(
        env,
        "Choose project template: ",
        PROJECT_TEMPLATES[template_range.1..template_range.2]
            .iter()
            .map(|c| c.description)
            .collect::<Vec<_>>()
            .as_slice(),
    );

    if index.is_none() {
        return ().into_lisp(env);
    }

    let template = PROJECT_TEMPLATES[template_range.1 + index.unwrap()]
        .id
        .to_string();

    let project = jiroscope.create_project(ProjectCreate {
        key,
        name,
        description,
        url,
        lead_account_id,
        project_type_key,
        assignee_type,
        category_id,
        details: ProjectCreateDetails::Template {
            project_template_key: template,
        },
    })?;

    let args = vec![format!("Created project {}.", project.key).into_lisp(env)?];

    env.call("message", &args)?;

    ().into_lisp(env)
}

#[defun]
fn delete_project(env: &Env, project_key: String) -> Result<Value<'_>> {
    get_jiroscope().delete_project(&*project_key)?;

    let args = vec![format!("Deleted project {}.", project_key).into_lisp(env)?];

    env.call("message", &args)?;

    ().into_lisp(env)
}

#[defun]
fn delete_project_interactive(env: &Env) -> Result<Value<'_>> {
    let projects = get_jiroscope().get_projects()?;

    let index = utils::prompt_select_index(
        env,
        "Choose project to delete: ",
        projects
            .iter()
            .map(|p| p.key.clone())
            .collect::<Vec<_>>()
            .as_slice(),
    );

    if index.is_none() {
        return ().into_lisp(env);
    }

    let project = projects[index.unwrap()].key.clone();

    delete_project(env, project)?;

    ().into_lisp(env)
}

#[defun]
fn open_jiroscope_buffer(env: &Env) -> Result<Value<'_>> {
    let args = vec![JIROSCOPE_BUFFER_NAME.to_string().into_lisp(env)?];

    let buffer = env.call("get-buffer-create", &args)?;

    let args = vec![buffer];

    env.call("switch-to-buffer", &args)?;

    env.call("erase-buffer", [])?;

    ().into_lisp(env)
}
