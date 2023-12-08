use std::sync::{Mutex, MutexGuard, OnceLock};

use emacs::{defun, Env};
use jiroscope_core::jira::{Issue, Project, ProjectDetailed};

use crate::{
    concurrent, get_jiroscope,
    utils::{self, current_buffer_button, current_buffer_print, current_buffer_println},
    JIROSCOPE_BUFFER_NAME,
};

static STATE: OnceLock<Mutex<State>> = OnceLock::new();

pub(crate) trait ConflictAware: Sized {
    type Key;
    fn key(&self) -> Self::Key;
    fn lookup(values: &[Self], key: &Self::Key) -> Option<Self>;
    fn has_changed(&self, other: &Self) -> bool;
}

impl ConflictAware for Issue {
    type Key = String;
    fn key(&self) -> Self::Key {
        self.key.clone()
    }
    fn lookup(values: &[Self], key: &Self::Key) -> Option<Self> {
        values.iter().find(|i| i.key == *key).cloned()
    }
    fn has_changed(&self, other: &Self) -> bool {
        self.fields.updated != other.fields.updated
    }
}

impl ConflictAware for ProjectDetailed {
    type Key = String;
    fn key(&self) -> Self::Key {
        self.key.clone()
    }
    fn lookup(values: &[Self], key: &Self::Key) -> Option<Self> {
        values.iter().find(|i| i.key == *key).cloned()
    }
    fn has_changed(&self, other: &Self) -> bool {
        self != other
    }
}

pub(crate) enum ConflictCell<T: ConflictAware> {
    Empty,
    Armed { key: T::Key },
    Outdated { key: T::Key, old: T },
    Deleted { key: T::Key },
}

pub struct State {
    projects: Vec<ProjectDetailed>,
    issues: Vec<Issue>,
    dirty: bool,
    issue_rentcell: ConflictCell<Issue>,
    project_rentcell: ConflictCell<ProjectDetailed>,
}

