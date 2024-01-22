use emacs::{defun, Env, Result, Value};
use jirascope_core::jira::{
    Project, ProjectCategory, ProjectCreate, ProjectCreateDetails, ProjectEdit, PROJECT_TEMPLATES,
    PROJECT_TYPE_KEYS, PROJECT_TYPE_NAMES_TO_TEMPLATE_RANGE,
};

use crate::{
    concurrent::{self, workthread_spawn}, get_jirascope,
    state::{self, get_state, ConflictCell, get_state_mut},
    utils::{
        self, close_jirascope_diff_buffer, current_buffer_face_println, current_buffer_println,
        get_jirascope_buffer_content, open_jirascope_buffer, open_jirascope_diff_buffer,
        prompt_force_change, signal_result, signal_result_async, with_buffer,
        JIRASCOPE_FACE_DIFF_ALERT, JIRASCOPE_FACE_DIFF_NEW, JIRASCOPE_FACE_DIFF_OLD, set_buffer_mode, JirascopeBufferMode,
    },
    JIRASCOPE_DIFF_BUFFER_NAME,
};

pub fn prompt_select_project(env: &Env) -> Option<Project> {
    let state = get_state();
    let projects = state.projects();
    let index = utils::prompt_select_index(
        env,
        "Choose project: ",
        projects
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>()
            .as_slice(),
    )?;

    Some(projects[index].clone().to_project())
}

#[defun]
fn create_interactive(env: &Env) -> Result<()> {
    let mut jirascope = get_jirascope();
    let key = utils::force_prompt_string(env, "Enter project key: ")?;
    let name = utils::force_prompt_string(env, "Enter project name: ")?;
    let description = utils::force_prompt_string(env, "Enter project description: ")?;
    let url = utils::prompt_string(env, "Enter project info URL (or leave empty): ");

    let users = jirascope
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
        return Ok(());
    }

    let lead_account_id = users[index.unwrap()].account_id.clone();

    let project_categories = jirascope.get_project_categories()?;

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
            return Ok(());
        }

        Some(project_categories[index.unwrap()].id)
    };

    let index = utils::prompt_select_index(
        env,
        "Choose project type: ",
        PROJECT_TYPE_KEYS.to_vec().as_slice(),
    );

    if index.is_none() {
        return Ok(());
    }

    let project_type_key = PROJECT_TYPE_KEYS[index.unwrap()].to_string();

    let index = utils::prompt_select_index(
        env,
        "Choose assignee type: ",
        ["Project Lead", "Unassigned"].to_vec().as_slice(),
    );

    if index.is_none() {
        return Ok(());
    }

    let assignee_type = match index.unwrap() {
        0 => jirascope_core::jira::AssigneeType::ProjectLead,
        1 => jirascope_core::jira::AssigneeType::Unassigned,
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
        return Ok(());
    }

    let template = PROJECT_TEMPLATES[template_range.1 + index.unwrap()]
        .id
        .to_string();

    let project_create = ProjectCreate {
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
    };

    workthread_spawn(move || {
        let result = get_jirascope().create_project(project_create);

        if result.is_ok() {
            concurrent::push_command(Box::new(|env| {
                state::refresh(env)?;

                env.message("Project created successfully.")?;

                Ok(())
            }));
        } else {
            concurrent::push_command(Box::new(|env| {
                env.message("Failed to create project.")?;

                Ok(())
            }));
        }
    });

    Ok(())
}

#[defun]
fn edit_interactive(env: &Env) -> Result<()> {
    let project = prompt_select_project(env);

    if project.is_none() {
        return Ok(());
    }

    let project = project.unwrap();

    let key = utils::prompt_string(env, "Enter new project key (leave empty for no change): ");

    let name = utils::prompt_string(env, "Enter new project name (leave empty for no change): ");

    let description = utils::prompt_string(
        env,
        "Enter new project description (leave empty for no change): ",
    );

    let url = utils::prompt_string(
        env,
        "Enter new project URL (leave empty for no change, \"none\" for empty value): ",
    );

    let url = if let Some("none") = url.as_deref() {
        None
    } else {
        Some(url)
    };

    let users = get_jirascope()
        .get_users()?
        .into_iter()
        .filter(|u| u.active && u.account_type == "atlassian")
        .collect::<Vec<_>>();

    let index = utils::prompt_select_index(
        env,
        "Choose new project lead (leave empty for no change): ",
        users
            .iter()
            .map(|u| u.display_name.clone())
            .collect::<Vec<_>>()
            .as_slice(),
    );

    let lead_account_id = index.map(|index| users[index].account_id.clone());

    let mut categories = get_jirascope().get_project_categories()?;
    categories.push(ProjectCategory {
        id: 0,
        name: "None".to_string(),
        description: "No category".to_string(),
    });
    let index = utils::prompt_select_index(
        env,
        "Choose new project category (leave empty for no change): ",
        categories
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<_>>()
            .as_slice(),
    );
    let category_id = index.map(|index| {
        if index == categories.len() - 1 {
            None
        } else {
            Some(categories[index].id)
        }
    });

    let assignee_type = match utils::prompt_select_index(
        env,
        "Choose new assignee type (leave empty for no change): ",
        ["Project Lead", "Unassigned"].to_vec().as_slice(),
    ) {
        Some(0) => Some(jirascope_core::jira::AssigneeType::ProjectLead),
        Some(1) => Some(jirascope_core::jira::AssigneeType::Unassigned),
        _ => None,
    };

    let project_edit = ProjectEdit {
        key,
        name,
        description,
        url,
        lead_account_id,
        category_id,
        assignee_type,
    };

    workthread_spawn(move || {
        let result = get_jirascope().edit_project(&*project.key, project_edit);

        if result.is_ok() {
            concurrent::push_command(Box::new(|env| {
                state::refresh(env)?;

                env.message("Project edited successfully.")?;

                Ok(())
            }));
        } else {
            concurrent::push_command(Box::new(|env| {
                env.message("Failed to edit project.")?;

                Ok(())
            }));
        }
    });

    Ok(())
}

