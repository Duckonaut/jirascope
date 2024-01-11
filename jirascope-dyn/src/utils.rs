use std::{fmt::Display, sync::Mutex};

use emacs::{Env, IntoLisp, Result, Value};

use crate::{concurrent, state, JIRASCOPE_BUFFER_NAME, JIRASCOPE_DIFF_BUFFER_NAME};

pub(crate) static JIRASCOPE_FACE_DIFF_ALERT: &str = "jirascope-diff-alert";
pub(crate) static JIRASCOPE_FACE_DIFF_NEW: &str = "jirascope-diff-new";
pub(crate) static JIRASCOPE_FACE_DIFF_OLD: &str = "jirascope-diff-old";

pub fn nil(env: &Env) -> Result<Value<'_>> {
    ().into_lisp(env)
}

pub fn dump_string_to_buffer(env: &Env, buffer_name: &str, string: &str) {
    let args = vec![buffer_name.to_string().into_lisp(env).unwrap()];
    let buffer = env.call("get-buffer-create", &args).unwrap();

    let args = vec![buffer.into_lisp(env).unwrap()];
    env.call("with-current-buffer", &args).unwrap();

    let args = vec![string.to_string().into_lisp(env).unwrap()];
    env.call("insert", &args).unwrap();

    let args = vec!["\n".to_string().into_lisp(env).unwrap()];
    env.call("insert", &args).unwrap();

    let args = vec![buffer.into_lisp(env).unwrap()];
    env.call("switch-to-buffer", &args).unwrap();
}

pub trait AsRow {
    fn as_row(&self) -> Vec<String>;
}

impl AsRow for Vec<String> {
    fn as_row(&self) -> Vec<String> {
        self.clone()
    }
}

macro_rules! all_tuples {
    ($m:tt) => {
        $m!(1, (T1,));
        $m!(2, (T1, T2));
        $m!(3, (T1, T2, T3));
        $m!(4, (T1, T2, T3, T4));
        $m!(5, (T1, T2, T3, T4, T5));
        $m!(6, (T1, T2, T3, T4, T5, T6));
        $m!(7, (T1, T2, T3, T4, T5, T6, T7));
        $m!(8, (T1, T2, T3, T4, T5, T6, T7, T8));
    };
}

macro_rules! impl_as_row {
    ($n:tt, ($t:ident $(,$tailt:ident)* $(,)?)) => {
        impl<$t: std::fmt::Display, $($tailt: std::fmt::Display),*> AsRow for ($t, $($tailt),*) {
            fn as_row(&self) -> Vec<String> {
                let mut row = Vec::new();
                let ($t, $($tailt),*) = self;
                row.push(format!("{}", $t));
                $(
                    row.push(format!("{}", $tailt));
                )*
                row
            }
        }
    }
}

all_tuples!(impl_as_row);

pub fn tuples_to_md_table<T: AsRow>(titles: &[&str], tuples: &[T]) -> String {
    let mut buffer = Vec::new();

    let mut writer = std::io::Cursor::new(&mut buffer);

    write_tuples_to_md_table(&mut writer, titles, tuples).unwrap();

    String::from_utf8(buffer).unwrap()
}

pub fn write_tuples_to_md_table<T: AsRow>(
    w: &mut impl std::io::Write,
    titles: &[&str],
    tuples: &[T],
) -> std::io::Result<()> {
    write!(w, "| ")?;
    for title in titles {
        write!(w, "{}", title)?;
        write!(w, " | ")?;
    }
    writeln!(w)?;

    write!(w, "|")?;
    for _ in titles {
        write!(w, " --- |")?;
    }
    writeln!(w)?;

    for tuple in tuples {
        let row = tuple.as_row();
        write!(w, "| ")?;
        for cell in row {
            write!(w, "{} |", cell)?;
        }
        writeln!(w)?;
    }

    Ok(())
}

