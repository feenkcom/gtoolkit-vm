use crate::{ApplicationError, Result};
use clap::{AppSettings, ArgEnum, Parser, PossibleValue};
use std::fmt::format;
use std::path::{Path, PathBuf};

lazy_static! {
    pub static ref WORKER_HELP: String = WorkerThreadMode::long_help();
}

#[derive(Parser, Clone, Debug)]
#[clap(author = "feenk gmbh <contact@feenk.com>")]
#[clap(global_setting(AppSettings::NoAutoVersion))]
pub struct AppOptions {
    /// A path to a custom Pharo .image to use instead of automatically detecting one
    #[clap(long, parse(from_os_str))]
    image: Option<PathBuf>,
    #[clap(long, value_name = "MODE", arg_enum, default_value_t = WorkerThreadMode::Auto, long_help = WorkerThreadMode::long_help_str())]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
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

    pub fn possible_values() -> impl Iterator<Item = PossibleValue<'static>> {
        Self::value_variants()
            .iter()
            .filter_map(ArgEnum::to_possible_value)
    }

    pub fn long_help() -> String {
        let values = Self::possible_values()
            .map(|each| format!("   {}: {}.", each.get_name(), each.get_help().unwrap_or("")))
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            "Select if the pharo interpreter should be run in a worker thread.\n\n\
            It can be configured with the following options:\n\
            {}\n",
            values
        )
    }

    pub fn long_help_str() -> &'static str {
        WORKER_HELP.as_str()
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
