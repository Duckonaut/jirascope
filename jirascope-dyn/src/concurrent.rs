use std::sync::OnceLock;

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
// static mut COMMAND_QUEUE_A: RwLock<VecDeque<CommandEntry>> = RwLock::new(VecDeque::new());
// static mut COMMAND_QUEUE_B: RwLock<VecDeque<CommandEntry>> = RwLock::new(VecDeque::new());
// static COMMAND_QUEUE_INDEX: AtomicUsize = AtomicUsize::new(0);

static mut COMMAND_QUEUE_RECEIVER: OnceLock<std::sync::mpsc::Receiver<CommandEntry>> =
    OnceLock::new();
static mut COMMAND_QUEUE_SENDER: OnceLock<std::sync::mpsc::Sender<CommandEntry>> = OnceLock::new();

pub(crate) fn push_command(callback: Box<Command>) {
    let sender = unsafe { COMMAND_QUEUE_SENDER.get().cloned().unwrap() };

    sender.send(CommandEntry::new(callback)).unwrap();
}

#[defun]
fn event_handler(env: &Env) -> emacs::Result<()> {
    let receiver = unsafe { COMMAND_QUEUE_RECEIVER.get().unwrap() };
    loop {
        match receiver.try_recv() {
            Ok(entry) => entry.run(env)?,
            Err(std::sync::mpsc::TryRecvError::Empty) => break,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                panic!("Command queue disconnected");
            }
        }
    }
    if utils::workthread_count() > 0 {
        env.message("[jirascope] Task running...")?;
    }

    Ok(())
}

#[defun]
pub(crate) fn install_handler(env: &Env) -> emacs::Result<Value<'_>> {
    unsafe {
        let (sender, receiver) = std::sync::mpsc::channel();
        COMMAND_QUEUE_RECEIVER.set(receiver).unwrap();
        COMMAND_QUEUE_SENDER.set(sender).unwrap();
    }
    env.call(
        "run-with-timer",
        [
            0.1.into_lisp(env)?,
            0.1.into_lisp(env)?,
            env.intern("jirascope-dyn-concurrent-event-handler")?,
        ],
    )
}
