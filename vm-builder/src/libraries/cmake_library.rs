use crate::options::FinalOptions;
use crate::Library;
use std::path::PathBuf;
use url::Url;

pub struct CMakeLibrary {
    name: String,
    repository: Url,
    tag: Option<String>,
}

impl CMakeLibrary {}

impl Library for CMakeLibrary {
    fn is_downloaded(&self, options: &FinalOptions) -> bool {
        unimplemented!()
    }

    fn force_download(&self, options: &FinalOptions) {
        unimplemented!()
    }

    fn force_compile(&self, options: &FinalOptions) {
        unimplemented!()
    }

    fn compiled_library(&self, options: &FinalOptions) -> PathBuf {
        unimplemented!()
    }

    fn ensure_requirements(&self) {
        unimplemented!()
    }
}
