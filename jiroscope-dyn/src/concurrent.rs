use std::{collections::VecDeque, sync::RwLock};

use emacs::{defun, Env, Value};

pub(crate) type Command = dyn FnOnce(&Env) -> emacs::Result<()> + Send + 'static;

pub(crate) struct CommandEntry {
    callback: Box<Command>,
}

impl CommandEntry {
    pub(crate) fn new(callback: Box<Command>) -> Self {
        Self { callback }
    }

    pub(crate) fn run(self, env: &Env) -> emacs::Result<()> {
        (self.callback)(env)
    }
}

static mut COMMAND_QUEUE: RwLock<VecDeque<CommandEntry>> = RwLock::new(VecDeque::new());

pub(crate) fn push_command(callback: Box<Command>) {
    unsafe {
        COMMAND_QUEUE
            .write()
            .unwrap()
            .push_back(CommandEntry::new(callback));
    }
}

pub(crate) fn flush_commands(env: &Env) -> emacs::Result<()> {
    let mut queue = unsafe { COMMAND_QUEUE.write().unwrap() };
    while let Some(entry) = queue.pop_front() {
        entry.run(env)?;
    }
    Ok(())
}

#[defun]
fn event_handler(env: &Env) -> emacs::Result<()> {
    flush_commands(env)
}

#[defun]
pub(crate) fn install_handler(env: &Env) -> emacs::Result<Value<'_>> {
    env.call(
        "add-hook",
        [
            env.intern("pre-command-hook")?,
            env.intern("jiroscope-dyn-concurrent-event-handler")?,
        ],
    )
}
