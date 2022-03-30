use crate::{ApplicationError, Result};
use libffi::high::call;
use libffi::low::{ffi_cif, ffi_type, CodePtr};
use std::ffi::CString;
use std::fmt::{Debug, Formatter};
use std::intrinsics::transmute;
use std::os::raw::c_void;
use std::sync::mpsc::{channel, Receiver, RecvError, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub enum EventLoopMessage {
    Terminate,
    Call(Arc<Mutex<EventLoopCallout>>),
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
            EventLoopMessage::Call(callout) => callout.lock().unwrap().call(),
        }
        Ok(true)
    }
}

#[repr(C)]
pub struct EventLoopCallout {
    pub(crate) function_name: Option<CString>,
    pub(crate) module_name: Option<CString>,
    pub(crate) cif: *mut ffi_cif,
    pub(crate) func: CodePtr,
    pub(crate) args: Option<*mut *mut c_void>,
    pub(crate) result: Option<*mut c_void>,
    pub(crate) callback: Option<Box<dyn FnOnce()>>,
}

impl EventLoopCallout {
    pub fn call(&mut self) {
        unsafe {
            libffi::raw::ffi_call(
                self.cif,
                Some(*unsafe { self.func.as_safe_fun() }),
                self.result.unwrap_or(std::ptr::null_mut()),
                self.args.unwrap_or(std::ptr::null_mut()),
            )
        }
        let callback = self.callback.take().unwrap();
        callback();
    }

    pub fn return_type(&self) -> &ffi_type {
        let cif: &ffi_cif = unsafe { transmute(self.cif) };
        let rtype: &ffi_type = unsafe { transmute(cif.rtype) };
        rtype
    }

    pub fn number_of_arguments(&self) -> usize {
        let cif: &ffi_cif = unsafe { transmute(self.cif) };
        cif.nargs as usize
    }
}

impl Debug for EventLoopCallout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Callout")
            .field("function_name", &self.function_name)
            .field("module_name", &self.module_name)
            .field("cif", &self.cif)
            .field("func", &self.func)
            .field("args", &self.args)
            .field("result", &self.result)
            .finish()
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