#[defun]
fn button_action(env: &Env, button: Value<'_>) -> Result<()> {
    let button_content = env.call("button-label", [button])?.into_rust::<String>()?;

    edit_graphical(env, button_content)
}

#[defun]
fn edit_graphical_interactive(env: &Env) -> Result<()> {
    let project = prompt_select_project(env);

    if project.is_none() {
        return Ok(());
    }

    edit_graphical(env, project.unwrap().key)
}

#[defun]
fn edit_graphical(env: &Env, key: String) -> Result<()> {
    let project = get_state().get_project_detailed(&key);

    if project.is_none() {
        return Ok(());
    }

    let project = project.unwrap();

    get_state_mut().return_project();
    get_state_mut().check_out_project(key.clone())?;

    open_jirascope_buffer(env)?;

    current_buffer_face_println(
        env,
        &format!("* {} *", project.key),
        "jirascope-project-key",
    )?;

    current_buffer_println(env, &format!("Name: {}", project.name))?;

    current_buffer_println(env, &format!("Description: {}", project.description))?;

    if let Some(url) = &project.url {
        current_buffer_println(env, &format!("URL: {}", url))?;
    } else {
        current_buffer_println(env, "URL: None")?;
    }

    current_buffer_println(env, &format!("Lead: {}", project.lead.display_name))?;

    set_buffer_mode(env, JirascopeBufferMode::ProjectEdit)?;

    Ok(())
}

#[defun]
fn edit_graphical_finish(env: &Env) -> Result<()> {
    let mut project_edit = ProjectEdit::default();

    let edited_project = get_jirascope_buffer_content(env)?;

    let og_key = match get_state().get_current_work_project() {
        ConflictCell::Empty => return Ok(()),
        ConflictCell::Armed { key } => key,
        ConflictCell::Outdated { key, .. } => key,
        ConflictCell::Deleted { key } => key,
    }
    .clone();

    // parse out project edit
    let key = edited_project
        .lines()
        .next()
        .unwrap()
        .trim_start_matches("* ")
        .trim_end_matches(" *")
        .to_string();

    project_edit.key = match key.as_str() {
        "" => None,
        _ => Some(key.clone()),
    };

    project_edit.name = edited_project
        .lines()
        .find(|l| l.starts_with("Name: "))
        .map(|l| l.trim_start_matches("Name: ").to_string());

    let description_str = edited_project
        .lines()
        .find(|l| l.starts_with("Description: "))
        .map(|l| l.trim_start_matches("Description: ").to_string());

    match description_str.as_deref() {
        Some(description_str) => project_edit.description = Some(description_str.to_string()),
        None => project_edit.description = None,
    };

    let url_str = edited_project
        .lines()
        .find(|l| l.starts_with("URL: "))
        .map(|l| l.trim_start_matches("URL: ").to_string());

    project_edit.url = match url_str.as_deref() {
        Some("None") => None,
        Some(url_str) => Some(Some(url_str.to_string())),
        None => None,
    };

    let lead_str = edited_project
        .lines()
        .find(|l| l.starts_with("Lead: "))
        .map(|l| l.trim_start_matches("Lead: ").to_string());

    project_edit.lead_account_id = match lead_str.as_deref() {
        Some(lead_str) => {
            let users = get_jirascope().get_users()?;

            let user = users.into_iter().find(|u| u.display_name == lead_str);

            match user {
                Some(user) => Some(user.account_id),
                None => return Err(jirascope_core::Error::jirascope("Invalid user.").into()),
            }
        }
        None => None,
    };

    workthread_spawn(move || {
        if !get_state_mut().try_return_project(og_key.as_str()) {
            concurrent::push_command(Box::new(move |env| {
                env.message("Project changed since last access.")?;

                display_old_and_changed(env)?;

                if prompt_force_change(env, "Project changed since last access")? {
                    let result = get_jirascope().edit_project(og_key.as_str(), project_edit);

                    state::get_state_mut().return_project();

                    signal_result(env, result, "Project edited.", "Failed to edit project.")?;

                    state::get_state_mut().check_out_project(key.clone())?;
                }

                close_jirascope_diff_buffer(env)?;

                Ok(())
            }));
            return;
        }

        let result = get_jirascope().edit_project(og_key.as_str(), project_edit);

        state::get_state_mut().return_project();

        signal_result_async(result, "Project edited.", "Failed to edit project.");

        concurrent::push_command(Box::new(move |env| {
            state::open(env)?;

            Ok(())
        }));
    });

    Ok(())
}

