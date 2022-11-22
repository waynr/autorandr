use std::convert::Infallible;

use thiserror::Error;
use xrandr;

/// The Result type for ghmd.
pub type Result<T> = std::result::Result<T, Error>;

/// The Error type for autorandr.
#[derive(Error, Debug)]
pub enum Error {
    /// Xrandr library error.
    #[error("xrandr error: {0}")]
    XrandrError(#[from] xrandr::XrandrError),

    /// Logger failed to initialize.
    #[error("log initialization error: {0}")]
    SetLoggerError(#[from] log::SetLoggerError),

    /// Config toml is malformed.
    #[error("failed to decode hex: {0}")]
    FromHexError(#[from] hex::FromHexError),
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}
