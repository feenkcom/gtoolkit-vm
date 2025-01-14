use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::mpsc::{RecvError, TryRecvError};
use thiserror::Error;

pub type Result<T> = core::result::Result<T, ApplicationError>;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Object memory error")]
    ObjectMemoryError(#[from] vm_object_model::Error),
    #[error("Input/Output error")]
    IoError(#[from] std::io::Error),
    #[cfg(target_os = "macos")]
    #[error("Failed to load the library")]
    LibLoadingError(#[from] libloading::Error),
    #[error("Failed to canonicalize a path")]
    CanonicalizationError(#[from] to_absolute::Error),
    #[error("Failed to detect if the executable is translocated")]
    FailedToDetectIfTranslocated,
    #[error("Failed to detect the original translocated path")]
    FailedToDetectOriginalTranslocatedPath,
    #[error("Failed to open terminal")]
    FailedToOpenTerminal,
    #[error("Could not get an executable from the arguments")]
    NoExecutableInArguments,
    #[error("Failed to get the file name of `{0}`")]
    FailedToGetFileName(PathBuf),
    #[error("Failed to convert `{0:?}` from OsString to String")]
    FailedToConvertOsString(OsString),
    #[error("Provided image does not exist: `{0}`")]
    ImageFileDoesNotExist(PathBuf),
    #[error("Could not find any .image file")]
    ImageFileNotFound,
    #[error("Directory `{0}` does not have a parent")]
    NoParentDirectory(PathBuf),
    #[error("Failed to receive an event loop message")]
    EventLoopReceiverError(#[from] RecvError),
    #[error("Failed to receive an event loop message")]
    EventLoopTryReceiverError(#[from] TryRecvError),
    #[error("Failed to join a thread")]
    JoinHandleError,
    #[error("unknown data store error")]
    Unknown,
}

impl<T> From<ApplicationError> for std::result::Result<T, ApplicationError> {
    fn from(error: ApplicationError) -> Self {
        Err(error)
    }
}
