use crate::{ApplicationError, Result};
use clap::builder::{PossibleValue, StyledStr};
use clap::{Parser, ValueEnum};
use std::fmt::format;
use std::path::{Path, PathBuf};

#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about = None)]
pub struct AppOptions {
    /// A path to a custom Pharo .image to use instead of automatically detecting one
    #[clap(long)]
    image: Option<PathBuf>,
    #[clap(long, value_name = "MODE", value_enum, default_value_t = WorkerThreadMode::Auto, long_help)]
    /// Choose whether to run Pharo in a worker thread
    worker: WorkerThreadMode,
    /// Print the version information of the executable.
    #[clap(long, short = 'V')]
    pub version: bool,
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

    pub fn should_run_in_worker_thread(&self) -> bool {
        self.worker.should_run_in_worker_thread()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum WorkerThreadMode {
    /// Run the pharo interpreter in a worker thread freeing the main thread.
    Yes,
    /// Run the pharo interpreter on the main application thread.
    No,
    /// Automatically decide whether pharo interpreter should be run in a worker thread based on the current platform and support.
    Auto,
}

impl WorkerThreadMode {
    pub fn should_run_in_worker_thread(&self) -> bool {
        match self {
            WorkerThreadMode::Yes => true,
            WorkerThreadMode::No => false,
            WorkerThreadMode::Auto => cfg!(target_os = "macos") || cfg!(target_os = "windows"),
        }
    }

    pub fn possible_values() -> impl Iterator<Item = PossibleValue> {
        Self::value_variants()
            .iter()
            .filter_map(ValueEnum::to_possible_value)
    }
}

impl std::str::FromStr for WorkerThreadMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("Invalid variant: {}", s))
    }
}
