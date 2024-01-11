#![allow(non_snake_case)] // Stops RA from complaining about the Emacs macros.

use std::sync::{Mutex, MutexGuard, OnceLock};

use emacs::{defun, Env, Result};
use jirascope_core::{Auth, Config, Jirascope};

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

static JIRASCOPE: OnceLock<Mutex<Jirascope>> = OnceLock::new();
static JIRASCOPE_BUFFER_NAME: &str = "*jirascope*";
static JIRASCOPE_DIFF_BUFFER_NAME: &str = "*jirascope-diff*";

// Register the initialization hook that Emacs will call when it loads the module.
#[emacs::module]
fn init(env: &Env) -> Result<()> {
    env.call(
        "set",
        (
            env.intern("jirascope-dyn--version")?,
            option_env!("CARGO_PKG_VERSION"),
        ),
    )?;

    concurrent::install_handler(env)?;

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        concurrent::workthread_panic_cleanup();
        // propagate panic to the default handler
        default_hook(info);
    }));
    Ok(())
}

#[defun]
fn setup(url: String, login: String, api_token: String) -> Result<()> {
    let config = Config::new(url);
    let auth = Auth::new(login, api_token);

    let mut jirascope = Jirascope::new(config, auth);
    jirascope.init()?;

    let res = JIRASCOPE.set(Mutex::new(jirascope));

    if res.is_err() {
        panic!("Jirascope already initialized.");
    }

    state::setup(30.0);

    Ok(())
}

fn get_jirascope<'a>() -> MutexGuard<'a, Jirascope> {
    let j = JIRASCOPE
        .get_or_init(|| {
            panic!("Jirascope not setup. Call `jirascope-setup` first.");
        })
        .lock()
        .unwrap();
    j
}
