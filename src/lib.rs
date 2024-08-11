mod browser;
pub mod tui;
mod components;

use std::{fmt::{self, Display, Formatter}, io, result};

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::IOError(e) => f.write_fmt(format_args!("{}", e)),
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
