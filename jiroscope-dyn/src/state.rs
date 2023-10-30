use emacs::defun;
use jiroscope_core::jira::{Issue, Project};

use crate::{concurrent, get_jiroscope, utils, JIROSCOPE_BUFFER_NAME};

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

pub(crate) fn get_state() -> &'static mut State {
    static mut STATE: Option<State> = None;
    unsafe {
        if STATE.is_none() {
            STATE = Some(State::new());
        }
        STATE.as_mut().unwrap()
    }
}

pub(crate) fn setup(refresh_interval: f64) {
    let state = get_state();
    let refresh_interval = std::time::Duration::from_secs_f64(refresh_interval);
    std::thread::spawn(move || loop {
        match state.refresh() {
            Ok(_) => {
                if state.dirty {
                    if let Some(utils::JiroscopeBufferMode::Tree) = utils::get_buffer_mode() {
                        concurrent::push_command(Box::new(|env| {
                            utils::with_buffer(env, JIROSCOPE_BUFFER_NAME, |env| {
                                env.call("erase-buffer", [])?;

                                print_tree(env, get_state())
                            })
                        }));
                    }
                }
            }
            Err(err) => eprintln!("Error refreshing state: {}", err),
        }
        std::thread::sleep(refresh_interval);
    });
}

#[defun]
fn open(env: &emacs::Env) -> emacs::Result<()> {
    let state = get_state();

    utils::set_buffer_mode(utils::JiroscopeBufferMode::Tree);
    utils::open_jiroscope_buffer(env)?;

    env.call("erase-buffer", [])?;

    print_tree(env, state)?;

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
            .filter(|i| i.fields.project.key == project.key);

        let size = issues.size_hint().0;
        for (i, issue) in issues.enumerate() {
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
