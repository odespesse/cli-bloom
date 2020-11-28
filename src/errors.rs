use std::error::Error as StdError;
use std::io;
use std::fmt;
use index_bloom::Error as IndexBloomError;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    IndexInvalidData(io::Error),
    IndexError(IndexBloomError)
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(error) => Some(error),
            Error::IndexInvalidData(error) => Some(error),
            Error::IndexError(error) => Some(error)
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(_) => write!(f, "Error reading file"),
            Error::IndexInvalidData(_) => write!(f, "Error source must be an UTF-8 text file"),
            Error::IndexError(_) => write!(f, "Error from index")
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        match error.kind() {
            io::ErrorKind::InvalidData => return Error::IndexInvalidData(error),
            _ => Error::Io(error)
        }
    }
}

impl From<IndexBloomError> for Error {
    fn from(error: IndexBloomError) -> Error {
        Error::IndexError(error)
    }
}
