use std::sync::{Mutex, MutexGuard, OnceLock};

use emacs::{defun, Env};
use jiroscope_core::jira::{Issue, Project};

use crate::{concurrent, get_jiroscope, utils, JIROSCOPE_BUFFER_NAME};

static STATE: OnceLock<Mutex<State>> = OnceLock::new();

pub struct State {
    projects: Vec<Project>,
    issues: Vec<Issue>,
    dirty: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            issues: Vec::new(),
            dirty: false,
        }
    }

    pub fn projects(&self) -> &[Project] {
        &self.projects
    }

    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    pub fn refresh(&mut self) -> Result<(), jiroscope_core::Error> {
        let new_projects = get_jiroscope().get_projects()?;

        if !new_projects.iter().eq(self.projects.iter()) {
            self.dirty = true;
        }

        self.projects = new_projects;

        let new_issues = get_jiroscope()
            .get_all_issues()
            .map(|issues| issues.issues)?;

        if !new_issues.iter().eq(self.issues.iter()) {
            self.dirty = true;
        }

        self.issues = new_issues;

        Ok(())
    }
}

pub(crate) fn get_state<'a>() -> MutexGuard<'a, State> {
    let s = STATE
        .get_or_init(|| Mutex::new(State::new()))
        .lock()
        .unwrap();
    s
}

pub(crate) fn setup(refresh_interval: f64) {
    let refresh_interval = std::time::Duration::from_secs_f64(refresh_interval);
    std::thread::spawn(move || loop {
        let mut state = get_state();
        match state.refresh() {
            Ok(_) => {
                if state.dirty {
                    concurrent::push_command(Box::new(|env| {
                        update_buffers(env, &get_state());
                        Ok(())
                    }));
                }
            }
            Err(err) => eprintln!("Error refreshing state: {}", err),
        }
        drop(state);
        std::thread::sleep(refresh_interval);
    });
}

pub(crate) fn refresh(env: &Env) -> Result<(), jiroscope_core::Error> {
    let mut state = get_state();
    match state.refresh() {
        Ok(_) => {
            if state.dirty {
                update_buffers(env, &state);
            }
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn update_buffers(env: &Env, state: &State) {
    if let Some(utils::JiroscopeBufferMode::Tree) = utils::get_buffer_mode() {
        utils::with_buffer(env, JIROSCOPE_BUFFER_NAME, |env| {
            env.call("erase-buffer", [])?;

            print_tree(env, state)?;

            Ok(())
        }).unwrap();
    }
}

#[defun]
fn open(env: &emacs::Env) -> emacs::Result<()> {
    let state = get_state();

    utils::set_buffer_mode(utils::JiroscopeBufferMode::Tree);
    utils::open_jiroscope_buffer(env)?;

    env.call("erase-buffer", [])?;

    print_tree(env, &state)?;

    Ok(())
}

fn get_icon(i: usize, len: usize) -> &'static str {
    if i == len - 1 {
        "└"
    } else {
        "├"
    }
}

fn print_tree(env: &emacs::Env, state: &State) -> emacs::Result<()> {
    for project in state.projects() {
        env.call("insert", (format!("{}: {}\n", project.key, project.name),))?;

        let issues = state
            .issues()
            .iter()
            .filter(|i| i.fields.project.key == project.key)
            .collect::<Vec<_>>();

        let size = issues.len();
        for (i, issue) in issues.iter().enumerate() {
            env.call(
                "insert",
                (format!(
                    "{} {}: {} - {}\n",
                    get_icon(i, size),
                    issue.key,
                    issue.fields.summary,
                    issue.fields.status.name
                ),),
            )?;
        }
    }

    Ok(())
}
