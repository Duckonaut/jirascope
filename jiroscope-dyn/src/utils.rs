use std::{fmt::Display, sync::Mutex};

use emacs::{Env, IntoLisp, Result, Value};

use crate::JIROSCOPE_BUFFER_NAME;

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

pub fn force_prompt_string(env: &Env, prompt: &str) -> emacs::Result<String> {
    let s = prompt_string(env, prompt);

    if let Some(s) = s {
        Ok(s)
    } else {
        Err(jiroscope_core::Error::jiroscope("Empty string not allowed.").into())
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

pub fn open_jiroscope_buffer(env: &Env) -> Result<()> {
    let args = vec![JIROSCOPE_BUFFER_NAME.to_string().into_lisp(env)?];

    let buffer = env.call("get-buffer-create", &args)?;

    let args = vec![buffer];

    env.call("switch-to-buffer", &args)?;

    env.call("erase-buffer", [])?;

    Ok(())
}

pub fn clear_jiroscope_buffer(env: &Env) -> Result<()> {
    let args = vec![JIROSCOPE_BUFFER_NAME.to_string().into_lisp(env)?];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JiroscopeBufferMode {
    Issue,
    Project,
    Tree,
}

static JIROSCOPE_BUFFER_MODE: Mutex<JiroscopeBufferMode> = Mutex::new(JiroscopeBufferMode::Issue);

pub fn set_buffer_mode(mode: JiroscopeBufferMode) {
    let mut buffer_mode = JIROSCOPE_BUFFER_MODE.lock().unwrap();
    *buffer_mode = mode;
}

pub fn get_buffer_mode() -> Option<JiroscopeBufferMode> {
    let buffer_mode = JIROSCOPE_BUFFER_MODE.lock().unwrap();
    Some(*buffer_mode)
}
