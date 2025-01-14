use crate::{ObjectFormat, ObjectRef, RawObjectPointer};
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
    #[error("Forwarded objects are not supported for this operation")]
    ForwardedUnsupported,
    #[error("Expected an object of type {0}")]
    InvalidType(String)
}

pub type Result<T> = core::result::Result<T, Error>;