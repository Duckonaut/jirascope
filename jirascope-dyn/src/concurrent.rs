use std::sync::{atomic::AtomicUsize, OnceLock};

use emacs::{defun, Env, IntoLisp, Value};

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

static mut COMMAND_QUEUE_RECEIVER: OnceLock<std::sync::mpsc::Receiver<CommandEntry>> =
    OnceLock::new();
static mut COMMAND_QUEUE_SENDER: OnceLock<std::sync::mpsc::Sender<CommandEntry>> = OnceLock::new();
static BATCH_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

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
    if workthread_count() > 0 {
        env.message("[jirascope] Task running...")?;
    }

    Ok(())
}

#[defun]
pub(crate) fn install_handler(env: &Env) -> emacs::Result<Value<'_>> {
    let noninteractive = env.call("eval", [env.intern("noninteractive")?])?;
    if noninteractive.is_not_nil() {
        BATCH_MODE.store(true, std::sync::atomic::Ordering::SeqCst);
    }

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

static WORKTHREAD_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn workthread_panic_cleanup() {
    WORKTHREAD_COUNTER.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
}

pub fn workthread_spawn<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> Option<std::thread::JoinHandle<T>> {

    WORKTHREAD_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let handle = std::thread::spawn(move || {
        let result = f();

        WORKTHREAD_COUNTER.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

        result
    });

    if BATCH_MODE.load(std::sync::atomic::Ordering::SeqCst) {
        handle.join().unwrap();
        None
    } else {
        Some(handle)
    }
}

pub fn workthread_count() -> usize {
    WORKTHREAD_COUNTER.load(std::sync::atomic::Ordering::SeqCst)
}
