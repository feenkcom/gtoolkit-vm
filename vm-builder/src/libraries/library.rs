use crate::options::FinalOptions;
use std::path::PathBuf;

pub trait Library {
    fn is_downloaded(&self, options: &FinalOptions) -> bool;
    fn download(&self, options: &FinalOptions) {
        if self.is_downloaded(options) {
            return;
        };
        self.force_download(options);
    }
    fn force_download(&self, options: &FinalOptions);

    fn is_compiled(&self, options: &FinalOptions) -> bool {
        self.compiled_library(options).exists()
    }

    fn compile(&self, options: &FinalOptions) {
        if self.is_compiled(options) {
            return;
        }
        self.force_compile(options);
    }

    fn force_compile(&self, options: &FinalOptions);

    fn compiled_library(&self, options: &FinalOptions) -> PathBuf;

    fn ensure_requirements(&self);
}
