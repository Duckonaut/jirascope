#![allow(non_snake_case)] // Stops RA from complaining about the Emacs macros.

use emacs::{defun, Env, Result, Value, IntoLisp};
use jiroscope_core::Jiroscope;

// Emacs won't load the module without this.
emacs::plugin_is_GPL_compatible!();

static JIROSCOPE: Jiroscope = Jiroscope::new();

// Register the initialization hook that Emacs will call when it loads the module.
#[emacs::module]
fn init(_: &Env) -> Result<()> {
    Ok(())
}

#[defun]
fn benchmark(env: &Env) -> Result<Value<'_>> {
    let time = std::time::Instant::now();

    for _ in 0..100 {
        JIROSCOPE.get_notes()?;
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
    let note = JIROSCOPE.register_note(message)?;

    let id = note.id.unwrap();

    id.into_lisp(env)
}

#[defun]
fn get_notes(env: &Env) -> Result<Value<'_>> {
    let notes = JIROSCOPE.get_notes()?;

    let v = env.make_vector(notes.len(), ())?;

    for (i, note) in notes.iter().enumerate() {
        v.set(i, note.message.clone().into_lisp(env)?)?;
    }

    v.into_lisp(env)
}

#[defun]
fn get_note_by_id(env: &Env, id: usize) -> Result<Value<'_>> {
    let note = JIROSCOPE.get_note_by_id(id)?;

    note.message.into_lisp(env)
}

#[defun]
fn update_note_by_id(env: &Env, id: usize, message: String) -> Result<Value<'_>> {
    let note = JIROSCOPE.update_note_by_id(id, message)?;

    note.message.into_lisp(env)
}
