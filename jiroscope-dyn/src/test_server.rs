use emacs::{Result, Value, Env, IntoLisp};

use crate::get_jiroscope;

#[emacs::defun]
fn create_note(env: &Env, message: String) -> Result<Value<'_>> {
    let note = get_jiroscope().register_note(message)?;

    let id = note.id.unwrap();

    id.into_lisp(env)
}

#[emacs::defun]
fn get_notes(env: &Env) -> Result<Value<'_>> {
    let notes = get_jiroscope().get_notes()?;

    let v = env.make_vector(notes.len(), ())?;

    for (i, note) in notes.iter().enumerate() {
        v.set(i, note.message.clone().into_lisp(env)?)?;
    }

    v.into_lisp(env)
}

#[emacs::defun]
fn get_note_by_id(env: &Env, id: usize) -> Result<Value<'_>> {
    let note = get_jiroscope().get_note_by_id(id)?;

    note.message.into_lisp(env)
}

#[emacs::defun]
fn update_note_by_id(env: &Env, id: usize, message: String) -> Result<Value<'_>> {
    let note = get_jiroscope().update_note_by_id(id, message)?;

    note.message.into_lisp(env)
}
