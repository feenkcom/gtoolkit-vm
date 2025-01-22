use crate::{ObjectFormat, ObjectHeader, RawObjectPointer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Expected an object, found an immediate value {0:?}")]
    NotAnObject(RawObjectPointer),
    #[error("Expected an immediate value, found an object {0:?}")]
    NotAnImmediate(RawObjectPointer),
    #[error("Expected an array, got {0:?} instead")]
    NotAnArray(ObjectFormat),
    #[error("Forwarded object ({0:?}) is not supported for this operation")]
    ForwardedUnsupported(ObjectHeader),
    #[error("Expected an object of type {0}")]
    InvalidType(String),
    #[error(
        "Object {object:?} has a wrong amount of slots; expected {expected} but got {actual}."
    )]
    WrongAmountOfSlots {
        object: ObjectHeader,
        expected: usize,
        actual: usize,
    },
}

pub type Result<T> = core::result::Result<T, Error>;
