use url::Url;
use crate::Library;
use crate::options::FinalOptions;
use std::path::PathBuf;

pub struct CMakeLibrary {
    name: String,
    repository: Url,
    tag: Option<String>,
}

impl CMakeLibrary {

}

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
}