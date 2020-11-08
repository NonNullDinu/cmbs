use crate::diagnostics::DiagConfig;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    error_cascade: bool,
    pub(crate) output_directory: PathBuf,
    signal_build_failure: bool,
    pub(crate) diagnostics_config: DiagConfig,
}

impl Config {
    #[must_use]
    pub fn new(error_cascade: bool, output_directory: PathBuf, signal_build_failure: bool) -> Self {
        Self {
            error_cascade,
            output_directory,
            signal_build_failure,
            diagnostics_config: DiagConfig::default(),
        }
    }
}
