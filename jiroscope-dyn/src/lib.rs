#![allow(non_snake_case)] // Stops RA from complaining about the Emacs macros.

use std::sync::{Mutex, MutexGuard, OnceLock};

use emacs::{defun, Env, Result};
use jiroscope_core::{Auth, Config, Jiroscope};

#[cfg(feature = "benchmark")]
mod benchmark;
mod concurrent;
mod issue;
mod project;
mod state;
#[cfg(feature = "test_server")]
mod test_server;
#[allow(dead_code)]
mod utils;

// Emacs won't load the module without this.
emacs::plugin_is_GPL_compatible!();

static JIROSCOPE: OnceLock<Mutex<Jiroscope>> = OnceLock::new();
static JIROSCOPE_BUFFER_NAME: &str = "*jiroscope*";

// Register the initialization hook that Emacs will call when it loads the module.
#[emacs::module]
fn init(env: &Env) -> Result<()> {
    env.call(
        "set",
        (
            env.intern("jiroscope-dyn--version")?,
            option_env!("CARGO_PKG_VERSION"),
        ),
    )?;

    concurrent::install_handler(env)?;
    Ok(())
}

#[defun]
fn setup(url: String, login: String, api_token: String) -> Result<()> {
    let config = Config::new(url);
    let auth = Auth::new(login, api_token);

    let mut jiroscope = Jiroscope::new(config, auth);
    jiroscope.init()?;

    let res = JIROSCOPE.set(Mutex::new(jiroscope));

    if res.is_err() {
        panic!("Jiroscope already initialized.");
    }

    state::setup(30.0);

    Ok(())
}

fn get_jiroscope<'a>() -> MutexGuard<'a, Jiroscope> {
    let j = JIROSCOPE
        .get_or_init(|| {
            panic!("Jiroscope not initialized. Call `jiroscope-dyn--setup` first.");
        })
        .lock()
        .unwrap();
    j
}