fn display_old_and_changed(env: &Env) -> Result<()> {
    let state = get_state();

    let work_project = state.get_current_work_project();

    if matches!(
        work_project,
        ConflictCell::Empty | ConflictCell::Armed { .. }
    ) {
        return Ok(());
    }

    with_buffer(env, JIRASCOPE_DIFF_BUFFER_NAME, |env| {
        env.call("erase-buffer", [])?;
        match work_project {
            ConflictCell::Deleted { key } => {
                current_buffer_face_println(env, &format!("* {} *", key), "jirascope-project-key")?;
                current_buffer_face_println(env, "Issue was deleted.", JIRASCOPE_FACE_DIFF_ALERT)?;
            }
            ConflictCell::Outdated { key, old } => {
                current_buffer_face_println(env, &format!("* {} *", key), "jirascope-project-key")?;
                current_buffer_face_println(
                    env,
                    "Issue was changed since last access.",
                    JIRASCOPE_FACE_DIFF_ALERT,
                )?;
                current_buffer_face_println(env, "Old:", JIRASCOPE_FACE_DIFF_ALERT)?;
                current_buffer_face_println(
                    env,
                    &format!("* {} *", old.key),
                    JIRASCOPE_FACE_DIFF_OLD,
                )?;
                current_buffer_face_println(
                    env,
                    &format!("Name: {}", old.name),
                    JIRASCOPE_FACE_DIFF_OLD,
                )?;
                current_buffer_face_println(
                    env,
                    &format!("Description: {}", old.description),
                    JIRASCOPE_FACE_DIFF_OLD,
                )?;

                if let Some(ref description) = old.url {
                    current_buffer_face_println(
                        env,
                        &format!("URL: {}", description),
                        JIRASCOPE_FACE_DIFF_OLD,
                    )?;
                }

                current_buffer_face_println(
                    env,
                    &format!("Lead: {}", old.lead.display_name),
                    JIRASCOPE_FACE_DIFF_OLD,
                )?;
                current_buffer_face_println(env, "New:", JIRASCOPE_FACE_DIFF_ALERT)?;
                let current = state.get_project_detailed(key).unwrap();

                current_buffer_face_println(
                    env,
                    &format!("* {} *", current.key),
                    JIRASCOPE_FACE_DIFF_NEW,
                )?;

                current_buffer_face_println(
                    env,
                    &format!("Name: {}", current.name),
                    JIRASCOPE_FACE_DIFF_NEW,
                )?;
                current_buffer_face_println(
                    env,
                    &format!("Description: {}", current.description),
                    JIRASCOPE_FACE_DIFF_NEW,
                )?;

                if let Some(ref description) = current.url {
                    current_buffer_face_println(
                        env,
                        &format!("URL: {}", description),
                        JIRASCOPE_FACE_DIFF_NEW,
                    )?;
                }

                current_buffer_face_println(
                    env,
                    &format!("Lead: {}", current.lead.display_name),
                    JIRASCOPE_FACE_DIFF_NEW,
                )?;
            }
            _ => {}
        }
        Ok(())
    })?;

    open_jirascope_diff_buffer(env)?;

    Ok(())
}

#[defun]
fn delete_project_interactive(env: &Env) -> Result<Value<'_>> {
    let projects = get_jirascope().get_projects()?;

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
        return utils::nil(env);
    }

    let project_key = projects[index.unwrap()].key.clone();

    workthread_spawn(move || {
        let result = get_jirascope().delete_project(&*project_key);

        if result.is_ok() {
            concurrent::push_command(Box::new(move |env| {
                state::refresh(env)?;

                env.message(format!("Deleted project {}.", project_key).as_str())?;

                Ok(())
            }));
        } else {
            concurrent::push_command(Box::new(|env| {
                env.message("Failed to delete project.")?;

                Ok(())
            }));
        }
    });

    utils::nil(env)
}
