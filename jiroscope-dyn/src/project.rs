use std::thread;

use emacs::{defun, Env, IntoLisp, Result, Value};
use jiroscope_core::jira::{
    ProjectCreate, ProjectCreateDetails, PROJECT_TEMPLATES, PROJECT_TYPE_KEYS,
    PROJECT_TYPE_NAMES_TO_TEMPLATE_RANGE,
};

use crate::{concurrent, get_jiroscope, state, utils};

#[defun]
fn create_interactive(env: &Env) -> Result<Value<'_>> {
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
        return utils::nil(env);
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
            return utils::nil(env);
        }

        Some(project_categories[index.unwrap()].id)
    };

    let index = utils::prompt_select_index(
        env,
        "Choose project type: ",
        PROJECT_TYPE_KEYS.to_vec().as_slice(),
    );

    if index.is_none() {
        return utils::nil(env);
    }

    let project_type_key = PROJECT_TYPE_KEYS[index.unwrap()].to_string();

    let index = utils::prompt_select_index(
        env,
        "Choose assignee type: ",
        ["Project Lead", "Unassigned"].to_vec().as_slice(),
    );

    if index.is_none() {
        return utils::nil(env);
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
        return utils::nil(env);
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

    thread::spawn(move || {
        let result = get_jiroscope().create_project(project_create);

        if result.is_ok() {
            concurrent::push_command(Box::new(|env| {
                state::refresh(env)?;

                env.call("message", ["Project created successfully.".into_lisp(env)?])?;

                Ok(())
            }));
        } else {
            concurrent::push_command(Box::new(|env| {
                env.call("message", ["Failed to create project.".into_lisp(env)?])?;

                Ok(())
            }));
        }
    });

    utils::nil(env)
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

    thread::spawn(move || {
        let result = get_jiroscope().delete_project(&*project_key);

        if result.is_ok() {
            concurrent::push_command(Box::new(move |env| {
                state::refresh(env)?;

                env.call(
                    "message",
                    [format!("Deleted project {}.", project_key).into_lisp(env)?],
                )?;

                Ok(())
            }));
        } else {
            concurrent::push_command(Box::new(|env| {
                env.call("message", ["Failed to delete project.".into_lisp(env)?])?;

                Ok(())
            }));
        }
    });

    utils::nil(env)
}