pub fn write_tuples_to_pyplot_data<T: AsRow>(
    w: &mut impl std::io::Write,
    titles: &[&str],
    tuples: &[T],
) -> std::io::Result<()> {
    write!(w, "labels = [")?;
    for title in titles {
        write!(w, "'{}', ", title)?;
    }
    writeln!(w, "]")?;

    write!(w, "data = [")?;
    for tuple in tuples {
        let row = tuple.as_row();
        write!(w, "[")?;
        for cell in row {
            write!(w, "{}, ", cell)?;
        }
        write!(w, "], ")?;
    }
    writeln!(w, "]")?;

    Ok(())
}

pub fn prompt_select_index(
    env: &Env,
    prompt: &str,
    choices: &[impl AsRef<str> + Display + PartialEq<String>],
) -> Option<usize> {
    let options = env
        .list(
            choices
                .iter()
                .map(|c| c.to_string().into_lisp(env).unwrap())
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .unwrap();

    let args = vec![prompt.to_string().into_lisp(env).unwrap(), options];
    let choice = env.call("completing-read", &args).unwrap();

    let choice = choice.into_rust::<String>().unwrap();

    choices.iter().position(|x| *x == choice)
}

pub fn prompt_string(env: &Env, prompt: &str) -> Option<String> {
    let args = vec![prompt.to_string().into_lisp(env).unwrap()];
    let choice = env.call("read-string", &args).unwrap();

    let s = choice.into_rust::<String>().ok();

    s.filter(|s| !s.is_empty())
}

pub fn prompt_force_change<'a>(env: &Env, reason: impl Into<&'a str>) -> Result<bool> {
    let choice = env.call(
        "y-or-n-p",
        [format!("{}, force change? ", reason.into())
            .into_lisp(env)
            .unwrap()],
    )?;

    // y-or-n-p returns t for yes and nil for no
    Ok(choice.is_not_nil())
}

pub fn signal_result_async<T, E>(
    result: std::result::Result<T, E>,
    on_success: &'static str,
    on_failure: &'static str,
) {
    if result.is_ok() {
        concurrent::push_command(Box::new(move |env| {
            state::refresh(env)?;

            env.message(on_success)?;

            Ok(())
        }));
    } else {
        concurrent::push_command(Box::new(move |env| {
            env.message(on_failure)?;

            Ok(())
        }));
    }
}

pub fn signal_result<T, E>(
    env: &Env,
    result: std::result::Result<T, E>,
    on_success: &'static str,
    on_failure: &'static str,
) -> emacs::Result<()> {
    if result.is_ok() {
        state::refresh(env)?;

        env.message(on_success)?;
    } else {
        env.message(on_failure)?;
    }

    Ok(())
}

pub fn force_prompt_string(env: &Env, prompt: &str) -> emacs::Result<String> {
    let s = prompt_string(env, prompt);

    if let Some(s) = s {
        Ok(s)
    } else {
        Err(jirascope_core::Error::jirascope("Empty string not allowed.").into())
    }
}

pub fn get_current_buffer_name(env: &Env) -> Result<String> {
    let buffer = env.call("current-buffer", [])?;
    let name = env.call("buffer-name", [buffer])?;
    name.into_rust()
}

pub fn goto_buffer(env: &Env, buffer_name: &str) -> Result<()> {
    let args = vec![buffer_name.to_string().into_lisp(env)?];
    let buffer = env.call("get-buffer", &args)?;
    let args = vec![buffer];
    env.call("switch-to-buffer", &args)?;
    Ok(())
}

pub fn open_jirascope_buffer(env: &Env) -> Result<()> {
    let buffer = env.call(
        "get-buffer-create",
        [JIRASCOPE_BUFFER_NAME.to_string().into_lisp(env)?],
    )?;

    env.call("switch-to-buffer", [buffer])?;
    env.call("set", [env.intern("buffer-read-only")?, nil(env)?])?;

    env.call("erase-buffer", [])?;

    Ok(())
}

pub fn open_jirascope_diff_buffer(env: &Env) -> Result<()> {
    let buffer = env.call(
        "get-buffer-create",
        [JIRASCOPE_DIFF_BUFFER_NAME.to_string().into_lisp(env)?],
    )?;

    env.call("display-buffer-in-side-window", [buffer, nil(env)?])?;

    Ok(())
}

