use std::thread;

use emacs::{defun, Env, IntoLisp, Result, Value};
use jiroscope_core::jira::{
    AtlassianDoc, Issue, IssueCreation, IssueCreationFields, IssueEdit, IssueTransitionDescriptor,
};

use crate::{
    concurrent, get_jiroscope,
    state::{self, get_state},
    utils::{self, open_jiroscope_buffer},
};

#[defun]
fn create_interactive(env: &Env) -> Result<Value<'_>> {
    let mut jiroscope = get_jiroscope();
    let state = get_state();
    let projects = state.projects();

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

    let project = projects[index.unwrap()].clone();

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

    thread::spawn(move || {
        let result = get_jiroscope().create_issue(issue_creation);

        if result.is_ok() {
            concurrent::push_command(Box::new(|env| {
                state::refresh(env)?;

                env.call("message", ["Created issue.".into_lisp(env)?])?;

                Ok(())
            }));
        } else {
            concurrent::push_command(Box::new(|env| {
                env.call("message", ["Failed to create issue.".into_lisp(env)?])?;

                Ok(())
            }));
        }
    });

    utils::nil(env)
}

fn prompt_issue(env: &Env) -> Option<Issue> {
    // let user choose issue
    let state = get_state();
    let issues = state.issues();

    let index = utils::prompt_select_index(
        env,
        "Choose issue: ",
        issues
            .iter()
            .map(|t| t.key.clone())
            .collect::<Vec<_>>()
            .as_slice(),
    )?;

    Some(issues[index].clone())
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

    thread::spawn(move || {
        let result = get_jiroscope().edit_issue(&*issue.key, issue_edit);

        if result.is_ok() {
            concurrent::push_command(Box::new(|env| {
                state::refresh(env)?;

                env.call("message", ["Edited issue.".into_lisp(env)?])?;

                Ok(())
            }));
        } else {
            concurrent::push_command(Box::new(|env| {
                env.call("message", ["Failed to edit issue.".into_lisp(env)?])?;

                Ok(())
            }));
        }
    });

    utils::nil(env)
}

#[defun]
fn delete_interactive(env: &Env) -> Result<Value<'_>> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return utils::nil(env);
    }

    let issue = issue.unwrap();
    let issue_key = issue.key;

    thread::spawn(move || {
        let result = get_jiroscope().delete_issue(&*issue_key);
        if result.is_ok() {
            concurrent::push_command(Box::new(move |env| {
                state::refresh(env)?;

                env.call(
                    "message",
                    [format!("Deleted issue {}.", issue_key).into_lisp(env)?],
                )?;

                Ok(())
            }));
        } else {
            concurrent::push_command(Box::new(|env| {
                env.call("message", ["Failed to delete issue.".into_lisp(env)?])?;

                Ok(())
            }));
        }
    });

    utils::nil(env)
}

#[defun]
fn transition_interactive(env: &Env) -> Result<Value<'_>> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return utils::nil(env);
    }

    let issue = issue.unwrap();
    let issue_key = issue.key;

    let transition = prompt_issue_transition(env, &issue_key);

    if transition.is_none() {
        return utils::nil(env);
    }

    let transition = transition.unwrap();

    thread::spawn(move || {
        let result = get_jiroscope().transition_issue(issue_key.as_str(), transition);

        if result.is_ok() {
            concurrent::push_command(Box::new(move |env| {
                state::refresh(env)?;

                env.call(
                    "message",
                    [format!("Transitioned issue {}.", issue_key).into_lisp(env)?],
                )?;

                Ok(())
            }));
        } else {
            concurrent::push_command(Box::new(|env| {
                env.call("message", ["Failed to transition issue.".into_lisp(env)?])?;

                Ok(())
            }));
        }
    });

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
