pub type BoxError = std::boxed::Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, BoxError>;

#[derive(Debug)]
pub struct Error {
    message: String,
    source: Option<BoxError>,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    pub fn result<T>(self) -> Result<T> {
        Err(Box::new(self))
    }

    pub fn from(self, error: impl Into<Box<dyn std::error::Error>>) -> Self {
        Self {
            message: self.message,
            source: Some(error.into()),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(error) = &self.source {
            write!(f, "\nCaused by: {}", error)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|error| error.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl<T> From<Error> for Result<T> {
    fn from(error: Error) -> Self {
        error.result()
    }
}

unsafe impl Sync for Error {}
unsafe impl Send for Error {}
