use crate::{ApplicationError, Result};
use clap::{AppSettings, Parser};
use std::path::{Path, PathBuf};

#[derive(Parser, Clone, Debug)]
#[clap(version = "1.0", author = "feenk gmbh <contact@feenk.com>")]
pub struct AppOptions {
    /// A path to a custom Pharo .image to use instead of automatically detecting one
    #[clap(long, parse(from_os_str))]
    image: Option<PathBuf>,
}

impl AppOptions {
    pub fn canonicalize(&mut self) -> Result<()> {
        if let Some(ref image) = self.image {
            if !image.exists() {
                return ApplicationError::ImageFileDoesNotExist(image.clone()).into();
            }
            self.image = Some(to_absolute::canonicalize(image)?);
        }
        Ok(())
    }

    pub fn image(&self) -> Option<&Path> {
        self.image.as_ref().map(|image| image.as_path())
    }
}
