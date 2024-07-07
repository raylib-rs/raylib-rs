//! Definitions for error types used throught the crate

// NOTE: These errors assume that all the error strings are known at compile-time.
// If there is a need to make the contained type hold an owned value - make them generic

use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
#[repr(transparent)]
#[error("{0}")]
pub struct RaylibError(pub(crate) &'static str);

#[derive(Error, Debug)]
#[error(
    "{message}\npath: {path}",
    path = path.display(),
)]
pub struct RaylibPathError {
    pub(crate) message: &'static str,
    pub(crate) path: PathBuf,
}

impl RaylibPathError {
    pub(crate) const fn new(message: &'static str, path: PathBuf) -> Self {
        Self { message, path }
    }
}
