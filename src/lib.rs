pub mod browser;
mod components;
pub mod config;
pub mod tui;

use std::{
    fmt::{self, Display, Formatter},
    io, result,
};

use homedir::GetHomeError;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    GetHomeError(GetHomeError),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<GetHomeError> for Error {
    fn from(value: GetHomeError) -> Self {
        Self::GetHomeError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::IOError(e) => f.write_fmt(format_args!("{}", e)),
            Error::GetHomeError(e) => f.write_fmt(format_args!("{}", e)),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//     }
// }
