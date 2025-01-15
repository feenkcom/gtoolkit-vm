use crate::{Object, ObjectFormat, ObjectHeader, ObjectRef, RawObjectPointer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Expected an object, found an immediate value {0:?}")]
    NotAnObject(RawObjectPointer),
    #[error("Expected an immediate value, found an object {0:?}")]
    NotAnImmediate(RawObjectPointer),
    #[error("Expected an array, got {object_ref:?} with {object_format:?} instead")]
    NotAnArray {
        object_ref: ObjectRef,
        object_format: ObjectFormat,
    },
    #[error("Forwarded object ({0:?} {1:?}) is not supported for this operation")]
    ForwardedUnsupported(ObjectRef, ObjectHeader),
    #[error("Expected an object of type {0}")]
    InvalidType(String),
    #[error(
        "Object {object_ref:?} has a wrong amount of slots; expected {expected} but got {actual}."
    )]
    WrongAmountOfSlots {
        object_ref: ObjectRef,
        expected: usize,
        actual: usize,
    },
}

pub type Result<T> = core::result::Result<T, Error>;
