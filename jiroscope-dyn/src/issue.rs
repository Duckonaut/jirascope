use emacs::{defun, Env, IntoLisp, Result, Value};
use jiroscope_core::jira::{
    AtlassianDoc, Issue, IssueCreation, IssueCreationFields, IssueEdit, IssueTransitionDescriptor,
    Issues, Project,
};

use crate::{
    get_jiroscope,
    utils::{self, open_jiroscope_buffer},
};

#[defun(user_ptr)]
fn get(_: &Env, issue_key: String) -> Result<Issue> {
    let issue = get_jiroscope().get_issue(&*issue_key)?;

    Ok(issue)
}

#[defun(user_ptr)]
fn get_all(_: &Env) -> Result<Issues> {
    let issues = get_jiroscope().get_all_issues()?;

    Ok(issues)
}

#[defun]
fn get_key<'e>(env: &'e Env, issue: &mut Issue) -> Result<Value<'e>> {
    issue.key.clone().into_lisp(env)
}

#[defun]
fn get_summary<'e>(env: &'e Env, issue: &mut Issue) -> Result<Value<'e>> {
    issue.fields.summary.clone().into_lisp(env)
}

#[defun]
fn create(env: &Env) -> Result<Value<'_>> {
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
        return utils::nil(env);
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
        return utils::nil(env);
    }

    let issue_type = issue_types.remove(index.unwrap());

    // let user enter summary
    let summary = utils::prompt_string(env, "Enter issue summary: ");

    if summary.is_none() {
        return utils::nil(env);
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

    utils::nil(env)
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
fn edit_interactive(env: &Env) -> Result<Value<'_>> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return utils::nil(env);
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

    utils::nil(env)
}

#[defun]
fn delete(env: &Env, issue_key: String) -> Result<Value<'_>> {
    get_jiroscope().delete_issue(&*issue_key)?;

    utils::nil(env)
}

#[defun]
fn delete_interactive(env: &Env) -> Result<Value<'_>> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return utils::nil(env);
    }

    let issue = issue.unwrap();

    get_jiroscope().delete_issue(&*issue.key)?;

    let args = vec![format!("Deleted issue {}.", issue.key).into_lisp(env)?];

    env.call("message", &args)?;

    utils::nil(env)
}

#[defun]
fn transition_interactive(env: &Env) -> Result<Value<'_>> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return utils::nil(env);
    }

    let issue = issue.unwrap();

    let transition = prompt_issue_transition(env, &issue.key);

    if transition.is_none() {
        return utils::nil(env);
    }

    let transition = transition.unwrap();

    get_jiroscope().transition_issue(issue.key.as_str(), transition)?;

    let args = vec![format!("Transitioned issue {}.", issue.key).into_lisp(env)?];

    env.call("message", &args)?;

    utils::nil(env)
}

#[defun]
fn display(env: &Env, issue_key: String) -> Result<Value<'_>> {
    let issue = get_jiroscope().get_issue(&*issue_key)?;
    utils::set_buffer_mode(utils::JiroscopeBufferMode::Issue);
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

    utils::nil(env)
}

#[defun]
fn display_interactive(env: &Env) -> Result<Value<'_>> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return utils::nil(env);
    }

    let issue = issue.unwrap();

    display(env, issue.key)?;

    utils::nil(env)
}
