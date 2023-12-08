use emacs::{defun, Env, Result, Value};
use jiroscope_core::jira::{
    AtlassianDoc, Issue, IssueCreation, IssueCreationFields, IssueEdit, IssueTransitionDescriptor,
    WrappedId,
};

use crate::{
    concurrent, get_jiroscope, project,
    state::{self, get_state, ConflictCell},
    utils::{
        self, close_jiroscope_diff_buffer, current_buffer_face_println, current_buffer_println,
        get_jiroscope_buffer_content, open_jiroscope_buffer, open_jiroscope_diff_buffer,
        prompt_force_change, signal_result, signal_result_async, with_buffer, workthread_spawn,
        ScopeCleaner, JIROSCOPE_FACE_DIFF_ALERT, JIROSCOPE_FACE_DIFF_NEW, JIROSCOPE_FACE_DIFF_OLD,
    },
    JIROSCOPE_DIFF_BUFFER_NAME,
};

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

fn prompt_issue_parent(env: &Env, project_key: &str) -> Option<i64> {
    let state = get_state();
    // let user choose issue status
    let mut issue_parents = state.issues().iter().filter_map(|i| {
        if i.fields.project.key == project_key {
            Some((i.id, i.key.clone()))
        } else {
            None
        }
    });

    let index = utils::prompt_select_index(
        env,
        "Choose parent issue: ",
        issue_parents
            .clone()
            .map(|(_, k)| k)
            .collect::<Vec<_>>()
            .as_slice(),
    )?;

    Some(issue_parents.nth(index).unwrap().0)
}

#[defun]
fn create_interactive(env: &Env) -> Result<Value<'_>> {
    let mut jiroscope = get_jiroscope();
    let state = get_state();

    // let user choose project
    let project = match project::prompt_select_project(env) {
        Some(p) => p,
        None => return utils::nil(env),
    };
    drop(state);

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

    let parent = if issue_type.is_subtask {
        let parent_id = prompt_issue_parent(env, &project.key);

        if parent_id.is_none() {
            return utils::nil(env);
        }

        parent_id
    } else {
        None
    };

    // let user enter summary
    let summary = utils::prompt_string(env, "Enter issue summary: ");

    if summary.is_none() {
        return utils::nil(env);
    }

    let summary = summary.unwrap();

    // let user enter description
    let description = utils::prompt_string(env, "Enter issue description (or leave empty): ")
        .filter(|d| !d.is_empty())
        .map(|d| AtlassianDoc::from_markdown(&d));

    let issue_creation = IssueCreation {
        fields: IssueCreationFields {
            project,
            issue_type,
            summary,
            description,
            priority: None,
            assignee: None,
            parent: parent.map(WrappedId::new),
        },
    };

    workthread_spawn(move || {
        let result = get_jiroscope().create_issue(issue_creation);

        if result.is_ok() {
            concurrent::push_command(Box::new(|env| {
                state::refresh(env)?;

                env.message("Issue created successfully.")?;

                Ok(())
            }));
        } else {
            concurrent::push_command(Box::new(|env| {
                env.message("Failed to create issue.")?;

                Ok(())
            }));
        }
    });

    utils::nil(env)
}

#[defun]
fn edit_interactive(env: &Env) -> Result<Value<'_>> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return utils::nil(env);
    }

    let issue = issue.unwrap();

    get_state().check_out_issue(issue.key.clone())?;
    // in case something fails, we want to make sure the issue is returned
    // to not lock it forever
    let guard = ScopeCleaner::new(|| get_state().return_issue());

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

    workthread_spawn(move || {
        if !get_state().try_return_issue(&*issue.key) {
            concurrent::push_command(Box::new(move |env| {
                env.message("Issue changed since last access. Please check the diff buffer.")?;

                if prompt_force_change(env, "Issue changed since last access")? {
                    let result = get_jiroscope().edit_issue(&*issue.key, issue_edit);

                    signal_result(env, result, "Issue edited.", "Failed to edit issue.")?;
                }

                Ok(())
            }));
            return;
        }

        let result = get_jiroscope().edit_issue(&*issue.key, issue_edit);

        signal_result_async(result, "Issue edited.", "Failed to edit issue.");
        // make sure the guard is moved into the closure
        drop(guard);
    });

    utils::nil(env)
}

#[defun]
fn button_action(env: &Env, button: Value<'_>) -> Result<()> {
    let button_content = env.call("button-label", [button])?.into_rust::<String>()?;

    edit_graphical(env, button_content)
}

#[defun]
fn edit_graphical_interactive(env: &Env) -> Result<()> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return Ok(());
    }

    edit_graphical(env, issue.unwrap().key)
}

fn edit_graphical(env: &Env, issue_key: String) -> Result<()> {
    let issue = get_state().get_issue(&issue_key);

    if issue.is_none() {
        return Ok(());
    }

    let issue = issue.unwrap();

    get_state().return_issue();
    get_state().check_out_issue(issue.key.clone())?;

    open_jiroscope_buffer(env)?;

    current_buffer_face_println(env, &format!("* {} *", issue_key), "jiroscope-issue-key")?;

    current_buffer_println(env, &format!("Summary: {}", issue.fields.summary))?;

    current_buffer_println(env, &format!("Status: {}", issue.fields.status.name))?;

    if let Some(description) = issue.fields.description {
        current_buffer_println(env, &format!("Description: {}", description.to_markdown()))?;
    }

    utils::set_buffer_mode(env, utils::JiroscopeBufferMode::IssueEdit)?;

    Ok(())
}

