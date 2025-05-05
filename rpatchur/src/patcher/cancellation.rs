use std::sync::mpsc;
use std::fmt;
use crate::patcher::PatcherCommand;

#[derive(Debug)]
pub enum InterruptibleFnError {
    Err(String),
    Interrupted,
}

impl fmt::Display for InterruptibleFnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterruptibleFnError::Err(msg) => write!(f, "{}", msg),
            InterruptibleFnError::Interrupted => write!(f, "Operation was interrupted"),
        }
    }
}

pub type InterruptibleFnResult<T> = Result<T, InterruptibleFnError>;

pub fn process_incoming_commands(
    patching_thread_rx: &mut mpsc::Receiver<PatcherCommand>,
) -> Result<(), InterruptibleFnError> {
    match patching_thread_rx.try_recv() {
        Ok(PatcherCommand::CancelUpdate) => Err(InterruptibleFnError::Interrupted),
        Ok(PatcherCommand::Quit) => Err(InterruptibleFnError::Interrupted),
        Ok(_) => Ok(()),
        Err(mpsc::TryRecvError::Empty) => Ok(()),
        Err(mpsc::TryRecvError::Disconnected) => Err(InterruptibleFnError::Interrupted),
    }
}

pub async fn wait_for_cancellation(
    patching_thread_rx: &mut mpsc::Receiver<PatcherCommand>,
) -> InterruptibleFnError {
    match patching_thread_rx.recv() {
        Ok(PatcherCommand::CancelUpdate) => InterruptibleFnError::Interrupted,
        Ok(PatcherCommand::Quit) => InterruptibleFnError::Interrupted,
        Ok(_) => InterruptibleFnError::Interrupted,
        Err(_) => InterruptibleFnError::Interrupted,
    }
}
