use crate::error::{ApplicationError, Result};
use std::sync::mpsc::{channel, Receiver, RecvError, Sender};
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub enum EventLoopMessage {
    Terminate,
    //Call(GToolkitCallout),
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

    pub fn run(mut self) -> Result<()> {
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

    fn process_message(&mut self, message: EventLoopMessage) -> Result<bool> {
        match message {
            EventLoopMessage::Terminate => {
                return Ok(false);
            }
            EventLoopMessage::WakeUp => {
                // wake up!
            }
        }
        Ok(true)
    }
}
