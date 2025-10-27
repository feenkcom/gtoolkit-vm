use std::fmt::{Debug, Formatter};
use std::intrinsics::transmute;
use std::os::raw::c_void;
use std::sync::mpsc::{channel, Receiver, RecvError, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::{ApplicationError, Result};

#[derive(Debug, Clone)]
pub enum EventLoopMessage {
    Terminate,
    #[cfg(feature = "ffi")]
    Call(Arc<Mutex<crate::EventLoopCallout>>),
    WakeUp,
}

#[derive(Debug)]
pub struct EventLoop {
    receiver: Receiver<EventLoopMessage>,
}

impl EventLoop {
    pub fn new() -> (Self, Sender<EventLoopMessage>) {
        let (sender, receiver) = channel::<EventLoopMessage>();
        let event_loop = Self { receiver };

        (event_loop, sender)
    }

    pub fn run(&self) -> Result<()> {
        loop {
            match self.receiver.recv() {
                Ok(message) => match self.process_message(message) {
                    Ok(should_continue) => {
                        if !should_continue {
                            break;
                        }
                    }
                    Err(error) => return Err(error),
                },
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }

    pub fn try_recv(&self) -> Result<()> {
        loop {
            match self.receiver.try_recv() {
                Ok(message) => {
                    trace!("Received {:?}", &message);
                    match self.process_message(message) {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                        }
                        Err(error) => {
                            error!("{}", &error);
                            return Err(error);
                        }
                    }
                }
                Err(error) => {
                    return match error {
                        TryRecvError::Empty => Ok(()),
                        TryRecvError::Disconnected => Err(error.into()),
                    };
                }
            }
        }
        Ok(())
    }

    fn process_message(&self, message: EventLoopMessage) -> Result<bool> {
        match message {
            EventLoopMessage::Terminate => {
                return Ok(false);
            }
            EventLoopMessage::WakeUp => {
                // wake up!
            }
            #[cfg(feature = "ffi")]
            EventLoopMessage::Call(callout) => callout.lock().unwrap().call(),
        }
        Ok(true)
    }
}

#[derive(Debug)]
pub struct EventLoopWaker {
    waker: extern "C" fn(*const c_void, u32) -> bool,
    waker_thunk: *const c_void,
}

impl EventLoopWaker {
    pub fn new(
        waker: extern "C" fn(*const c_void, u32) -> bool,
        waker_thunk: *const c_void,
    ) -> Self {
        Self { waker, waker_thunk }
    }

    pub fn wake(&self) -> bool {
        (self.waker)(self.waker_thunk, 0)
    }
}
