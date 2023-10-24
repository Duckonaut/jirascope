use emacs::defun;
use jiroscope_core::jira::{Issue, Project};

use crate::get_jiroscope;

pub struct State {
    projects: Vec<Project>,
    issues: Vec<Issue>,
}

impl State {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            issues: Vec::new(),
        }
    }

    pub fn projects(&self) -> &[Project] {
        &self.projects
    }

    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    pub fn refresh(&mut self) -> Result<bool, jiroscope_core::Error> {
        let mut changed = false;

        let new_projects = get_jiroscope().get_projects()?;

        if !new_projects.iter().eq(self.projects.iter()) {
            changed = true;
        }

        self.projects = new_projects;

        let new_issues = get_jiroscope()
            .get_all_issues()
            .map(|issues| issues.issues)?;

        if !new_issues.iter().eq(self.issues.iter()) {
            changed = true;
        }

        self.issues = new_issues;

        Ok(changed)
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
            Ok(changed) => {
                if changed {
                    println!("State refreshed");
                }
            }
            Err(err) => eprintln!("Error refreshing state: {}", err),
        }
        std::thread::sleep(refresh_interval);
    });
}

#[defun]
fn print() -> emacs::Result<()> {
    let state = get_state();
    println!("Projects: {:?}", state.projects());
    println!("Issues: {:?}", state.issues());

    Ok(())
}
