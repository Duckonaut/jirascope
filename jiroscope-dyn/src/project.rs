use emacs::{defun, Env, Result, Value};
use jiroscope_core::jira::{
    Project, ProjectCategory, ProjectCreate, ProjectCreateDetails, ProjectEdit, PROJECT_TEMPLATES,
    PROJECT_TYPE_KEYS, PROJECT_TYPE_NAMES_TO_TEMPLATE_RANGE,
};

use crate::{
    concurrent, get_jiroscope,
    state::{self, get_state},
    utils::{self, workthread_spawn},
};

fn prompt_select_project(env: &Env) -> Option<Project> {
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

    Some(projects[index].clone())
}

#[defun]
fn create_interactive(env: &Env) -> Result<()> {
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
        return Ok(());
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
        let result = get_jiroscope().create_project(project_create);

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

    let users = get_jiroscope()
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

    let mut categories = get_jiroscope().get_project_categories()?;
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
        Some(0) => Some(jiroscope_core::jira::AssigneeType::ProjectLead),
        Some(1) => Some(jiroscope_core::jira::AssigneeType::Unassigned),
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
        let result = get_jiroscope().edit_project(&*project.key, project_edit);

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
        return utils::nil(env);
    }

    let project_key = projects[index.unwrap()].key.clone();

    workthread_spawn(move || {
        let result = get_jiroscope().delete_project(&*project_key);

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