impl State {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            issues: Vec::new(),
            dirty: false,
            issue_rentcell: ConflictCell::Empty,
            project_rentcell: ConflictCell::Empty,
        }
    }

    pub fn projects(&self) -> &[ProjectDetailed] {
        &self.projects
    }

    pub fn get_project_detailed(&self, key: &str) -> Option<ProjectDetailed> {
        ProjectDetailed::lookup(&self.projects, &key.to_string())
    }

    pub fn get_project(&self, key: &str) -> Option<Project> {
        ProjectDetailed::lookup(&self.projects, &key.to_string()).map(ProjectDetailed::to_project)
    }

    pub fn issues(&self) -> &[Issue] {
        &self.issues
    }

    pub fn get_issue(&self, key: &str) -> Option<Issue> {
        Issue::lookup(&self.issues, &key.to_string())
    }

    pub(crate) fn get_current_work_project(&self) -> &ConflictCell<ProjectDetailed> {
        &self.project_rentcell
    }

    pub(crate) fn get_current_work_issue(&self) -> &ConflictCell<Issue> {
        &self.issue_rentcell
    }

    pub fn check_out_issue(&mut self, issue_key: String) -> Result<(), jiroscope_core::Error> {
        match self.issue_rentcell {
            ConflictCell::Empty => {
                self.issue_rentcell = ConflictCell::Armed { key: issue_key };
                Ok(())
            }
            _ => Err(jiroscope_core::Error::jiroscope(
                "Issue already checked out.",
            )),
        }
    }

    pub fn check_out_project(&mut self, project_key: String) -> Result<(), jiroscope_core::Error> {
        match self.project_rentcell {
            ConflictCell::Empty => {
                self.project_rentcell = ConflictCell::Armed { key: project_key };
                Ok(())
            }
            _ => Err(jiroscope_core::Error::jiroscope(
                "Project already checked out.",
            )),
        }
    }

    pub fn try_return_issue<'a>(&mut self, issue_key: impl Into<&'a str>) -> bool {
        match self.issue_rentcell {
            ConflictCell::Armed { ref key } if key == issue_key.into() => {
                self.issue_rentcell = ConflictCell::Empty;
                true
            }
            _ => false,
        }
    }

    pub fn return_issue(&mut self) {
        self.issue_rentcell = ConflictCell::Empty;
    }

    pub fn try_return_project<'a>(&mut self, project_key: impl Into<&'a str>) -> bool {
        match self.project_rentcell {
            ConflictCell::Armed { ref key } if key == project_key.into() => {
                self.project_rentcell = ConflictCell::Empty;
                true
            }
            _ => false,
        }
    }

    pub fn return_project(&mut self) {
        self.project_rentcell = ConflictCell::Empty;
    }

    pub fn refresh(&mut self) -> Result<(), jiroscope_core::Error> {
        let new_projects = get_jiroscope().get_projects()?;

        if !new_projects.iter().eq(self.projects.iter()) {
            self.dirty = true;

            if let ConflictCell::Armed { ref key } = self.project_rentcell {
                if let Some(project) = ProjectDetailed::lookup(&new_projects, key) {
                    let old =
                        ProjectDetailed::lookup(&self.projects, key).expect("Project not found");
                    if project.has_changed(&old) {
                        self.project_rentcell = ConflictCell::Outdated {
                            key: key.clone(),
                            old,
                        };
                    }
                } else {
                    self.project_rentcell = ConflictCell::Deleted { key: key.clone() };
                }
            }
        }

        self.projects = new_projects;

        let new_issues = get_jiroscope()
            .get_all_issues()
            .map(|issues| issues.issues)?;

        if !new_issues.iter().eq(self.issues.iter()) {
            self.dirty = true;

            if let ConflictCell::Armed { ref key } = self.issue_rentcell {
                if let Some(issue) = Issue::lookup(&new_issues, key) {
                    let old = Issue::lookup(&self.issues, key).expect("Issue not found");
                    if issue.has_changed(&old) {
                        self.issue_rentcell = ConflictCell::Outdated {
                            key: key.clone(),
                            old,
                        };
                    }
                } else {
                    self.issue_rentcell = ConflictCell::Deleted { key: key.clone() };
                }
            }
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
        })
        .unwrap();
    }
}

#[defun]
pub fn open(env: &emacs::Env) -> emacs::Result<()> {
    let state = get_state();

    utils::set_buffer_mode(env, utils::JiroscopeBufferMode::Tree)?;
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
        current_buffer_button(env, &project.key, "jiroscope-project-button")?;
        current_buffer_println(env, &format!(": {}", project.name))?;

        let mut issues = state
            .issues()
            .iter()
            .filter(|i| i.fields.project.key == project.key && i.fields.parent.is_none())
            .collect::<Vec<_>>();

        issues.sort_by_key(|i| &i.id);

        let mut subtask_issues = state
            .issues()
            .iter()
            .filter(|i| i.fields.project.key == project.key && i.fields.parent.is_some())
            .collect::<Vec<_>>();

        subtask_issues.sort_by_key(|i| &i.id);

        let size = issues.len();
        for (i, issue) in issues.iter().enumerate() {
            current_buffer_print(env, &format!("{} ", get_icon(i, size)))?;

            current_buffer_button(env, &issue.key, "jiroscope-issue-button")?;
            current_buffer_println(
                env,
                &format!(": {} - {}", issue.fields.summary, issue.fields.status.name),
            )?;

            let issue_subtasks = subtask_issues
                .iter()
                .filter(|i| i.fields.parent.as_ref().unwrap().id == issue.id)
                .collect::<Vec<_>>();

            let subtask_size = issue_subtasks.len();

            for (i, subtask) in issue_subtasks.iter().enumerate() {
                current_buffer_print(env, &format!("  {} ", get_icon(i, subtask_size)))?;

                current_buffer_button(env, &subtask.key, "jiroscope-issue-button")?;
                current_buffer_println(
                    env,
                    &format!(
                        ": {} - {}",
                        subtask.fields.summary, subtask.fields.status.name
                    ),
                )?;
            }
        }
    }

    Ok(())
}
