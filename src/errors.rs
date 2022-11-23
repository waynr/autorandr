use std::io;
use std::convert::Infallible;
use std::path::PathBuf;

use thiserror::Error;
use xrandr;
use serde_yaml;

/// The Result type for ghmd.
pub type Result<T> = std::result::Result<T, Error>;

/// The Error type for autorandr.
#[derive(Error, Debug)]
pub enum Error {
    /// Wrapper around `io::Error`.
    #[error("error: {0}")]
    StdIOError(#[from] io::Error),

    /// Xrandr library error.
    #[error("xrandr error: {0}")]
    XrandrError(#[from] xrandr::XrandrError),

    /// Logger failed to initialize.
    #[error("log initialization error: {0}")]
    SetLoggerError(#[from] log::SetLoggerError),

    /// Failed to decode hex value.
    #[error("failed to decode hex: {0}")]
    FromHexError(#[from] hex::FromHexError),

    /// Yaml serialization/deserialization failiure.
    #[error("failed to serialize (or deserialize): {0}")]
    SerdeYamlError(#[from] serde_yaml::Error),

    #[error("cannot determine configuration directory on this platform")]
    CannotDetermineConfigDir,

    #[error("invalid profile data: {0}")]
    UnrecognizedProfileConfigFile(PathBuf),

    #[error("invalid monitor data: {0}")]
    UnrecognizedMonitorConfigFile(PathBuf),

}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}
