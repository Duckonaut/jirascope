use std::{
    collections::VecDeque,
    sync::{atomic::AtomicUsize, RwLock},
};

use emacs::{defun, Env, IntoLisp, Value};

use crate::utils;

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

// This is a bit of a hack. We have two queues, and we swap between them. This
// lets us push further async commands while we're running the current queue.
static mut COMMAND_QUEUE_A: RwLock<VecDeque<CommandEntry>> = RwLock::new(VecDeque::new());
static mut COMMAND_QUEUE_B: RwLock<VecDeque<CommandEntry>> = RwLock::new(VecDeque::new());
static COMMAND_QUEUE_INDEX: AtomicUsize = AtomicUsize::new(0);

fn swap_command_queues() {
    COMMAND_QUEUE_INDEX.store(
        1 - COMMAND_QUEUE_INDEX.load(std::sync::atomic::Ordering::Relaxed),
        std::sync::atomic::Ordering::Relaxed,
    );
}

fn current_command_queue() -> &'static RwLock<VecDeque<CommandEntry>> {
    unsafe {
        if COMMAND_QUEUE_INDEX.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            &COMMAND_QUEUE_A
        } else {
            &COMMAND_QUEUE_B
        }
    }
}

pub(crate) fn push_command(callback: Box<Command>) {
    current_command_queue()
        .write()
        .unwrap()
        .push_back(CommandEntry::new(callback));
}

pub(crate) fn flush_commands(env: &Env) -> emacs::Result<()> {
    let mut queue = current_command_queue().write().unwrap();
    swap_command_queues();
    while let Some(entry) = queue.pop_front() {
        entry.run(env)?;
    }
    Ok(())
}

#[defun]
fn event_handler(env: &Env) -> emacs::Result<()> {
    if utils::workthread_count() > 0 {
        env.message("[jiroscope] Task running...")?;
    }

    flush_commands(env)
}

#[defun]
pub(crate) fn install_handler(env: &Env) -> emacs::Result<Value<'_>> {
    env.call(
        "run-with-timer",
        [
            0.1.into_lisp(env)?,
            0.1.into_lisp(env)?,
            env.intern("jiroscope-dyn-concurrent-event-handler")?,
        ],
    )
}
