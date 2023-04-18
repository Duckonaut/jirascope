#![allow(non_snake_case)] // Stops RA from complaining about the Emacs macros.

use std::sync::{Mutex, MutexGuard};

use emacs::{defun, Env, IntoLisp, Result, Value};
use jiroscope_core::{
    jira::{Issue, Issues},
    Auth, Config, Jiroscope,
};

// Emacs won't load the module without this.
emacs::plugin_is_GPL_compatible!();

static mut JIROSCOPE: Option<Mutex<Jiroscope>> = None;

// Register the initialization hook that Emacs will call when it loads the module.
#[emacs::module]
fn init(_: &Env) -> Result<()> {
    Ok(())
}

#[defun]
fn setup(url: String, login: String, api_token: String) -> Result<()> {
    let config = Config::new(url);
    let auth = Auth::new(login, api_token);

    let mut jiroscope = Jiroscope::new(config, auth);
    jiroscope.init()?;

    unsafe {
        JIROSCOPE = Some(Mutex::new(jiroscope));
    }

    Ok(())
}

fn get_jiroscope<'a>() -> MutexGuard<'a, Jiroscope> {
    unsafe { JIROSCOPE.as_ref().unwrap() }.lock().unwrap()
}

#[defun]
fn benchmark(env: &Env) -> Result<Value<'_>> {
    let time = std::time::Instant::now();

    for _ in 0..100 {
        get_jiroscope().get_notes()?;
    }
    println!("Rust ureq time: {:?}", time.elapsed());

    let args = vec!["http://localhost:1937/notes".to_string().into_lisp(env)?];

    let time = std::time::Instant::now();
    env.call("benchmark-request-el-jiroscope", &args)?;
    println!("Emacs time: {:?}", time.elapsed());

    ().into_lisp(env)
}

// Define a function callable by Lisp code.
#[defun]
fn create_note(env: &Env, message: String) -> Result<Value<'_>> {
    let note = get_jiroscope().register_note(message)?;

    let id = note.id.unwrap();

    id.into_lisp(env)
}

#[defun]
fn get_notes(env: &Env) -> Result<Value<'_>> {
    let notes = get_jiroscope().get_notes()?;

    let v = env.make_vector(notes.len(), ())?;

    for (i, note) in notes.iter().enumerate() {
        v.set(i, note.message.clone().into_lisp(env)?)?;
    }

    v.into_lisp(env)
}

#[defun]
fn get_note_by_id(env: &Env, id: usize) -> Result<Value<'_>> {
    let note = get_jiroscope().get_note_by_id(id)?;

    note.message.into_lisp(env)
}

#[defun]
fn update_note_by_id(env: &Env, id: usize, message: String) -> Result<Value<'_>> {
    let note = get_jiroscope().update_note_by_id(id, message)?;

    note.message.into_lisp(env)
}

#[defun(user_ptr)]
fn get_issue(_: &Env, issue_key: String) -> Result<Issue> {
    let issue = get_jiroscope().get_issue(&*issue_key)?;

    Ok(issue)
}

#[defun(user_ptr)]
fn get_all_issues(_: &Env) -> Result<Issues> {
    let issues = get_jiroscope().get_all_issues()?;

    Ok(issues)
}

#[defun]
fn get_issue_key<'e>(env: &'e Env, issue: &mut Issue) -> Result<Value<'e>> {
    issue.key.clone().into_lisp(env)
}

#[defun]
fn get_issue_summary<'e>(env: &'e Env, issue: &mut Issue) -> Result<Value<'e>> {
    issue.fields.summary.clone().into_lisp(env)
}

#[defun]
fn display_issue(env: &Env, issue_key: String) -> Result<Value<'_>> {
    let issue = get_jiroscope().get_issue(&*issue_key)?;
    open_jiroscope_buffer(env)?;

    let args = vec![format!("* {} *", issue_key).into_lisp(env)?];

    env.call("insert", &args)?;

    let args = vec![];

    env.call("newline", &args)?;

    let args = vec![format!("Summary: {}", issue.fields.summary).into_lisp(env)?];

    env.call("insert", &args)?;

    let args = vec![];

    env.call("newline", &args)?;

    if let Some(description) = issue.fields.description {
        let args = vec![format!("Description: {}", description).into_lisp(env)?];

        env.call("insert", &args)?;

        let args = vec![];

        env.call("newline", &args)?;
    }

    let args = vec![format!("Status: {}", issue.fields.status.name).into_lisp(env)?];

    env.call("insert", &args)?;

    let args = vec![];

    env.call("newline", &args)?;

    ().into_lisp(env)
}

#[defun]
fn open_jiroscope_buffer(env: &Env) -> Result<Value<'_>> {
    let args = vec!["*jiroscope*".to_string().into_lisp(env)?];

    let buffer = env.call("get-buffer-create", &args)?;

    let args = vec![buffer];

    env.call("switch-to-buffer", &args)?;

    env.call("erase-buffer", [])?;

    ().into_lisp(env)
}
