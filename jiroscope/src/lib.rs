#![allow(non_snake_case)] // Stops RA from complaining about the Emacs macros.

use std::sync::{Mutex, MutexGuard};

use emacs::{defun, Env, IntoLisp, Result, Value};
use jiroscope_core::{
    jira::{Issue, IssueCreation, IssueCreationFields, Issues, Project},
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
fn init(_: &Env) -> Result<()> {
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
    let mut projects: Vec<Project> = get_jiroscope().get_projects()?;

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
    let create_meta = get_jiroscope().get_issue_creation_meta()?;
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
            description,
            priority: None,
            assignee: None,
        },
    };

    let issue = get_jiroscope().create_issue(issue_creation)?;

    let args = vec![format!("Created issue {}.", issue.key).into_lisp(env)?];

    env.call("message", &args)?;

    ().into_lisp(env)
}

#[defun]
fn display_issue(env: &Env, issue_key: String) -> Result<Value<'_>> {
    let issue = get_jiroscope().get_issue(&*issue_key)?;
    open_jiroscope_buffer(env)?;

    let args = vec![format!("* {} *", issue_key).into_lisp(env)?];

    env.call("insert", &args)?;

    let args = vec![];

    env.call("newline", &args)?;

    let args = vec![format!("Summary: {}", issue.fields.summary).into_lisp(env)?];

    env.call("insert", &args)?;

    let args = vec![];

    env.call("newline", &args)?;

    if let Some(description) = issue.fields.description {
        let args = vec![format!("Description: {}", description).into_lisp(env)?];

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
fn open_jiroscope_buffer(env: &Env) -> Result<Value<'_>> {
    let args = vec![JIROSCOPE_BUFFER_NAME.to_string().into_lisp(env)?];

    let buffer = env.call("get-buffer-create", &args)?;

    let args = vec![buffer];

    env.call("switch-to-buffer", &args)?;

    env.call("erase-buffer", [])?;

    ().into_lisp(env)
}