#[defun]
fn edit_graphical_finish(env: &Env) -> Result<Value<'_>> {
    let mut issue_edit = IssueEdit::default();

    let edited_issue = get_jiroscope_buffer_content(env)?;

    // parse out issue edit
    let key = edited_issue
        .lines()
        .next()
        .unwrap()
        .trim_start_matches("* ")
        .trim_end_matches(" *")
        .to_string();

    issue_edit.fields.summary = edited_issue
        .lines()
        .find(|l| l.starts_with("Summary: "))
        .map(|l| l.trim_start_matches("Summary: ").to_string());

    let description_str = &edited_issue[edited_issue
        .find("Description: ")
        .map(|i| i + "Description: ".len())
        .unwrap_or(0)..];

    issue_edit.fields.description = if description_str.is_empty() {
        None
    } else {
        Some(AtlassianDoc::from_markdown(description_str))
    };

    workthread_spawn(move || {
        if !get_state().try_return_issue(&*key) {
            concurrent::push_command(Box::new(move |env| {
                env.message("Issue changed since last access.")?;

                display_old_and_changed(env)?;

                if prompt_force_change(env, "Issue changed since last access")? {
                    let result = get_jiroscope().edit_issue(&*key, issue_edit);

                    state::get_state().return_issue();

                    signal_result(env, result, "Issue edited.", "Failed to edit issue.")?;

                    state::get_state().check_out_issue(key.clone())?;
                }

                close_jiroscope_diff_buffer(env)?;

                Ok(())
            }));
            return;
        }

        let result = get_jiroscope().edit_issue(&*key, issue_edit);

        state::get_state().return_issue();

        signal_result_async(result, "Issue edited.", "Failed to edit issue.");

        concurrent::push_command(Box::new(move |env| {
            state::open(env)?;

            Ok(())
        }));
    });

    utils::nil(env)
}

fn display_old_and_changed(env: &Env) -> Result<()> {
    let state = get_state();

    let work_issue = state.get_current_work_issue();

    if matches!(work_issue, ConflictCell::Empty | ConflictCell::Armed { .. }) {
        return Ok(());
    }

    with_buffer(env, JIROSCOPE_DIFF_BUFFER_NAME, |env| {
        env.call("erase-buffer", [])?;
        match work_issue {
            ConflictCell::Deleted { key } => {
                current_buffer_face_println(env, &format!("* {} *", key), "jiroscope-issue-key")?;
                current_buffer_face_println(env, "Issue was deleted.", JIROSCOPE_FACE_DIFF_ALERT)?;
            }
            ConflictCell::Outdated { key, old } => {
                current_buffer_face_println(env, &format!("* {} *", key), "jiroscope-issue-key")?;
                current_buffer_face_println(
                    env,
                    "Issue was changed since last access.",
                    JIROSCOPE_FACE_DIFF_ALERT,
                )?;
                current_buffer_face_println(env, "Old:", JIROSCOPE_FACE_DIFF_ALERT)?;
                current_buffer_face_println(
                    env,
                    &format!("Summary: {}", old.fields.summary),
                    JIROSCOPE_FACE_DIFF_OLD,
                )?;
                current_buffer_face_println(
                    env,
                    &format!("Status: {}", old.fields.status.name,),
                    JIROSCOPE_FACE_DIFF_OLD,
                )?;

                if let Some(ref description) = old.fields.description {
                    current_buffer_face_println(
                        env,
                        &format!("Description: {}", description.to_markdown()),
                        JIROSCOPE_FACE_DIFF_OLD,
                    )?;
                }
                current_buffer_face_println(env, "New:", JIROSCOPE_FACE_DIFF_ALERT)?;
                let current = state.get_issue(key).unwrap();

                current_buffer_face_println(
                    env,
                    &format!("Summary: {}", current.fields.summary),
                    JIROSCOPE_FACE_DIFF_NEW,
                )?;
                current_buffer_face_println(
                    env,
                    &format!("Status: {}", current.fields.status.name),
                    JIROSCOPE_FACE_DIFF_NEW,
                )?;

                if let Some(ref description) = current.fields.description {
                    current_buffer_face_println(
                        env,
                        &format!("Description: {}", description.to_markdown()),
                        JIROSCOPE_FACE_DIFF_NEW,
                    )?;
                }
            }
            _ => {}
        }
        Ok(())
    })?;

    open_jiroscope_diff_buffer(env)?;

    Ok(())
}

#[defun]
fn delete_interactive(env: &Env) -> Result<Value<'_>> {
    let issue = prompt_issue(env);

    if issue.is_none() {
        return utils::nil(env);
    }

    let issue = issue.unwrap();
    let issue_key = issue.key;

    workthread_spawn(move || {
        let result = get_jiroscope().delete_issue(&*issue_key);
        signal_result_async(result, "Issue deleted.", "Failed to delete issue.");
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

    workthread_spawn(move || {
        let result = get_jiroscope().transition_issue(issue_key.as_str(), transition);

        signal_result_async(result, "Transitioned issue.", "Failed to transition issue.");
    });

    utils::nil(env)
}

#[defun]
fn display(env: &Env, issue_key: String) -> Result<Value<'_>> {
    let issue = get_jiroscope().get_issue(&*issue_key)?;
    open_jiroscope_buffer(env)?;

    current_buffer_face_println(env, &format!("* {} *", issue_key), "jiroscope-issue-key")?;

    current_buffer_println(env, &format!("Summary: {}", issue.fields.summary))?;

    current_buffer_println(env, &format!("Status: {}", issue.fields.status.name))?;

    if let Some(description) = issue.fields.description {
        current_buffer_println(env, &format!("Description: {}", description.to_markdown()))?;
    }

    utils::set_buffer_mode(env, utils::JiroscopeBufferMode::Issue)?;

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