pub fn close_jirascope_diff_buffer(env: &Env) -> Result<()> {
    let buffer = env.call(
        "get-buffer-create",
        [JIRASCOPE_DIFF_BUFFER_NAME.to_string().into_lisp(env)?],
    )?;

    env.call("kill-buffer", [buffer])?;

    Ok(())
}

pub fn get_jirascope_buffer_content(env: &Env) -> Result<String> {
    with_buffer(env, JIRASCOPE_BUFFER_NAME, |env| {
        let args = vec![];
        let content = env.call("buffer-string", &args)?;
        content.into_rust()
    })
}

pub fn clear_jirascope_buffer(env: &Env) -> Result<()> {
    let args = vec![JIRASCOPE_BUFFER_NAME.to_string().into_lisp(env)?];

    let buffer = env.call("get-buffer-create", &args)?;

    let args = vec![buffer];

    env.call("erase-buffer", &args)?;

    Ok(())
}

pub fn with_buffer<T, F: FnOnce(&Env) -> Result<T>>(
    env: &Env,
    buffer_name: &str,
    f: F,
) -> Result<T> {
    let args = vec![buffer_name.to_string().into_lisp(env)?];
    let buffer = env.call("get-buffer-create", &args)?;
    let args = vec![buffer.into_lisp(env)?];
    env.call("set-buffer", &args)?;
    f(env)
}

pub fn current_buffer_print(env: &Env, s: &str) -> Result<()> {
    env.call("insert", [s.to_string().into_lisp(env)?])?;
    Ok(())
}

pub fn current_buffer_println(env: &Env, s: &str) -> Result<()> {
    current_buffer_print(env, s)?;
    env.call("newline", [])?;
    Ok(())
}

pub fn current_buffer_face_print(env: &Env, s: &str, face: &str) -> Result<()> {
    let len = s.len();
    let current_point = env.call("point", [])?.into_rust::<i64>()? - 1;
    current_buffer_print(env, s)?;
    let overlay = env.call(
        "make-overlay",
        [
            current_point.into_lisp(env)?,
            (current_point + 1 + len as i64).into_lisp(env)?,
        ],
    )?;

    env.call(
        "overlay-put",
        [overlay, env.intern("face")?, env.intern(face)?],
    )?;

    Ok(())
}

pub fn current_buffer_face_println(env: &Env, s: &str, face: &str) -> Result<()> {
    current_buffer_face_print(env, s, face)?;
    env.call("newline", [])?;
    Ok(())
}

pub fn current_buffer_button(env: &Env, s: &str, button_type: &str) -> Result<()> {
    env.call(
        "jirascope-insert-button",
        [s.to_string().into_lisp(env)?, env.intern(button_type)?],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JirascopeBufferMode {
    Issue,
    Project,
    Tree,
    IssueEdit,
}

static JIRASCOPE_BUFFER_MODE: Mutex<JirascopeBufferMode> = Mutex::new(JirascopeBufferMode::Issue);

pub fn set_buffer_mode(env: &Env, mode: JirascopeBufferMode) -> Result<()> {
    let mut buffer_mode = JIRASCOPE_BUFFER_MODE.lock().unwrap();
    *buffer_mode = mode;

    with_buffer(env, JIRASCOPE_BUFFER_NAME, |env| {
        match mode {
            JirascopeBufferMode::IssueEdit => {
                env.call("set", [env.intern("buffer-read-only")?, nil(env)?])?;
            }

            _ => {
                env.call(
                    "set",
                    [env.intern("buffer-read-only")?, "t".into_lisp(env)?],
                )?;
            }
        }
        Ok(())
    })?;

    Ok(())
}

pub fn get_buffer_mode() -> Option<JirascopeBufferMode> {
    let buffer_mode = JIRASCOPE_BUFFER_MODE.lock().unwrap();
    Some(*buffer_mode)
}

pub struct ScopeCleaner<T: FnMut()> {
    f: T,
}

impl<T: FnMut()> ScopeCleaner<T> {
    pub fn new(f: T) -> Self {
        Self { f }
    }
}

impl<T: FnMut()> Drop for ScopeCleaner<T> {
    fn drop(&mut self) {
        (self.f)();
    }
}
